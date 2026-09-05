ALTER TABLE IF EXISTS public.actions RENAME TO legacy_actions;
ALTER TABLE IF EXISTS public.action_refs RENAME TO legacy_action_refs;
ALTER TABLE IF EXISTS public.automod_rules RENAME TO legacy_automod_rules;
ALTER TABLE IF EXISTS public.guild_settings RENAME TO legacy_guild_settings;
ALTER TABLE IF EXISTS public.guild_encryption RENAME TO legacy_guild_encryption;
ALTER TABLE IF EXISTS public.log_messages_context RENAME TO legacy_log_messages;
ALTER TABLE IF EXISTS public.message_cache_store RENAME TO legacy_channel_budget;
ALTER TABLE IF EXISTS public.message_edits RENAME TO legacy_message_edits;
ALTER TABLE IF EXISTS public.message_store RENAME TO legacy_messages;
ALTER TABLE IF EXISTS public.sticky_messages RENAME TO legacy_sticky_messages;
ALTER TABLE IF EXISTS public.transcripts RENAME TO legacy_transcripts;

ALTER TABLE IF EXISTS public.channel_message_budget RENAME TO discarded_channel_message_budget;
ALTER TABLE IF EXISTS public.command_traces RENAME TO discarded_command_traces;
ALTER TABLE IF EXISTS public.guild_log_channels RENAME TO discarded_guild_log_channels;
ALTER TABLE IF EXISTS public.guild_permissions RENAME TO discarded_guild_permissions;
ALTER TABLE IF EXISTS public.invocations RENAME TO discarded_invocations;
ALTER TABLE IF EXISTS public.log_messages RENAME TO discarded_log_messages;
ALTER TABLE IF EXISTS public.messages RENAME TO discarded_messages;
ALTER TABLE IF EXISTS public.ocr_image_evaluations RENAME TO discarded_ocr_image_evaluations;
ALTER TABLE IF EXISTS public.ocr_image_hashes RENAME TO discarded_ocr_hashes;
ALTER TABLE IF EXISTS public.scheduled_work RENAME TO discarded_scheduled_work;
ALTER TABLE IF EXISTS public.transcript_messages RENAME TO discarded_transcript_messages;

CREATE TABLE IF NOT EXISTS public.legacy_actions (
    id varchar(128) NOT NULL,
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    moderator_id bigint NOT NULL,
    reason text NOT NULL,
    note varchar(128),
    type text NOT NULL,
    active boolean NOT NULL DEFAULT true,
    created_at timestamp NOT NULL DEFAULT now(),
    updated_at timestamp,
    expires_at timestamp,
    last_reapplied_at timestamptz
);

CREATE TABLE IF NOT EXISTS public.legacy_action_refs (
    action_id varchar(128) NOT NULL,
    image_url text,
    ref_message_id bigint,
    ref_channel_id bigint,
    ref_author_id bigint,
    ref_content bytea
);

CREATE TABLE IF NOT EXISTS public.legacy_automod_rules (
    id varchar(128) NOT NULL,
    guild_id bigint NOT NULL,
    name varchar(128) NOT NULL,
    type varchar(128) NOT NULL,
    rule varchar(512) NOT NULL,
    is_regex boolean NOT NULL DEFAULT false,
    created_at timestamp NOT NULL DEFAULT now(),
    reason text NOT NULL,
    punishment_type text NOT NULL DEFAULT 'warn',
    duration bigint,
    day_clear_amount smallint,
    silent boolean,
    log_channel_id bigint
);

CREATE TABLE IF NOT EXISTS public.legacy_guild_settings (
    guild_id bigint NOT NULL,
    log_bot boolean,
    log_channel_ids jsonb
);

CREATE TABLE IF NOT EXISTS public.legacy_guild_encryption (
    guild_id bigint NOT NULL,
    encrypted boolean NOT NULL DEFAULT false,
    key_channel_id bigint,
    key_message_id bigint
);

CREATE TABLE IF NOT EXISTS public.legacy_log_messages (
    message_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    target_id bigint NOT NULL,
    moderator_id bigint NOT NULL,
    db_id varchar(128),
    content bytea
);

