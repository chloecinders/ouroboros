use serenity::all::Color;

const BRAND_RED: Color = Color::new(0xE35C68);
const BRAND_BLUE: Color = Color::new(0x04D9B2);
const SOFT_YELLOW: Color = Color::new(0xFFF3B0);
const SOFT_GREEN: Color = Color::new(0xA8D5BA);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Info,
    Danger,
    Warn,
    Success,
}

impl Tone {
    pub fn color(&self) -> Color {
        match self {
            Tone::Info => BRAND_BLUE,
            Tone::Danger => BRAND_RED,
            Tone::Warn => SOFT_YELLOW,
            Tone::Success => SOFT_GREEN,
        }
    }
}
