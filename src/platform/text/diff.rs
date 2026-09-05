enum Line<'a> {
    Unchanged(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

pub fn create(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let mut table = vec![vec![0usize; new_lines.len() + 1]; old_lines.len() + 1];

    for oi in 0..old_lines.len() {
        for ni in 0..new_lines.len() {
            table[oi + 1][ni + 1] = match old_lines[oi] == new_lines[ni] {
                true => table[oi][ni] + 1,
                false => table[oi][ni + 1].max(table[oi + 1][ni]),
            };
        }
    }

    let mut oi = old_lines.len();
    let mut ni = new_lines.len();
    let mut pairs = Vec::new();

    while oi > 0 && ni > 0 {
        if old_lines[oi - 1] == new_lines[ni - 1] {
            pairs.push((oi - 1, ni - 1));
            oi -= 1;
            ni -= 1;

            continue;
        }

        if table[oi - 1][ni] > table[oi][ni - 1] {
            oi -= 1;
        } else {
            ni -= 1;
        }
    }

    pairs.reverse();

    let mut old_index = 0;
    let mut new_index = 0;
    let mut lines = Vec::new();

    for (oi, ni) in pairs {
        while old_index < oi {
            lines.push(Line::Removed(old_lines[old_index]));
            old_index += 1;
        }

        while new_index < ni {
            lines.push(Line::Added(new_lines[new_index]));
            new_index += 1;
        }

        lines.push(Line::Unchanged(old_lines[oi]));
        old_index += 1;
        new_index += 1;
    }

    while old_index < old_lines.len() {
        lines.push(Line::Removed(old_lines[old_index]));
        old_index += 1;
    }

    while new_index < new_lines.len() {
        lines.push(Line::Added(new_lines[new_index]));
        new_index += 1;
    }

    let mut out = String::new();

    for line in lines {
        let (marker, text) = match line {
            Line::Unchanged(text) => ("  ", text),
            Line::Added(text) => ("+ ", text),
            Line::Removed(text) => ("- ", text),
        };

        out.push_str(marker);
        out.push_str(text);
        out.push('\n');
    }

    out
}
