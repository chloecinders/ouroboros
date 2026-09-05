use crate::features::references::Captured;
use crate::platform::ui::embed::{self, Embed, mention};
use crate::platform::ui::tone::Tone;

pub fn viewed(guild: u64, captured: &Captured) -> Embed {
    let link = match captured.origin.jumpable() {
        true => format!(
            "[jump]({})",
            embed::jump(guild, captured.channel, captured.message)
        ),
        false => String::from("(deleted)"),
    };

    Embed::new("REFERENCE")
        .subtitle(format!("Message ID: `{}` {link}", captured.message))
        .subtitle(format!("Author: {}", mention(captured.author)))
        .quote(
            captured
                .content
                .clone()
                .unwrap_or_else(|| String::from("no text was captured")),
        )
        .maybe_footnote(captured.image_url.clone())
        .tone(Tone::Info)
}
