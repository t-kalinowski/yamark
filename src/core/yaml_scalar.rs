use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

/// All callers share flow-scalar folding. Width planning continues to decline
/// escaped physical line continuations.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecodePolicy {
    Format,
    Width,
    Validation,
}

pub(crate) fn decode_quoted(raw: &str, policy: DecodePolicy) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    decode_quoted_chars(raw, policy, |ch| out.push(ch))?;
    Some(out)
}

pub(crate) fn decode_quoted_chars(
    raw: &str,
    policy: DecodePolicy,
    mut push: impl FnMut(char),
) -> Option<()> {
    let quote = match raw.as_bytes().first()? {
        b'\'' => '\'',
        b'"' => '"',
        _ => return None,
    };
    let inner = raw.strip_prefix(quote)?.strip_suffix(quote)?;
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if matches!(ch, ' ' | '\t' | '\r' | '\n') {
            let whitespace = std::iter::once(ch).chain(chars.clone());
            let line_break = if matches!(ch, '\r' | '\n') {
                Some(ch)
            } else {
                while chars.peek().is_some_and(|ch| matches!(ch, ' ' | '\t')) {
                    chars.next();
                }
                match chars.peek() {
                    Some('\r' | '\n') => chars.next(),
                    _ => None,
                }
            };
            if let Some(line_break) = line_break {
                let breaks = consume_flow_line_breaks(line_break, &mut chars);
                if breaks == 1 {
                    push(' ');
                } else {
                    for _ in 1..breaks {
                        push('\n');
                    }
                }
            } else {
                for ch in whitespace.take_while(|ch| matches!(ch, ' ' | '\t')) {
                    push(ch);
                }
            }
        } else if quote == '\'' && ch == '\'' {
            if chars.next()? != '\'' {
                return None;
            }
            push('\'');
        } else if quote == '"' && ch == '\\' {
            let escaped = chars.next()?;
            let decoded = match escaped {
                '0' => '\0',
                'a' => '\u{0007}',
                'b' => '\u{0008}',
                't' | '\t' => '\t',
                'n' => '\n',
                'v' => '\u{000b}',
                'f' => '\u{000c}',
                'r' => '\r',
                'e' => '\u{001b}',
                '"' => '"',
                '/' => '/',
                '\\' => '\\',
                'x' => decode_hex_escape(&mut chars, 2)?,
                'u' => decode_hex_escape(&mut chars, 4)?,
                'U' => decode_hex_escape(&mut chars, 8)?,
                '\r' | '\n' if policy != DecodePolicy::Width => {
                    for _ in 1..consume_flow_line_breaks(escaped, &mut chars) {
                        push('\n');
                    }
                    continue;
                }
                _ => return None,
            };
            push(decoded);
        } else {
            push(ch);
        }
    }
    Some(())
}

// Flow scalars discard physical whitespace around ordinary breaks, but escaped
// whitespace has already been emitted and remains significant.
fn consume_flow_line_breaks(first: char, chars: &mut Peekable<Chars<'_>>) -> usize {
    let mut line_break = first;
    let mut breaks = 0;
    loop {
        breaks += 1;
        if line_break == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
        }
        while chars.peek().is_some_and(|ch| matches!(ch, ' ' | '\t')) {
            chars.next();
        }
        match chars.peek() {
            Some('\r' | '\n') => {
                line_break = chars.next().expect("line break was just observed");
            }
            _ => return breaks,
        }
    }
}

pub(crate) fn decode_plain(raw: &str) -> Cow<'_, str> {
    if !raw.contains(['\r', '\n']) {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut blank_lines = 0usize;
    let mut has_content = false;
    let mut previous_cr = false;
    for line in raw.split_inclusive(['\r', '\n']) {
        // The two bytes of CRLF delimit one physical line.
        if previous_cr && line == "\n" {
            previous_cr = false;
            continue;
        }
        previous_cr = line.ends_with('\r');
        let line = line.trim_ascii();
        if line.is_empty() {
            if has_content {
                blank_lines += 1;
            }
            continue;
        }
        if has_content {
            if blank_lines == 0 {
                out.push(' ');
            } else {
                out.extend(std::iter::repeat_n('\n', blank_lines));
            }
        } else {
            has_content = true;
        }
        out.push_str(line);
        blank_lines = 0;
    }
    Cow::Owned(out)
}

fn decode_hex_escape(chars: &mut impl Iterator<Item = char>, digits: usize) -> Option<char> {
    let mut value = 0u32;
    for _ in 0..digits {
        value = value.checked_mul(16)?;
        value += chars.next()?.to_digit(16)?;
    }
    char::from_u32(value)
}
