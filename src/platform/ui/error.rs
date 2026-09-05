use crate::command::error::{Audience, Error, Help, Mark};
use crate::platform::text::lexer::Span;
use crate::platform::ui::embed::Embed;
use crate::platform::ui::tone::Tone;

struct Shown {
    line: String,
    at: usize,
    width: usize,
}

fn clip(source: &str, at: usize, width: usize) -> Shown {
    let chars: Vec<char> = source.chars().collect();

    if chars.len() <= 96 {
        return Shown {
            line: source.to_string(),
            at,
            width,
        };
    }

    let slack = 96usize.saturating_sub(width) / 2;
    let from = at.saturating_sub(slack);
    let to = (at + width + slack).min(chars.len());
    let mut line = String::new();

    if from > 0 {
        line.push_str("...");
    }

    line.extend(&chars[from..to]);

    if to < chars.len() {
        line.push_str("...");
    }

    let lead = match from > 0 {
        true => 3,
        false => 0,
    };

    Shown {
        line,
        at: at - from + lead,
        width,
    }
}

fn under(shown: &Shown, marker: char) -> String {
    format!(
        "{}{}",
        " ".repeat(shown.at),
        String::from(marker).repeat(shown.width.max(1))
    )
}

struct Cut<'a> {
    line: &'a str,
    head: usize,
    tail: usize,
}

fn cut(source: &str, span: Span) -> Cut<'_> {
    let quotes = usize::from(span.quoted);
    let at = span.start.saturating_sub(quotes).min(source.len());
    let from = source[..at].rfind('\n').map_or(0, |break_at| break_at + 1);
    let line = match source[from..].find('\n') {
        Some(break_at) => &source[from..from + break_at],
        None => &source[from..],
    };

    Cut {
        line,
        head: at - from,
        tail: (span.end() + quotes).min(from + line.len()) - from,
    }
}

fn located(source: &str, mark: Mark) -> Shown {
    let Mark::At(span) = mark else {
        let whole = cut(source, Span::default());

        return clip(whole.line, 0, whole.line.chars().count());
    };

    let cut = cut(source, span);

    clip(
        cut.line,
        cut.line[..cut.head].chars().count(),
        cut.line[cut.head..cut.tail].chars().count(),
    )
}

fn suggested(source: &str, help: &Help) -> Shown {
    let cut = cut(source, help.span);
    let lead = &cut.line[..cut.head];
    let space = match lead.is_empty() || lead.ends_with(char::is_whitespace) || cut.head < cut.tail
    {
        true => "",
        false => " ",
    };
    let line = format!("{lead}{space}{}{}", help.with, &cut.line[cut.tail..]);

    clip(
        &line,
        lead.chars().count() + space.len(),
        help.with.chars().count(),
    )
}

pub fn render(error: &Error) -> Embed {
    let mut lines: Vec<String> = Vec::new();

    if let (Some(source), Some(label)) = (error.source(), error.label()) {
        let shown = located(source, label.mark);

        lines.push(shown.line.clone());
        lines.push(format!("{} {}", under(&shown, '^'), label.text));
    }

    if let (Some(source), Some(help)) = (error.source(), error.help()) {
        let shown = suggested(source, help);

        lines.push(format!("help: {}", help.text));
        lines.push(shown.line.clone());
        lines.push(under(&shown, '+'));
    }

    let hint = match error.audience() {
        Audience::User => error.hint(),
        Audience::Operator => Some("report this to the bot developers"),
    }
    .map(|hint| format!("hint: {hint}"));

    let embed = Embed::new(format!("error: {}", error.headline())).tone(Tone::Danger);

    if lines.is_empty() {
        return embed.maybe_footnote(hint);
    }

    lines.extend(hint);

    embed.quote(lines.join("\n"))
}
