use std::{collections::HashSet, sync::LazyLock};

use base64::Engine;
use regex::Regex;
use serde_json::json;
use tracing::warn;

use crate::BOT_CONFIG;

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[MNO][a-zA-Z0-9]{22,28}\.[a-zA-Z0-9_-]{6,8}\.[a-zA-Z0-9_-]{27,40}").unwrap()
});

pub fn is_valid_discord_token(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(parts[0])
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[0]))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(parts[0]));

    match decoded {
        Ok(bytes) => {
            if bytes.is_empty() || !bytes.iter().all(|b| b.is_ascii_digit()) {
                return false;
            }
            (15..=22).contains(&bytes.len())
        }
        Err(_) => false,
    }
}

pub async fn process_tokens(input: &str, reason: &str) -> bool {
    let mut found_tokens = HashSet::new();

    for cap in TOKEN_REGEX.captures_iter(input) {
        if let Some(m) = cap.get(0) {
            let cleaned = m.as_str().trim();
            if !cleaned.is_empty() && is_valid_discord_token(cleaned) {
                found_tokens.insert(cleaned.to_string());
            }
        }
    }

    let no_spaces: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    for cap in TOKEN_REGEX.captures_iter(&no_spaces) {
        if let Some(m) = cap.get(0) {
            let cleaned = m.as_str().trim();
            if !cleaned.is_empty() && is_valid_discord_token(cleaned) {
                found_tokens.insert(cleaned.to_string());
            }
        }
    }

    if found_tokens.is_empty() {
        return false;
    }

    let client = reqwest::Client::new();

    for token in &found_tokens {
        let bot_res = client
            .get("https://discord.com/api/v10/users/@me")
            .header("Authorization", format!("Bot {}", token))
            .send()
            .await;

        let is_bot = match bot_res {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        };

        if is_bot {
            if let (Some(repo), Some(github_token)) = (
                BOT_CONFIG.reset_token_repository.clone(),
                BOT_CONFIG.github_token.clone(),
            ) {
                match client
                    .post(format!("https://api.github.com/repos/{repo}/issues"))
                    .header("Authorization", format!("Bearer {github_token}"))
                    .header(
                        "User-Agent",
                        format!("Aegis Bot v{}", env!("CARGO_PKG_VERSION")),
                    )
                    .json(&json!({
                        "title": format!("Token Reset Request - {}", reason),
                        "body": token
                    }))
                    .send()
                    .await
                {
                    Ok(res) => {
                        if !res.status().is_success() {
                            let status = res.status();
                            let error_body = res.text().await.unwrap_or_default();

                            warn!(
                                "GitHub API issue posting failed with status: {:?}; body: {}",
                                status, error_body
                            );
                        }
                    }
                    Err(err) => {
                        warn!("Failed to send GitHub issue request; err = {err:?}");
                    }
                }
            } else {
                warn!("reset_token_repository or github_token is not configured in BOT_CONFIG");
            }
            continue;
        }

        let user_res = client
            .get("https://discord.com/api/v10/users/@me")
            .header("Authorization", token)
            .send()
            .await;

        let is_user = match user_res {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        };

        if is_user {
            match client
                .post("https://discord.com/api/v10/auth/logout")
                .header("Authorization", token)
                .header("Content-Type", "application/json")
                .json(&json!({
                    "provider": null,
                    "voip_provider": null
                }))
                .send()
                .await
            {
                Ok(res) => {
                    if !res.status().is_success() {
                        let status = res.status();
                        let error_body = res.text().await.unwrap_or_default();
                        warn!(
                            "Discord auth logout failed with status: {:?}; body: {}",
                            status, error_body
                        );
                    }
                }
                Err(err) => {
                    warn!("Failed to send Discord auth logout request; err = {err:?}");
                }
            }
        }
    }

    true
}
