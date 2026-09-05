use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::platform::http::Http;

pub const API: &str = "https://api.github.com";
pub const ARTIFACT_CEILING: usize = 256 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct Runs {
    pub workflow_runs: Vec<Run>,
}

#[derive(Debug, Deserialize)]
pub struct Run {
    pub id: u64,
    pub status: String,
    pub conclusion: Option<String>,
    pub head_sha: String,
    pub artifacts_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Artifacts {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub archive_download_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not reach GitHub while fetching {resource}")]
    Unreachable { resource: &'static str },
    #[error("GitHub answered {status} when asked for {resource}")]
    Rejected { resource: &'static str, status: u16 },
    #[error("GitHub's answer for {resource} did not parse")]
    Unreadable { resource: &'static str },
    #[error("this repository has never run a workflow")]
    NeverBuilt,
    #[error("build #{id} is {state}, not a success")]
    Unfinished { id: u64, state: String },
    #[error("the latest build produced no artifact for this platform")]
    NoArtifact,
    #[error("the artifact is {size} bytes, over the {ARTIFACT_CEILING} byte ceiling")]
    TooLarge { size: usize },
}

impl Error {
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Error::Rejected {
                status: 401 | 403 | 404,
                ..
            } => Some("a private repository needs `github_token` with actions:read"),
            Error::NoArtifact => Some("check that the workflow uploads one artifact per platform"),
            _ => None,
        }
    }
}

pub fn usable(run: &Run) -> Result<(), Error> {
    if run.status != "completed" {
        return Err(Error::Unfinished {
            id: run.id,
            state: run.status.clone(),
        });
    }

    let state = match run.conclusion.as_deref() {
        Some("success") => return Ok(()),
        Some(other) => String::from(other),
        None => String::from("completed without a verdict"),
    };

    Err(Error::Unfinished { id: run.id, state })
}

pub fn wanted(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".exe") == cfg!(windows)
}

pub struct Source<'a> {
    http: &'a Http,
    token: Option<&'a str>,
}

impl<'a> Source<'a> {
    pub fn new(http: &'a Http, token: Option<&'a str>) -> Self {
        Self { http, token }
    }

    fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let request = self.http.client().get(url);

        match self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    async fn read<T: DeserializeOwned>(
        &self,
        resource: &'static str,
        url: &str,
    ) -> Result<T, Error> {
        let response = self
            .get(url)
            .send()
            .await
            .map_err(|_| Error::Unreachable { resource })?;

        let status = response.status();

        if !status.is_success() {
            return Err(Error::Rejected {
                resource,
                status: status.as_u16(),
            });
        }

        response
            .json()
            .await
            .map_err(|_| Error::Unreadable { resource })
    }

    pub async fn latest(&self, repo: &str) -> Result<Run, Error> {
        let runs: Runs = self
            .read(
                "the latest workflow run",
                &(format!("{API}/repos/{repo}/actions/runs?per_page=1")),
            )
            .await?;

        runs.workflow_runs
            .into_iter()
            .next()
            .ok_or(Error::NeverBuilt)
    }

    pub async fn artifact(&self, run: &Run) -> Result<Artifact, Error> {
        let listed: Artifacts = self
            .read("the build's artifacts", &run.artifacts_url)
            .await?;

        listed
            .artifacts
            .into_iter()
            .find(|artifact| wanted(&artifact.name))
            .ok_or(Error::NoArtifact)
    }

    pub async fn archive(&self, artifact: &Artifact) -> Result<Vec<u8>, Error> {
        let resource = "the artifact archive";

        let response = self
            .get(&artifact.archive_download_url)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|_| Error::Unreachable { resource })?;

        let status = response.status();

        if !status.is_success() {
            return Err(Error::Rejected {
                resource,
                status: status.as_u16(),
            });
        }

        if let Some(declared) = response.content_length()
            && declared as usize > ARTIFACT_CEILING
        {
            return Err(Error::TooLarge {
                size: declared as usize,
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|_| Error::Unreachable { resource })?;

        match bytes.len() > ARTIFACT_CEILING {
            true => Err(Error::TooLarge { size: bytes.len() }),
            false => Ok(bytes.to_vec()),
        }
    }
}