CREATE TABLE IF NOT EXISTS public.legacy_channel_budget (
    channel_id bigint NOT NULL,
    message_count integer NOT NULL DEFAULT 0,
    previous_action smallint NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS public.legacy_message_edits (
    edit_id bigint NOT NULL,
    message_id bigint NOT NULL,
    content bytea,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.legacy_messages (
    message_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    author_id bigint NOT NULL,
    author_name text NOT NULL,
    author_display_name text,
    author_avatar_url text,
    content bytea,
    embeds bytea,
    attachment_urls jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS public.legacy_sticky_messages (
    channel_id bigint NOT NULL,
    content text NOT NULL,
    title text,
    color bigint,
    last_message_id bigint
);

CREATE TABLE IF NOT EXISTS public.legacy_transcripts (
    transcript_id text NOT NULL,
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    channel_name text,
    moderator_name text NOT NULL,
    message_ids jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TYPE public.punishment_verb AS ENUM (
    'warn',
    'kick',
    'ban',
    'softban',
    'mute',
    'unban',
    'unmute'
);

CREATE TYPE public.punishment_state AS ENUM (
    'pending',
    'active',
    'expiring',
    'ended',
    'revoked',
    'lapsed',
    'failed'
);

CREATE TYPE public.rule_mode AS ENUM ('disabled', 'active');

CREATE TYPE public.work_kind AS ENUM (
    'lift_ban',
    'lift_mute',
    'refresh_timeout'
);

CREATE TYPE public.work_state AS ENUM ('pending', 'done', 'failed');

CREATE TYPE public.transcript_scope AS ENUM ('channel', 'user', 'cleared', 'selection');

CREATE TYPE public.permission_scope AS ENUM ('role', 'member', 'channel');

CREATE TYPE public.permission_effect AS ENUM ('allow', 'deny');

CREATE TYPE public.invocation_status AS ENUM ('running', 'complete', 'failed');

CREATE TYPE public.reference_origin AS ENUM ('live', 'archived');

CREATE TYPE public.message_removal AS ENUM ('manual', 'automod');

CREATE TABLE public.guild_settings (
    guild_id bigint PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE public.guild_log_channels (
    guild_id bigint NOT NULL REFERENCES public.guild_settings (guild_id) ON DELETE CASCADE,
    log_type text NOT NULL,
    channel_id bigint NOT NULL,
    PRIMARY KEY (guild_id, log_type)
);

CREATE TABLE public.guild_encryption (
    guild_id bigint PRIMARY KEY REFERENCES public.guild_settings (guild_id) ON DELETE CASCADE,
    enabled boolean NOT NULL DEFAULT false,
    key_channel_id bigint,
    key_message_id bigint
);

CREATE TABLE public.actions (
    id char(6) PRIMARY KEY,
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    moderator_id bigint NOT NULL,
    verb public.punishment_verb NOT NULL,
    state public.punishment_state NOT NULL DEFAULT 'active',
    reason text NOT NULL,
    note text,
    clear_days smallint NOT NULL DEFAULT 0 CHECK (clear_days BETWEEN 0 AND 7),
    target_present boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz
);

CREATE INDEX actions_history_idx ON public.actions (guild_id, user_id, created_at DESC);

CREATE INDEX actions_active_idx ON public.actions (guild_id, user_id)
WHERE
    state IN ('active', 'expiring');

CREATE INDEX actions_expiry_idx ON public.actions (expires_at)
WHERE
    state IN ('active', 'expiring');

CREATE TABLE public.action_refs (
    action_id char(6) PRIMARY KEY REFERENCES public.actions (id) ON DELETE CASCADE,
    origin public.reference_origin NOT NULL DEFAULT 'live',
    ref_message_id bigint,
    ref_channel_id bigint,
    ref_author_id bigint,
    ref_content bytea,
    image_url text
);

CREATE TABLE public.log_messages (
    message_id bigint PRIMARY KEY,
    guild_id bigint NOT NULL,
    channel_id bigint,
    target_id bigint NOT NULL,
    moderator_id bigint,
    action_id char(6) REFERENCES public.actions (id) ON DELETE CASCADE,
    content bytea,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX log_messages_action_idx ON public.log_messages (guild_id, action_id);

CREATE TABLE public.messages (
    message_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    guild_id bigint NOT NULL,
    author_id bigint NOT NULL,
    author_name text NOT NULL,
    author_display_name text,
    author_avatar_url text,
    referenced_message_id bigint,
    content bytea,
    embeds bytea,
    attachment_urls jsonb,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (message_id, created_at)
)
PARTITION BY
    RANGE (created_at);

CREATE INDEX messages_guild_idx ON public.messages (guild_id, created_at DESC);

CREATE INDEX messages_author_idx ON public.messages (guild_id, author_id, created_at DESC);

CREATE INDEX messages_channel_idx ON public.messages (channel_id, message_id);

CREATE TABLE public.messages_default PARTITION OF public.messages DEFAULT;

CREATE TABLE public.message_edits (
    edit_id bigserial NOT NULL,
    message_id bigint NOT NULL,
    content bytea,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (edit_id, created_at)
)
PARTITION BY
    RANGE (created_at);

CREATE INDEX message_edits_message_idx ON public.message_edits (message_id, created_at);

CREATE TABLE public.message_edits_default PARTITION OF public.message_edits DEFAULT;

DO $$
DECLARE
    month date := date_trunc('month', now() - interval '6 months')::date;
    stop date := date_trunc('month', now() + interval '24 months')::date;
BEGIN
    WHILE month < stop LOOP
        EXECUTE format(
            'CREATE TABLE public.messages_%s PARTITION OF public.messages FOR VALUES FROM (%L) TO (%L)',
            to_char(month, 'YYYYMM'),
            month,
            month + interval '1 month'
        );

        EXECUTE format(
            'CREATE TABLE public.message_edits_%s PARTITION OF public.message_edits FOR VALUES FROM (%L) TO (%L)',
            to_char(month, 'YYYYMM'),
            month,
            month + interval '1 month'
        );

        month := (month + interval '1 month')::date;
    END LOOP;
END $$;

CREATE TABLE public.message_deletions (
    message_id bigint PRIMARY KEY,
    guild_id bigint NOT NULL,
    source public.message_removal NOT NULL,
    rule text,
    deleted_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX message_deletions_guild_idx ON public.message_deletions (guild_id, deleted_at DESC);

CREATE TABLE public.transcripts (
    transcript_id text PRIMARY KEY,
    guild_id bigint NOT NULL,
    scope public.transcript_scope NOT NULL,
    channel_id bigint,
    channel_name text,
    subject_id bigint,
    subject_name text,
    window_start timestamptz,
    window_end timestamptz,
    moderator_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX transcripts_guild_idx ON public.transcripts (guild_id, created_at DESC);

CREATE TABLE public.transcript_messages (
    transcript_id text NOT NULL REFERENCES public.transcripts (transcript_id) ON DELETE CASCADE ON UPDATE CASCADE,
    message_id bigint NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY (transcript_id, message_id)
);

CREATE INDEX transcript_messages_pin_idx ON public.transcript_messages (message_id);

CREATE TABLE public.channel_message_budget (
    channel_id bigint PRIMARY KEY,
    message_count integer NOT NULL DEFAULT 0
);

CREATE UNLOGGED TABLE public.ocr_image_evaluations (
    image_hash char(64) NOT NULL,
    rule_hash char(64) NOT NULL,
    is_match boolean NOT NULL,
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (image_hash, rule_hash)
);

CREATE INDEX ocr_image_evaluations_eviction_idx ON public.ocr_image_evaluations (last_seen_at);

CREATE UNLOGGED TABLE public.command_traces (
    message_id bigint NOT NULL,
    command_name text NOT NULL,
    total_duration_nanos bigint NOT NULL,
    success boolean NOT NULL,
    failure text,
    points jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (message_id, created_at)
);

CREATE INDEX command_traces_recent_idx ON public.command_traces (created_at DESC);

CREATE INDEX command_traces_failure_idx ON public.command_traces (failure)
WHERE
    failure IS NOT NULL;

CREATE TABLE public.automod_rules (
    id char(6) PRIMARY KEY,
    guild_id bigint NOT NULL,
    name text NOT NULL,
    source text NOT NULL,
    compiled jsonb NOT NULL DEFAULT '{}'::jsonb,
    mode public.rule_mode NOT NULL DEFAULT 'disabled',
    rule_hash char(64) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX automod_rules_name_idx ON public.automod_rules (guild_id, lower(name));

CREATE INDEX automod_rules_enabled_idx ON public.automod_rules (guild_id)
WHERE
    mode <> 'disabled';

CREATE TABLE public.managed_rules (
    id char(7) PRIMARY KEY,
    name text NOT NULL,
    description text NOT NULL DEFAULT '',
    source text NOT NULL,
    compiled jsonb NOT NULL DEFAULT '{}'::jsonb,
    mode public.rule_mode NOT NULL DEFAULT 'disabled',
    rule_hash char(64) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX managed_rules_name_idx ON public.managed_rules (lower(name));

CREATE TABLE public.managed_rule_guilds (
    rule_id char(7) NOT NULL REFERENCES public.managed_rules (id) ON DELETE CASCADE,
    guild_id bigint NOT NULL,
    mode public.rule_mode NOT NULL DEFAULT 'disabled',
    response text NOT NULL DEFAULT 'then delete',
    compiled jsonb NOT NULL DEFAULT '{"outcome":{"verb":null,"duration":0,"clear_days":0,"delete":true,"notify":null,"reason":null}}'::jsonb,
    subscribed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (rule_id, guild_id)
);

CREATE INDEX managed_rule_guilds_guild_idx ON public.managed_rule_guilds (guild_id)
WHERE
    mode <> 'disabled';

CREATE TABLE public.sticky_messages (
    channel_id bigint PRIMARY KEY,
    guild_id bigint,
    content text NOT NULL,
    title text,
    color bigint,
    last_message_id bigint,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE public.scheduled_work (
    id bigserial PRIMARY KEY,
    kind public.work_kind NOT NULL,
    state public.work_state NOT NULL DEFAULT 'pending',
    action_id char(6) REFERENCES public.actions (id) ON DELETE CASCADE,
    guild_id bigint NOT NULL,
    user_id bigint,
    due_at timestamptz NOT NULL,
    attempts integer NOT NULL DEFAULT 0,
    last_error text,
    locked_until timestamptz NOT NULL DEFAULT to_timestamp(0),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX scheduled_work_due_idx ON public.scheduled_work (due_at)
WHERE
    state = 'pending';

CREATE UNIQUE INDEX scheduled_work_pending_idx ON public.scheduled_work (action_id, kind)
WHERE
    state = 'pending';

CREATE TABLE public.invocations (
    message_id bigint PRIMARY KEY,
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    author_id bigint NOT NULL,
    command text NOT NULL,
    args jsonb NOT NULL,
    action_id char(6),
    response_id bigint,
    status public.invocation_status NOT NULL DEFAULT 'running',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX invocations_expiry_idx ON public.invocations (created_at);

CREATE TABLE public.guild_permissions (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    scope public.permission_scope NOT NULL,
    subject_id bigint NOT NULL,
    target text NOT NULL,
    effect public.permission_effect NOT NULL,
    priority integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX guild_permissions_lookup_idx ON public.guild_permissions (guild_id, scope, subject_id);

CREATE TABLE public.command_snippets (
    id bigserial PRIMARY KEY,
    guild_id bigint,
    owner_id bigint,
    name text NOT NULL,
    body text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT command_snippets_one_home CHECK ((guild_id IS NULL) <> (owner_id IS NULL))
);

CREATE UNIQUE INDEX command_snippets_personal_idx ON public.command_snippets (owner_id, lower(name))
WHERE
    owner_id IS NOT NULL;

CREATE UNIQUE INDEX command_snippets_server_idx ON public.command_snippets (guild_id, lower(name))
WHERE
    guild_id IS NOT NULL;

CREATE TABLE public.dashboard_sessions (
    token text PRIMARY KEY,
    account_id bigint NOT NULL,
    name text NOT NULL,
    display text,
    avatar text,
    guilds jsonb NOT NULL,
    expires timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX dashboard_sessions_expires_idx ON public.dashboard_sessions (expires);

CREATE TABLE public.guild_errors (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    headline text NOT NULL,
    detail text,
    delivered boolean NOT NULL DEFAULT false,
    occurred_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX guild_errors_recent_idx ON public.guild_errors (guild_id, occurred_at DESC);

INSERT INTO
    public.guild_settings (guild_id)
SELECT DISTINCT
    guild_id
FROM
    public.legacy_guild_settings;

INSERT INTO
    public.guild_settings (guild_id)
SELECT DISTINCT
    guild_id
FROM
    public.legacy_actions
ON CONFLICT
DO NOTHING;

INSERT INTO
    public.guild_settings (guild_id)
SELECT DISTINCT
    guild_id
FROM
    public.legacy_guild_encryption
ON CONFLICT
DO NOTHING;

INSERT INTO
    public.guild_log_channels (guild_id, log_type, channel_id)
SELECT DISTINCT
    ON (settings.guild_id, entry.key) settings.guild_id,
    entry.key,
    (entry.value #>> '{}')::bigint
FROM
    public.legacy_guild_settings AS settings,
    LATERAL jsonb_each(COALESCE(settings.log_channel_ids, '{}'::jsonb)) AS entry
WHERE
    jsonb_typeof(entry.value) IN ('number', 'string')
    AND entry.key NOT IN ('action_update', 'avatar_update')
    AND EXISTS (
        SELECT
            1
        FROM
            public.guild_settings AS present
        WHERE
            present.guild_id = settings.guild_id
    );

INSERT INTO
    public.guild_encryption (guild_id, enabled, key_channel_id, key_message_id)
SELECT DISTINCT
    ON (legacy.guild_id) legacy.guild_id,
    legacy.encrypted,
    legacy.key_channel_id,
    legacy.key_message_id
FROM
    public.legacy_guild_encryption AS legacy
WHERE
    EXISTS (
        SELECT
            1
        FROM
            public.guild_settings AS present
        WHERE
            present.guild_id = legacy.guild_id
    );

INSERT INTO
    public.actions (
        id,
        guild_id,
        user_id,
        moderator_id,
        verb,
        state,
        reason,
        note,
        created_at,
        updated_at,
        expires_at
    )
SELECT DISTINCT
    ON (legacy.id) rpad(legacy.id, 6)::char(6),
    legacy.guild_id,
    CASE
        WHEN legacy.type::text = 'ban' THEN legacy.moderator_id
        ELSE legacy.user_id
    END,
    CASE
        WHEN legacy.type::text = 'ban' THEN legacy.user_id
        ELSE legacy.moderator_id
    END,
    (
        CASE legacy.type::text
            WHEN 'timeout' THEN 'mute'
            WHEN 'log' THEN 'warn'
            ELSE legacy.type::text
        END
    )::public.punishment_verb,
    (
        CASE
            WHEN NOT legacy.active THEN 'ended'
            WHEN legacy.expires_at AT TIME ZONE 'UTC' <= now() THEN 'ended'
            WHEN legacy.type::text NOT IN ('ban', 'mute', 'timeout') THEN 'ended'
            WHEN row_number() OVER (
                PARTITION BY
                    legacy.guild_id,
                    CASE
                        WHEN legacy.type::text = 'ban' THEN legacy.moderator_id
                        ELSE legacy.user_id
                    END,
                    CASE legacy.type::text
                        WHEN 'timeout' THEN 'mute'
                        ELSE legacy.type::text
                    END
                ORDER BY
                    legacy.created_at DESC,
                    legacy.id DESC
            ) > 1 THEN 'ended'
            ELSE 'active'
        END
    )::public.punishment_state,
    legacy.reason,
    legacy.note,
    legacy.created_at AT TIME ZONE 'UTC',
    COALESCE(legacy.updated_at, legacy.created_at) AT TIME ZONE 'UTC',
    legacy.expires_at AT TIME ZONE 'UTC'
FROM
    public.legacy_actions AS legacy
WHERE
    length(legacy.id) <= 6;

INSERT INTO
    public.action_refs (
        action_id,
        ref_message_id,
        ref_channel_id,
        ref_author_id,
        ref_content,
        image_url
    )
SELECT DISTINCT
    ON (legacy.action_id) rpad(legacy.action_id, 6)::char(6),
    legacy.ref_message_id,
    legacy.ref_channel_id,
    legacy.ref_author_id,
    legacy.ref_content,
    legacy.image_url
FROM
    public.legacy_action_refs AS legacy
WHERE
    EXISTS (
        SELECT
            1
        FROM
            public.actions AS present
        WHERE
            present.id = rpad(legacy.action_id, 6)::char(6)
    );

INSERT INTO
    public.log_messages (
        message_id,
        guild_id,
        channel_id,
        target_id,
        moderator_id,
        action_id,
        content
    )
SELECT DISTINCT
    ON (legacy.message_id) legacy.message_id,
    legacy.guild_id,
    (
        SELECT
            logged.channel_id
        FROM
            public.guild_log_channels AS logged
        WHERE
            logged.guild_id = legacy.guild_id
            AND logged.log_type = 'member_moderation'
            AND EXISTS (
                SELECT
                    1
                FROM
                    public.actions AS present
                WHERE
                    present.id = rpad(legacy.db_id, 6)::char(6)
            )
    ),
    legacy.target_id,
    NULLIF(legacy.moderator_id, legacy.target_id),
    (
        SELECT
            present.id
        FROM
            public.actions AS present
        WHERE
            present.id = rpad(legacy.db_id, 6)::char(6)
    ),
    legacy.content
FROM
    public.legacy_log_messages AS legacy;

INSERT INTO
    public.messages (
        message_id,
        channel_id,
        guild_id,
        author_id,
        author_name,
        author_display_name,
        author_avatar_url,
        content,
        embeds,
        attachment_urls,
        created_at
    )
SELECT DISTINCT
    ON (legacy.message_id) legacy.message_id,
    legacy.channel_id,
    legacy.guild_id,
    legacy.author_id,
    legacy.author_name,
    legacy.author_display_name,
    legacy.author_avatar_url,
    legacy.content,
    legacy.embeds,
    legacy.attachment_urls,
    legacy.created_at
FROM
    public.legacy_messages AS legacy;

INSERT INTO
    public.message_edits (message_id, content, created_at)
SELECT
    legacy.message_id,
    legacy.content,
    legacy.created_at
FROM
    public.legacy_message_edits AS legacy;

INSERT INTO
    public.transcripts (
        transcript_id,
        guild_id,
        scope,
        channel_id,
        channel_name,
        moderator_name,
        created_at
    )
SELECT DISTINCT
    ON (legacy.transcript_id) legacy.transcript_id,
    legacy.guild_id,
    'channel'::public.transcript_scope,
    legacy.channel_id,
    legacy.channel_name,
    legacy.moderator_name,
    legacy.created_at
FROM
    public.legacy_transcripts AS legacy;

INSERT INTO
    public.transcript_messages (transcript_id, message_id, created_at)
SELECT DISTINCT
    ON (saved.transcript_id, stored.message_id) saved.transcript_id,
    stored.message_id,
    stored.created_at
FROM
    public.legacy_transcripts AS saved,
    LATERAL jsonb_array_elements_text(saved.message_ids) AS listed (message_id)
    JOIN public.messages AS stored ON stored.message_id = listed.message_id::bigint;

UPDATE public.transcripts
SET
    transcript_id = gen_random_uuid()::text
WHERE
    length(transcript_id) <> 36;

INSERT INTO
    public.channel_message_budget (channel_id, message_count)
SELECT DISTINCT
    ON (channel_id) channel_id,
    message_count
FROM
    public.legacy_channel_budget;

INSERT INTO
    public.sticky_messages (
        channel_id,
        guild_id,
        content,
        title,
        color,
        last_message_id
    )
SELECT DISTINCT
    ON (channel_id) channel_id,
    NULL::bigint,
    content,
    title,
    color,
    last_message_id
FROM
    public.legacy_sticky_messages;

WITH
    rendered AS (
        SELECT DISTINCT
            ON (legacy.id) rpad(legacy.id, 6)::char(6) AS id,
            legacy.guild_id,
            legacy.name,
            concat_ws(
                E'\n',
                'on image',
                CASE
                    WHEN legacy.is_regex THEN 'match /' || legacy.rule || '/'
                    ELSE 'match "' || legacy.rule || '"'
                END,
                CASE
                    WHEN legacy.punishment_type::text = 'log' THEN 'then delete'
                    WHEN COALESCE(legacy.duration, 0) > 0 THEN 'then ' || legacy.punishment_type::text || ' ' || legacy.duration || 's'
                    ELSE 'then ' || legacy.punishment_type::text
                END,
                CASE
                    WHEN COALESCE(legacy.day_clear_amount, 0) > 0 THEN 'clear ' || legacy.day_clear_amount || ' days'
                END,
                'reason ' || legacy.reason,
                CASE
                    WHEN COALESCE(legacy.log_channel_id, 0) > 0 THEN 'notify <#' || legacy.log_channel_id || '>'
                END
            ) AS source,
            legacy.created_at AT TIME ZONE 'UTC' AS created_at,
            row_number() OVER (
                PARTITION BY
                    legacy.guild_id,
                    lower(legacy.name)
                ORDER BY
                    legacy.created_at,
                    legacy.id
            ) AS shared_name
        FROM
            public.legacy_automod_rules AS legacy
        WHERE
            length(legacy.id) <= 6
    )
INSERT INTO
    public.automod_rules (
        id,
        guild_id,
        name,
        source,
        mode,
        rule_hash,
        created_at,
        updated_at
    )
SELECT
    rendered.id,
    rendered.guild_id,
    CASE
        WHEN rendered.shared_name = 1 THEN rendered.name
        ELSE rendered.name || '-' || trim(rendered.id)
    END,
    rendered.source,
    'active'::public.rule_mode,
    encode(sha256(convert_to(rendered.source, 'UTF8')), 'hex')::char(64),
    rendered.created_at,
    rendered.created_at
FROM
    rendered;

INSERT INTO
    public.scheduled_work (kind, action_id, guild_id, user_id, due_at)
SELECT
    'lift_ban'::public.work_kind,
    live.id,
    live.guild_id,
    live.user_id,
    live.expires_at
FROM
    public.actions AS live
WHERE
    live.verb = 'ban'
    AND live.state IN ('active', 'expiring')
    AND live.expires_at IS NOT NULL;

INSERT INTO
    public.scheduled_work (kind, action_id, guild_id, user_id, due_at)
SELECT
    'lift_mute'::public.work_kind,
    live.id,
    live.guild_id,
    live.user_id,
    live.expires_at
FROM
    public.actions AS live
WHERE
    live.verb = 'mute'
    AND live.state IN ('active', 'expiring')
    AND live.expires_at IS NOT NULL;

INSERT INTO
    public.scheduled_work (kind, action_id, guild_id, user_id, due_at)
SELECT
    'refresh_timeout'::public.work_kind,
    live.id,
    live.guild_id,
    live.user_id,
    LEAST(
        COALESCE(live.expires_at, now() + interval '27 days'),
        now() + interval '27 days'
    ) - interval '1 hour'
FROM
    public.actions AS live
WHERE
    live.verb = 'mute'
    AND live.state IN ('active', 'expiring')
    AND (
        live.expires_at IS NULL
        OR live.expires_at > now() + interval '27 days'
    );

DROP TABLE IF EXISTS public.legacy_actions;
DROP TABLE IF EXISTS public.legacy_action_refs;
DROP TABLE IF EXISTS public.legacy_automod_rules;
DROP TABLE IF EXISTS public.legacy_guild_settings;
DROP TABLE IF EXISTS public.legacy_guild_encryption;
DROP TABLE IF EXISTS public.legacy_log_messages;
DROP TABLE IF EXISTS public.legacy_channel_budget;
DROP TABLE IF EXISTS public.legacy_message_edits;
DROP TABLE IF EXISTS public.legacy_messages;
DROP TABLE IF EXISTS public.legacy_sticky_messages;
DROP TABLE IF EXISTS public.legacy_transcripts;
DROP TABLE IF EXISTS public.user_flags;
DROP TABLE IF EXISTS public.discarded_channel_message_budget;
DROP TABLE IF EXISTS public.discarded_command_traces;
DROP TABLE IF EXISTS public.discarded_guild_log_channels;
DROP TABLE IF EXISTS public.discarded_guild_permissions;
DROP TABLE IF EXISTS public.discarded_invocations;
DROP TABLE IF EXISTS public.discarded_log_messages;
DROP TABLE IF EXISTS public.discarded_messages;
DROP TABLE IF EXISTS public.discarded_ocr_image_evaluations;
DROP TABLE IF EXISTS public.discarded_ocr_hashes;
DROP TABLE IF EXISTS public.discarded_scheduled_work;
DROP TABLE IF EXISTS public.discarded_transcript_messages;

ALTER INDEX IF EXISTS public.action_refs_pkey1 RENAME TO action_refs_pkey;
ALTER INDEX IF EXISTS public.automod_rules_pkey1 RENAME TO automod_rules_pkey;
ALTER INDEX IF EXISTS public.command_traces_pkey1 RENAME TO command_traces_pkey;
ALTER INDEX IF EXISTS public.guild_encryption_pkey1 RENAME TO guild_encryption_pkey;
ALTER INDEX IF EXISTS public.guild_settings_pkey1 RENAME TO guild_settings_pkey;
ALTER INDEX IF EXISTS public.message_edits_pkey1 RENAME TO message_edits_pkey;
ALTER INDEX IF EXISTS public.sticky_messages_pkey1 RENAME TO sticky_messages_pkey;
ALTER INDEX IF EXISTS public.transcripts_pkey1 RENAME TO transcripts_pkey;

DROP TYPE IF EXISTS public.action_type;
