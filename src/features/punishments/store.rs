use sqlx::PgPool;

use crate::command::error::{Ctx, Result};
use crate::domain::Snowflake;
use crate::domain::ids::ActionId;
use crate::domain::punishment::{Punishment, PunishmentState, PunishmentType};

pub async fn supersede(pool: &PgPool, punishment: &Punishment) -> Result<()> {
    if !(matches!(punishment.verb, PunishmentType::Ban | PunishmentType::Mute)) {
        return Ok(());
    }

    sqlx::query!(
        "UPDATE actions SET state = 'ended', updated_at = now()
        WHERE guild_id = $1 AND user_id = $2 AND verb = $3::punishment_verb
        AND state IN ('active', 'expiring')",
        punishment.guild as i64,
        punishment.target as i64,
        punishment.verb.as_str() as _
    )
    .execute(pool)
    .await
    .ctx("supersede active punishments")?;

    Ok(())
}

pub async fn insert(pool: &PgPool, punishment: &mut Punishment) -> Result<()> {
    const ATTEMPTS: u8 = 4;

    let initial_state = match matches!(punishment.verb, PunishmentType::Ban | PunishmentType::Mute)
    {
        true => PunishmentState::Active,
        false => PunishmentState::Ended,
    };

    let mut attempts = 1;

    let inserted = loop {
        let attempt = sqlx::query!(
            "INSERT INTO actions
        (id, guild_id, user_id, moderator_id, verb, state, reason, note, clear_days, expires_at)
        VALUES ($1, $2, $3, $4, $5::punishment_verb, $6::punishment_state, $7, $8, $9, $10)",
            punishment.id.as_str(),
            punishment.guild as i64,
            punishment.target as i64,
            punishment.actor as i64,
            punishment.verb.as_str() as _,
            initial_state.as_str() as _,
            punishment.reason.as_str(),
            punishment.note.as_ref().map(|note| note.as_str()),
            punishment.clear_days as i16,
            punishment.expires_at()
        )
        .execute(pool)
        .await;

        match attempt {
            Err(sqlx::Error::Database(ref taken))
                if attempts < ATTEMPTS && taken.constraint() == Some("actions_pkey") =>
            {
                punishment.id = ActionId::generate();
                attempts += 1;
            }
            outcome => break outcome,
        }
    };

    inserted.ctx("insert action")?;

    Ok(())
}

pub async fn withdraw(pool: &PgPool, id: &ActionId) -> Result<()> {
    sqlx::query!("DELETE FROM actions WHERE id = $1", id.as_str())
        .execute(pool)
        .await
        .ctx("withdraw unapplied action")?;

    Ok(())
}

pub async fn record_count(pool: &PgPool, guild: Snowflake, user: Snowflake) -> Result<i64> {
    let count = sqlx::query!(
        "SELECT count(*) AS total FROM actions WHERE guild_id = $1 AND user_id = $2",
        guild as i64,
        user as i64
    )
    .fetch_one(pool)
    .await
    .ctx("count prior actions")?;

    Ok(count.total.unwrap_or_default())
}

pub async fn set_presence(
    pool: &PgPool,
    guild: Snowflake,
    user: Snowflake,
    present: bool,
) -> Result<()> {
    sqlx::query!(
        "UPDATE actions SET target_present = $1, updated_at = now()
        WHERE guild_id = $2 AND user_id = $3 AND state IN ('active', 'expiring')",
        present,
        guild as i64,
        user as i64
    )
    .execute(pool)
    .await
    .ctx("record target presence")?;

    Ok(())
}

pub async fn mark_state(pool: &PgPool, id: &ActionId, state: PunishmentState) -> Result<()> {
    sqlx::query!(
        "UPDATE actions SET state = $1::punishment_state, updated_at = now() WHERE id = $2",
        state.as_str() as _,
        id.as_str()
    )
    .execute(pool)
    .await
    .ctx("mark punishment state")?;

    Ok(())
}

pub async fn guilds_with_active(pool: &PgPool) -> Result<Vec<Snowflake>> {
    let rows = sqlx::query_scalar!(
        "SELECT DISTINCT guild_id FROM actions WHERE state IN ('active', 'expiring')"
    )
    .fetch_all(pool)
    .await
    .ctx("list guilds with active punishments")?;

    Ok(rows.into_iter().map(|guild| guild as Snowflake).collect())
}
