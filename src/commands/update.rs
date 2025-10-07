use reqwest::{Client, Method, Request, Url, header::HeaderValue};
use serenity::{
    all::{CacheHttp, Context, Message},
    async_trait,
};
use tracing::warn;

use crate::{
    BOT_CONFIG,
    commands::{Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions, CommandSyntax, TransformerFnArc},
    event_handler::CommandError,
    lexer::Token,
    utils::is_developer,
};
use ouroboros_macros::command;
use std::process::exit;

pub struct Update;

impl Update {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Update {
    fn get_name(&self) -> &'static str {
        "update"
    }

    fn get_short(&self) -> &'static str {
        "Updates the bot remotely"
    }

    fn get_full(&self) -> &'static str {
        "Updates the Bot using the Github repository in the config. \
        Warning: This might print debug information in chat! Only run this in a channel with members you trust!"
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Developer
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(&self, ctx: Context, msg: Message) -> Result<(), CommandError> {
        if !is_developer(&msg.author) {
            return Ok(());
        }

        let cfg = BOT_CONFIG.get().unwrap();
        let Some(repo) = cfg.repository.clone() else {
            warn!("Update command disabled! Please set a repository in the config!");
            let _ = msg
                .reply(
                    &ctx.http(),
                    "Update command disabled! Please set a repository in the config",
                )
                .await;
            return Ok(());
        };

        let client = Client::new();
        let mut request = Request::new(
            Method::GET,
            Url::parse(
                format!("https://api.github.com/repos/{repo}/actions/runs?per_page=1").as_str(),
            )
            .unwrap(),
        );
        let headers = request.headers_mut();
        headers.append(
            "User-Agent",
            HeaderValue::from_str(format!("Ouroboros Bot v{}", env!("CARGO_PKG_VERSION")).as_str())
                .unwrap(),
        );

        if let Some(token) = cfg.github_token.clone() {
            headers.append(
                "Authorization",
                HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
            );
        }

        let res = match client.execute(request).await {
            Ok(o) => o,
            Err(e) => {
                let err = format!(
                    "Error getting actions, make sure to set a Github token with enough permissions if your repository is private; err = {e:?}"
                );
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }
        };

        if res.status() != 200 {
            let err = format!(
                "Error getting actions, make sure to set a Github token with enough permissions if your repository is private; res = {res:?}"
            );
            warn!(err);
            let _ = msg.reply(&ctx.http(), err).await;
            return Ok(());
        }

        let json = match res.json::<WorkflowRunsResponse>().await {
            Ok(r) => r,
            Err(err) => {
                let err = format!("Error deserializing actions response; err = {err:?}");
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }
        };

        if let Some(run) = json.workflow_runs.first() {
            if run.status != "completed" || run.conclusion.clone().is_none_or(|c| c != "success") {
                let err = format!(
                    "Latest run with id {} is not successful! Fix your code idiot!",
                    run.id
                );
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }

            let mut artifacts_req =
                Request::new(Method::GET, Url::parse(&run.artifacts_url).unwrap());
            let headers = artifacts_req.headers_mut();
            headers.append(
                "User-Agent",
                HeaderValue::from_str(
                    format!("Ouroboros Bot v{}", env!("CARGO_PKG_VERSION")).as_str(),
                )
                .unwrap(),
            );

            if let Some(token) = cfg.github_token.clone() {
                headers.append(
                    "Authorization",
                    HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
                );
            }

            let res = match client.execute(artifacts_req).await {
                Ok(o) => o,
                Err(e) => {
                    let err = format!("Error fetching artifacts; err = {e:?}");
                    warn!(err);
                    let _ = msg.reply(&ctx.http(), err).await;
                    return Ok(());
                }
            };

            if res.status() != 200 {
                let err = format!("Error fetching artifacts; res = {res:?}");
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }

            let json = match res.json::<ArtifactsResponse>().await {
                Ok(r) => r,
                Err(err) => {
                    let err = format!("Error deserializing artifacts response; err = {err:?}");
                    warn!(err);
                    let _ = msg.reply(&ctx.http(), err).await;
                    return Ok(());
                }
            };

            #[cfg(target_os = "windows")]
            fn artifact_matches(name: &str) -> bool {
                name.ends_with(".exe")
            }

            #[cfg(not(target_os = "windows"))]
            fn artifact_matches(name: &str) -> bool {
                !name.ends_with(".exe")
            }

            let Some(artifact) = json
                .artifacts
                .into_iter()
                .find(|a| artifact_matches(&a.name))
            else {
                let err =
                    "No artifact found. Check if the latest action produced the correct artifacts";
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            };

            let mut download_req = Request::new(
                Method::GET,
                Url::parse(&artifact.archive_download_url).unwrap(),
            );
            let headers = download_req.headers_mut();
            headers.append(
                "User-Agent",
                HeaderValue::from_str(
                    format!("Ouroboros Bot v{}", env!("CARGO_PKG_VERSION")).as_str(),
                )
                .unwrap(),
            );

            if let Some(token) = cfg.github_token.clone() {
                headers.append(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
                );
            }

            let res = match client.execute(download_req).await {
                Ok(o) => o,
                Err(e) => {
                    let err = format!("Error fetching artifact file; err = {e:?}");
                    warn!(err);
                    let _ = msg.reply(&ctx.http(), err).await;
                    return Ok(());
                }
            };

            if res.status() != 200 {
                let err = format!("Error fetching artifact file; res = {res:?}");
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }

            let Ok(bytes) = res.bytes().await else {
                let err = String::from("Error fetching artifact file;");
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            };

            let filename = String::from("new_") + artifact.name.as_str();

            let unzip_result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
                let reader = std::io::Cursor::new(bytes);
                let Ok(mut zip) = zip::ZipArchive::new(reader) else {
                    return Err(String::from("Failed to create a zip cursor"));
                };

                #[cfg(target_os = "windows")]
                let name = "Ouroboros.exe";
                #[cfg(not(target_os = "windows"))]
                let name = "Ouroboros";

                let Ok(mut file) = zip.by_name(&format!("release/{name}")) else {
                    return Err(String::from("Failed to extract file"));
                };

                let mut buffer = Vec::with_capacity(file.size() as usize);

                if let Err(err) = std::io::copy(&mut file, &mut buffer) {
                    return Err(format!("{err:?}"));
                }

                Ok(buffer)
            })
            .await;

            let extracted_bytes = match unzip_result {
                Ok(r) => match r {
                    Ok(b) => b,
                    Err(err) => {
                        let err = format!("Failed unzipping artifact zip; err = {err:?}");
                        warn!(err);
                        let _ = msg.reply(&ctx.http(), err).await;
                        return Ok(());
                    }
                },
                Err(err) => {
                    let err = format!("Failed unzipping artifact zip; err = {err:?}");
                    warn!(err);
                    let _ = msg.reply(&ctx.http(), err).await;
                    return Ok(());
                }
            };

            if let Err(err) = tokio::fs::write(filename.clone(), &extracted_bytes).await {
                let err = format!("Failed writing artifact file; err = {err:?}");
                warn!(err);
                let _ = msg.reply(&ctx.http(), err).await;
                return Ok(());
            }

            #[cfg(not(target_os = "windows"))]
            {
                use std::fs;
                use std::os::unix::fs::PermissionsExt;

                let _ = fs::write(
                    "./update.txt",
                    format!("{}:{}", msg.channel_id.get(), msg.id.get()),
                );

                let target = "./Ouroboros";

                let _ = fs::remove_file(target);

                if let Err(err) = fs::copy(&filename, target) {
                    warn!("Failed to copy file; err = {:?}", err);
                    return Ok(());
                }

                if let Err(err) = fs::set_permissions(target, fs::Permissions::from_mode(0o755)) {
                    warn!("Failed to set permissions; err = {:?}", err);
                    return Ok(());
                }

                exit(0);
            }

            #[cfg(target_os = "windows")]
            {
                use std::process::Command as SystemCommand;

                let child =
                    match SystemCommand::new(format!(".{}{filename}", std::path::MAIN_SEPARATOR))
                        .arg(format!(
                            "--update={}:{}",
                            msg.channel_id.get(),
                            msg.id.get()
                        ))
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            let err = format!("Could not run downloaded version; err = {e:?}");
                            warn!(err);
                            let _ = msg.reply(&ctx.http(), err).await;
                            return Ok(());
                        }
                    };

                drop(child);
                exit(0);
            }
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![],
            one_of: vec![],
        }
    }
}

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub status: String,
    pub conclusion: Option<String>,
    pub artifacts_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ArtifactsResponse {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub archive_download_url: String,
}
