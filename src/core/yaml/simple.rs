//! Standalone line and flow-table formatting helpers.

use super::syntax::find_trailing_comment;

pub fn format_simple_yaml_line(line: &str) -> String {
    let (body, newline) = strip_newline(line);
    let bom_len = if body.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let indent_len = bom_len
        + body[bom_len..]
            .bytes()
            .take_while(|byte| *byte == b' ')
            .count();
    let indent = &body[..indent_len];
    let trimmed = body[indent_len..].trim_ascii_end();

    if let Some(rest) = trimmed.strip_prefix("-") {
        if rest.is_empty() {
            return format!("{indent}-{newline}");
        }
        if rest.chars().next().is_some_and(char::is_whitespace) {
            return format!(
                "{indent}- {}{newline}",
                normalize_flow_value(rest.trim_ascii_start())
            );
        }
    }

    if let Some(colon) = simple_mapping_colon(trimmed) {
        let key = trimmed[..colon].trim_ascii_end();
        let value = normalize_flow_value(trimmed[colon + 1..].trim_ascii_start());
        if value.is_empty() {
            format!("{indent}{key}:{newline}")
        } else {
            format!("{indent}{key}: {value}{newline}")
        }
    } else {
        format!("{indent}{trimmed}{newline}")
    }
}

pub fn format_flow_table(source: &str) -> String {
    let mut rows = Vec::new();
    for line in source.split_inclusive('\n') {
        let (body, newline) = strip_newline(line);
        let indent_len = body.bytes().take_while(|byte| *byte == b' ').count();
        let indent = &body[..indent_len];
        let trimmed = body[indent_len..].trim_ascii();
        let Some(fields) = parse_flow_mapping_row(trimmed) else {
            return source.to_owned();
        };
        rows.push((indent.to_owned(), fields, newline.to_owned()));
    }
    if rows.iter().any(|(_, fields, _)| fields.is_empty()) {
        return source.to_owned();
    }
    let first_width = rows
        .iter()
        .map(|(_, fields, _)| {
            fields
                .first()
                .map(|(key, value)| format!("{key}: {value}").chars().count())
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for (indent, fields, newline) in rows {
        out.push_str(&indent);
        out.push_str("- {");
        for (index, (key, value)) in fields.iter().enumerate() {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            if index + 1 < fields.len() {
                out.push(',');
                if index == 0 {
                    let rendered = key.chars().count() + 2 + value.chars().count();
                    out.push_str(&" ".repeat(first_width.saturating_sub(rendered) + 1));
                } else {
                    out.push(' ');
                }
            }
        }
        out.push('}');
        out.push_str(&newline);
    }
    out
}

pub(super) fn simple_mapping_colon(text: &str) -> Option<usize> {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut flow_depth = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if in_double {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_double = false,
                _ => {}
            }
            continue;
        }
        if in_single {
            if ch == '\'' {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    in_single = false;
                }
            }
            continue;
        }
        match ch {
            '\'' => in_single = true,
            '"' => in_double = true,
            '[' | '{' => flow_depth += 1,
            ']' | '}' => flow_depth = flow_depth.saturating_sub(1),
            ':' if flow_depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

pub(super) fn strip_newline(line: &str) -> (&str, &str) {
    if let Some(line) = line.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = line.strip_suffix('\n') {
        (line, "\n")
    } else if let Some(line) = line.strip_suffix('\r') {
        (line, "\r")
    } else {
        (line, "")
    }
}

fn parse_flow_mapping_row(trimmed: &str) -> Option<Vec<(String, String)>> {
    let inner = trimmed.strip_prefix("- {")?.strip_suffix('}')?;
    let mut fields = Vec::new();
    for part in inner.split(',') {
        let (key, value) = part.split_once(':')?;
        let key = key.trim_ascii();
        let value = value.trim_ascii();
        if key.is_empty() || value.is_empty() {
            return None;
        }
        fields.push((key.to_owned(), value.to_owned()));
    }
    Some(fields)
}

pub(super) fn format_markdown_yaml_block_value(value: &str) -> String {
    replace_folded_block_marker(value.trim_ascii())
}

fn replace_folded_block_marker(source: &str) -> String {
    let (body, newline) = strip_newline(source);
    let comment_start = find_trailing_comment(body, 0).unwrap_or(body.len());
    let Some(marker) = body[..comment_start].rfind('>') else {
        return source.to_owned();
    };
    let mut out = String::with_capacity(source.len());
    out.push_str(&body[..marker]);
    out.push('|');
    out.push_str(&body[marker + 1..]);
    out.push_str(newline);
    out
}

fn normalize_flow_value(value: &str) -> String {
    let trimmed = value.trim_ascii();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let values = inner
            .split(',')
            .map(|item| item.trim_ascii())
            .collect::<Vec<_>>();
        return format!("[{}]", values.join(", "));
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let values = inner
            .split(',')
            .map(|item| {
                if let Some((key, value)) = item.split_once(':') {
                    format!("{}: {}", key.trim_ascii(), value.trim_ascii())
                } else {
                    item.trim_ascii().to_owned()
                }
            })
            .collect::<Vec<_>>();
        return format!("{{{}}}", values.join(", "));
    }
    trimmed.to_owned()
}
