/// Preserve each caller's supported multiline subset. The formatter retains
/// literal single-quoted line breaks; semantic validation leaves their folding
/// opaque. Width planning declines escaped continuations.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecodePolicy {
    Format,
    Width,
    Validation,
}

pub(crate) fn decode_quoted(raw: &str, policy: DecodePolicy) -> Option<String> {
    let mut out = String::new();
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
        if quote == '\'' && ch == '\'' {
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
                    if escaped == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    while chars.peek().is_some_and(|ch| matches!(ch, ' ' | '\t')) {
                        chars.next();
                    }
                    continue;
                }
                _ => return None,
            };
            push(decoded);
        } else {
            if matches!(ch, '\r' | '\n') && (quote == '"' || policy == DecodePolicy::Validation) {
                return None;
            }
            push(ch);
        }
    }
    Some(())
}

fn decode_hex_escape(chars: &mut impl Iterator<Item = char>, digits: usize) -> Option<char> {
    let mut value = 0u32;
    for _ in 0..digits {
        value = value.checked_mul(16)?;
        value += chars.next()?.to_digit(16)?;
    }
    char::from_u32(value)
}
