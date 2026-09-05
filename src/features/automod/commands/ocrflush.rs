use crate::command::cx::Cx;
use crate::command::error::Result;
use crate::command::{Command, Meta, Response};
use crate::platform::ocr;
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;
use aegis_macros::{command, meta};

#[command]
pub struct OcrFlush {}

impl Command for OcrFlush {
    const META: Meta = meta! {
        name: "ocrflush",
        aliases: ["ocr_flush"],
        short: "Drops all OCR hashes",
        full: "Drops all stored OCR hashes. Must be run after changes to the OCR image detection, as updates may change how images are processed.",
        category: Developer,
        developer: true,
        edit: Rerun,
    };

    async fn run(self, cx: &mut Cx) -> Result<Response> {
        cx.app.readings.forget_all();
        ocr::forget();

        Ok(Response::embed(Embed::new("DONE").tone(Tone::Success)))
    }
}
