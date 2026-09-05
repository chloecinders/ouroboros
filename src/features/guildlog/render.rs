use crate::domain::Snowflake;
use crate::features::guildlog::attribution::Attribution;
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

pub fn channel_deleted(
    channel: Snowflake,
    name: Option<&str>,
    actor: Attribution,
    bot: Snowflake,
    transcript: Option<&str>,
) -> Embed {
    Embed::new("CHANNEL DELETED")
        .subtitle(match name {
            Some(name) => format!("Channel: #{name}"),
            None => format!("Channel: `{channel}`"),
        })
        .maybe_subtitle(actor.line(bot))
        .maybe_footnote(transcript.map(|link| format!("[View transcript]({link})")))
        .tone(Tone::Danger)
}
