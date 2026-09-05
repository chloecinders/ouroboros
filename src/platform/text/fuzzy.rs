use std::borrow::Cow;

fn lower(text: &str) -> Cow<'_, str> {
    if text.is_ascii() {
        return match text.bytes().any(|byte| byte.is_ascii_uppercase()) {
            true => Cow::Owned(text.to_ascii_lowercase()),
            false => Cow::Borrowed(text),
        };
    }

    Cow::Owned(text.to_lowercase())
}

#[derive(Clone, Debug, Default)]
pub struct Haystack<'a> {
    raw: &'a str,
    folded: Folded<'a>,
    dense: Option<Dense>,
}

#[derive(Clone, Debug, Default)]
struct Folded<'a> {
    lowered: Cow<'a, str>,
    chars: Vec<char>,
}

#[derive(Clone, Debug)]
struct Dense {
    raw: String,
    lowered: String,
    chars: Vec<char>,
}

impl<'a> Haystack<'a> {
    pub fn new(raw: &'a str) -> Self {
        let lowered = lower(raw);
        let chars = lowered.chars().collect();

        let dense = match raw.chars().any(char::is_whitespace) {
            false => None,
            true => {
                let stripped: String = raw.chars().filter(|ch| !ch.is_whitespace()).collect();
                let lowered = lower(&stripped).into_owned();

                Some(Dense {
                    chars: lowered.chars().collect(),
                    raw: stripped,
                    lowered,
                })
            }
        };

        Self {
            raw,
            folded: Folded { lowered, chars },
            dense,
        }
    }

    pub fn text(&self) -> &'a str {
        self.raw
    }

    fn squeezed(&self) -> (&str, &str, &[char]) {
        match &self.dense {
            Some(dense) => (&dense.raw, &dense.lowered, &dense.chars),
            None => (self.raw, &self.folded.lowered, &self.folded.chars),
        }
    }
}

fn near(needle: &str, raw: &str, lowered: &str, chars: &[char], threshold: f64) -> bool {
    if raw.contains(needle) {
        return true;
    }

    let wanted = lower(needle);

    if lowered.contains(wanted.as_ref()) {
        return true;
    }

    let wanted: Vec<char> = wanted.chars().collect();
    let budget = ((1.0 - threshold) * wanted.len() as f64).ceil() as usize;

    distance(&wanted, chars).is_some_and(|distance| distance <= budget)
}

pub fn contains_loose(needle: &str, haystack: &Haystack, threshold: f64) -> bool {
    if near(
        needle,
        haystack.raw,
        &haystack.folded.lowered,
        &haystack.folded.chars,
        threshold,
    ) {
        return true;
    }

    let (raw, lowered, chars) = haystack.squeezed();

    let squeezed = match needle.chars().any(char::is_whitespace) {
        true => Cow::Owned(needle.chars().filter(|ch| !ch.is_whitespace()).collect()),
        false => Cow::Borrowed(needle),
    };

    near(&squeezed, raw, lowered, chars, threshold)
}

fn distance(needle: &[char], haystack: &[char]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    if haystack.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    let mut row = vec![0usize; haystack.len() + 1];

    for (down, want) in needle.iter().enumerate() {
        let mut diagonal = row[0];

        row[0] = down + 1;

        for (across, have) in haystack.iter().enumerate() {
            let above = row[across + 1];
            let cost = usize::from(want != have);

            row[across + 1] = (above + 1).min(row[across] + 1).min(diagonal + cost);
            diagonal = above;
        }
    }

    row.iter().skip(1).min().copied()
}
