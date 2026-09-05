use crate::platform::ui::embed::Embed;

#[derive(Clone, Copy, Debug, Default)]
pub struct Marks {
    pub silent: bool,
    pub dm_failed: bool,
    pub has_reference: bool,
    pub has_image: bool,
    pub edited: bool,
}

impl Marks {
    pub fn apply(&self, embed: Embed) -> Embed {
        let mut embed = match (self.silent, self.dm_failed) {
            (true, _) => embed.subtitle("silent"),
            (false, true) => embed.subtitle("DM failed"),
            (false, false) => embed,
        };

        embed = match (self.has_reference, self.has_image) {
            (true, true) => embed.subtitle("+ ref, + image"),
            (true, false) => embed.subtitle("+ ref"),
            (false, true) => embed.subtitle("+ image"),
            (false, false) => embed,
        };

        if self.edited {
            return embed.subtitle("edited");
        }

        embed
    }
}
