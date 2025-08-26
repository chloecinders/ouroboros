use sqlx::query;
use tracing::info;

use crate::SQL;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "action_type", rename_all="lowercase")]
pub enum ActionType {
    Warn,
    Kick,
    Ban,
    Softban,
    Mute,
    Unban,
    Unmute
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Warn => write!(f, "warn"),
            ActionType::Kick => write!(f, "kick"),
            ActionType::Ban => write!(f, "ban"),
            ActionType::Softban => write!(f, "softban"),
            ActionType::Mute => write!(f, "mute"),
            ActionType::Unban => write!(f, "unban"),
            ActionType::Unmute => write!(f, "unmute"),
        }
    }
}

pub async fn run_migrations() {
    info!("Running database migrations");
    create_actions_223320250818().await;
    create_guild_settings_195120250826().await;
}

pub async fn create_actions_223320250818() {
    if let Err(err) = query!(
        r#"
        CREATE TABLE IF NOT EXISTS public.actions
        (
            guild_id bigint NOT NULL,
            user_id bigint NOT NULL,
            reason text COLLATE pg_catalog."default" NOT NULL,
            moderator_id bigint NOT NULL,
            created_at timestamp without time zone NOT NULL DEFAULT now(),
            updated_at timestamp without time zone,
            id character varying(128) COLLATE pg_catalog."default" NOT NULL,
            type action_type NOT NULL DEFAULT 'warn'::action_type,
            active boolean NOT NULL DEFAULT true,
            expires_at timestamp without time zone,
            CONSTRAINT warns_pkey PRIMARY KEY (id)
        )
        "#
    ).execute(SQL.get().unwrap()).await {
        panic!("Couldnt run database migration create_actions_223320250818; Err = {err:?}");
    }
}

pub async fn create_guild_settings_195120250826() {
    if let Err(err) = query!(
        r#"
        CREATE TABLE IF NOT EXISTS public.guild_settings
        (
            guild_id bigint NOT NULL,
            log_channel bigint
        )
        "#
    ).execute(SQL.get().unwrap()).await {
        panic!("Couldnt run database migration create_guild_settings_195120250826; Err = {err:?}");
    }
}
