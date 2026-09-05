pub fn boundary(input: &str, chars: usize) -> Option<usize> {
    input.char_indices().nth(chars).map(|(index, _)| index)
}

pub fn cap(input: &str, max: usize) -> String {
    let Some(index) = boundary(input, max) else {
        return input.to_string();
    };

    let mut out = String::with_capacity(index + 3);
    out.push_str(&input[..index]);
    out.push_str("...");
    out
}

pub fn clamp(input: &str, max: usize) -> String {
    if boundary(input, max).is_none() {
        return input.to_string();
    }

    cap(input, max.saturating_sub(3))
}
