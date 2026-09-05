use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

pub fn picker() -> Embed {
    Embed::new("DEFINE LOG")
        .body("Select the events that should be logged in the channel below.")
        .body(
            [
                (
                    "Keep",
                    "set this channel for events that don’t have a channel yet",
                ),
                (
                    "All",
                    "set this channel for all events, even if they already have one",
                ),
                ("Reset", "remove this channel from all events"),
            ]
            .iter()
            .map(|(label, desc)| format!("`{label}` - {desc}"))
            .collect::<Vec<String>>()
            .join("\n"),
        )
        .tone(Tone::Info)
}
