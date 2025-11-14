enum LineType {
    Unchanged(String),
    Added(String),
    Removed(String),
}

/// Creates a diff string from a (old) b (new):
/// + A
///   B
/// - C
/// + D
///   E
pub fn create_diff(a: String, b: String) -> String {
    // Myers algorithm adapted from stackoverflow
    let old_lines = a.lines().collect::<Vec<&str>>();
    let new_lines = b.lines().collect::<Vec<&str>>();

    let old_len = old_lines.len();
    let new_len = new_lines.len();
    let mut table = vec![vec![0; new_len + 1]; old_len + 1];

    for oi in 0..old_len {
        for ni in 0..new_len {
            table[oi + 1][ni + 1] = match old_lines[oi] == new_lines[ni] {
                true => table[oi][ni] + 1,
                false => match table[oi][ni + 1] >= table[oi + 1][ni] {
                    true => table[oi][ni + 1],
                    false => table[oi + 1][ni],
                },
            };
        }
    }

    let mut oi = old_len;
    let mut ni = new_len;
    let mut pairs = Vec::new();

    while oi > 0 && ni > 0 {
        if old_lines[oi - 1] == new_lines[ni - 1] {
            pairs.push((oi - 1, ni - 1));
            oi -= 1;
            ni -= 1;
        } else {
            if table[oi - 1][ni] > table[oi][ni - 1] {
                oi -= 1;
            } else {
                ni -= 1;
            }
        }
    }

    pairs.reverse();

    let mut old_index = 0;
    let mut new_index = 0;
    let mut result = Vec::new();

    for (oi, ni) in pairs {
        while old_index < oi {
            result.push(LineType::Removed(old_lines[old_index].to_string()));
            old_index += 1;
        }

        while new_index < ni {
            result.push(LineType::Added(new_lines[new_index].to_string()));
            new_index += 1;
        }

        result.push(LineType::Unchanged(old_lines[oi].to_string()));
        old_index += 1;
        new_index += 1;
    }

    while old_index < old_lines.len() {
        result.push(LineType::Removed(old_lines[old_index].to_string()));
        old_index += 1;
    }

    while new_index < new_lines.len() {
        result.push(LineType::Added(new_lines[new_index].to_string()));
        new_index += 1;
    }

    let mut final_string = String::new();

    for line in result.into_iter() {
        final_string.push_str(match line {
            LineType::Unchanged(line) => format!("  {line}\n"),
            LineType::Added(line) => format!("+ {line}\n"),
            LineType::Removed(line) => format!("- {line}\n"),
        }.as_str());
    }

    final_string
}
