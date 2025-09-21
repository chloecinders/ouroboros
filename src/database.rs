use sqlx::query;
use tracing::info;

use crate::SQL;

#[derive(Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "action_type", rename_all = "lowercase")]
pub enum ActionType {
    Warn,
    Kick,
    Ban,
    Softban,
    Mute,
    Unban,
    Unmute,
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
    create_action_type_201420250826().await;
    add_log_bot_to_guild_settings_220420250829().await;
    add_log_mod_to_guild_settings_021020250918().await;
    remove_log_mod_and_change_channel_id_to_jsonb_21092025().await;
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
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!("Couldnt run database migration create_actions_223320250818; Err = {err:?}");
    }
}

pub async fn create_guild_settings_195120250826() {
    if let Err(err) = query!(
        r#"
        CREATE TABLE IF NOT EXISTS public.guild_settings
        (
            guild_id bigint NOT NULL PRIMARY KEY,
            log_channel bigint
        )
        "#
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!("Couldnt run database migration create_guild_settings_195120250826; Err = {err:?}");
    }
}

pub async fn create_action_type_201420250826() {
    if let Err(err) = query!(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1
                FROM pg_type t
                JOIN pg_namespace n ON n.oid = t.typnamespace
                WHERE t.typname = 'action_type'
                AND n.nspname = 'public'
            ) THEN
                CREATE TYPE public.action_type AS ENUM
                    ('warn', 'ban', 'kick', 'softban', 'timeout', 'unban', 'mute', 'unmute');
            END IF;
        END$$;
        "#
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!("Couldnt run database migration create_guild_settings_195120250826; Err = {err:?}");
    }
}

pub async fn add_log_bot_to_guild_settings_220420250829() {
    if let Err(err) = query!(
        r#"
        ALTER TABLE public.guild_settings
        ADD COLUMN IF NOT EXISTS log_bot BOOLEAN
        "#
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!(
            "Couldnt run database migration add_log_bot_to_guild_settings_220420250829; Err = {err:?}"
        );
    }
}

pub async fn add_log_mod_to_guild_settings_021020250918() {
    if let Err(err) = query!(
        r#"
        ALTER TABLE public.guild_settings
        ADD COLUMN IF NOT EXISTS log_mod bigint
        "#
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!(
            "Couldnt run database migration add_log_mod_to_guild_settings_021020250918; Err = {err:?}"
        );
    }
}

pub async fn remove_log_mod_and_change_channel_id_to_jsonb_21092025() {
    if let Err(err) = query!(
        r#"
        ALTER TABLE public.guild_settings
        DROP COLUMN IF EXISTS log_mod,
        DROP COLUMN IF EXISTS log_channel,
        ADD COLUMN IF NOT EXISTS log_channel_ids jsonb
        "#
    )
    .execute(SQL.get().unwrap())
    .await
    {
        panic!(
            "Couldnt run database migration remove_log_mod_and_change_channel_id_to_jsonb_21092025; Err = {err:?}"
        );
    }
}
