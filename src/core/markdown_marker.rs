pub(crate) fn markdown_list_marker_len(trimmed: &str) -> Option<usize> {
    let bytes = trimmed.as_bytes();
    if bytes.len() == 1 && matches!(bytes[0], b'-' | b'*' | b'+') {
        return Some(1);
    }
    if bytes.len() >= 2 && matches!(bytes[0], b'-' | b'*' | b'+') && bytes[1].is_ascii_whitespace()
    {
        return Some(1);
    }

    let digits = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits > 0
        && bytes
            .get(digits)
            .is_some_and(|byte| matches!(*byte, b'.' | b')'))
        && bytes.get(digits + 1).is_some_and(u8::is_ascii_whitespace)
    {
        return Some(digits + 1);
    }

    if let Some(marker_len) = parenthesized_ordered_marker_len(trimmed) {
        return Some(marker_len);
    }

    let alpha = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_alphabetic())
        .count();
    if alpha > 0
        && alpha <= 8
        && ordered_alpha_marker(&trimmed[..alpha])
        && bytes
            .get(alpha)
            .is_some_and(|byte| matches!(*byte, b'.' | b')'))
        && bytes.get(alpha + 1).is_some_and(u8::is_ascii_whitespace)
    {
        return Some(alpha + 1);
    }

    None
}

pub(crate) fn markdown_list_marker(trimmed: &str) -> Option<(&str, &str)> {
    let marker_len = markdown_list_marker_len(trimmed)?;
    Some((&trimmed[..marker_len], &trimmed[marker_len..]))
}

fn parenthesized_ordered_marker_len(trimmed: &str) -> Option<usize> {
    let bytes = trimmed.as_bytes();
    if bytes.first() != Some(&b'(') {
        return None;
    }
    let inner = bytes[1..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric())
        .count();
    if inner == 0 || inner > 8 {
        return None;
    }
    let close = 1 + inner;
    let marker = &trimmed[1..close];
    if ordered_parenthesized_marker(marker)
        && bytes.get(close) == Some(&b')')
        && bytes.get(close + 1).is_some_and(u8::is_ascii_whitespace)
    {
        Some(close + 1)
    } else {
        None
    }
}

fn ordered_parenthesized_marker(marker: &str) -> bool {
    marker.bytes().all(|byte| byte.is_ascii_digit()) || ordered_alpha_marker(marker)
}

fn ordered_alpha_marker(marker: &str) -> bool {
    marker.len() == 1 && marker.as_bytes()[0].is_ascii_alphabetic() || valid_roman_marker(marker)
}

fn valid_roman_marker(marker: &str) -> bool {
    if marker.is_empty()
        || marker.len() > 8
        || !marker.bytes().all(|byte| {
            matches!(
                byte.to_ascii_uppercase(),
                b'I' | b'V' | b'X' | b'L' | b'C' | b'D' | b'M'
            )
        })
    {
        return false;
    }

    let mut total = 0usize;
    let mut previous = 0usize;
    for byte in marker.bytes().rev() {
        let value = roman_value(byte.to_ascii_uppercase());
        if value < previous {
            total = total.saturating_sub(value);
        } else {
            total += value;
            previous = value;
        }
    }
    total > 0 && roman_numeral(total) == marker.to_ascii_uppercase()
}

fn roman_value(byte: u8) -> usize {
    match byte {
        b'I' => 1,
        b'V' => 5,
        b'X' => 10,
        b'L' => 50,
        b'C' => 100,
        b'D' => 500,
        b'M' => 1000,
        _ => 0,
    }
}

fn roman_numeral(mut value: usize) -> String {
    let mut out = String::new();
    for (amount, numeral) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= amount {
            out.push_str(numeral);
            value -= amount;
        }
    }
    out
}
