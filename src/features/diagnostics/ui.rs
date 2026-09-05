use crate::features::diagnostics::store::Timing;
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

pub fn elapsed(nanos: i64) -> String {
    if nanos < 1_000 {
        return format!("{nanos}ns");
    }

    if nanos < 1_000_000 {
        return format!("{:.1}µs", nanos as f64 / 1_000.0);
    }

    format!("{:.1}ms", nanos as f64 / 1_000_000.0)
}

pub fn points(points: &serde_json::Value) -> String {
    let Some(listed) = points.as_array() else {
        return String::from("no points recorded");
    };

    let written: Vec<String> = listed
        .iter()
        .filter_map(|point| Some((point.get("name")?.as_str()?, point.get("nanos")?.as_i64()?)))
        .map(|(name, nanos)| format!("{name} `{}`", elapsed(nanos)))
        .collect();

    match written.is_empty() {
        true => String::from("no points recorded"),
        false => written.join("\n"),
    }
}

pub fn timing(run: &Timing) -> Embed {
    Embed::new("COMMAND TRACE")
        .subtitle(format!("Command: {}", run.command))
        .subtitle(format!("Elapsed: {}", elapsed(run.nanos)))
        .maybe_subtitle(run.failure.as_ref().map(|why| format!("Failure: {why}")))
        .body(points(&run.points))
        .tone(Tone::Info)
}
