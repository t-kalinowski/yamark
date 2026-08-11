use std::borrow::Cow;
use std::path::Path;

use jsonc_parser::ast::{Comment, ObjectPropName, Value};
use jsonc_parser::common::Ranged;
use jsonc_parser::{
    CollectOptions, CommentCollectionStrategy, CommentMap, ParseOptions, parse_to_ast,
};

use crate::diagnostic::{Result, YamarkError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonSourceKind {
    Json,
    JsonLines,
    Jsonc,
    Json5,
}

impl JsonSourceKind {
    pub(crate) fn for_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "json" => Some(Self::Json),
            "jsonl" | "ndjson" => Some(Self::JsonLines),
            "jsonc" => Some(Self::Jsonc),
            "json5" => Some(Self::Json5),
            _ => None,
        }
    }
}

pub(crate) fn json_to_yaml_source(input: &str, kind: JsonSourceKind) -> Result<String> {
    match kind {
        JsonSourceKind::Json => render_json(input),
        JsonSourceKind::JsonLines => render_json_lines(input),
        JsonSourceKind::Jsonc => render_jsonc(input),
        JsonSourceKind::Json5 => crate::json5_to_yaml::json5_to_yaml_source(input),
    }
}

fn render_jsonc(input: &str) -> Result<String> {
    let normalized = normalize_jsonc_comment_line_breaks(input);
    check_json_nesting(&normalized, "JSONC", 0)?;
    let parsed = parse_to_ast(
        &normalized,
        &jsonc_collect_options(),
        &jsonc_parse_options(),
    )
    .map_err(|err| {
        YamarkError::at(
            format!("invalid JSONC: {}", err.kind()),
            err.line_display(),
            err.column_display(),
        )
    })?;
    let value = parsed
        .value
        .ok_or_else(|| YamarkError::new("invalid JSONC: expected a JSON value"))?;
    let comments = parsed
        .comments
        .expect("JSONC parsing requested separate comments");
    let mut output = String::with_capacity(input.len().saturating_add(1));
    if comments.is_empty() {
        emit_json_value(&mut output, &value);
        output.push('\n');
        return Ok(output);
    }
    let mut comments = JsoncCommentEmitter::new(&comments);
    comments.emit_at(&mut output, value.range().start, 0);
    emit_jsonc_block_value(&mut output, &value, 0, &mut comments);
    comments.emit_at(&mut output, value.range().end, 0);
    Ok(output)
}

#[derive(Clone, Copy)]
enum JsonScanState {
    Outside,
    String,
    LineComment,
    BlockComment,
}

fn normalize_jsonc_comment_line_breaks(input: &str) -> Cow<'_, str> {
    if memchr::memchr2(b'\r', 0xe2, input.as_bytes()).is_none() {
        return Cow::Borrowed(input);
    }
    let mut normalized = None;
    let mut copied_until = 0;
    let mut state = JsonScanState::Outside;
    let mut offset = 0;
    while offset < input.len() {
        let rest = &input[offset..];
        match state {
            JsonScanState::Outside if rest.starts_with("//") => {
                offset += 2;
                state = JsonScanState::LineComment;
                continue;
            }
            JsonScanState::Outside if rest.starts_with("/*") => {
                offset += 2;
                state = JsonScanState::BlockComment;
                continue;
            }
            JsonScanState::BlockComment if rest.starts_with("*/") => {
                offset += 2;
                state = JsonScanState::Outside;
                continue;
            }
            _ => {}
        }

        let character = rest.chars().next().expect("offset is before input end");
        let next_offset = offset + character.len_utf8();
        match state {
            JsonScanState::Outside => {
                state = if character == '"' {
                    JsonScanState::String
                } else {
                    JsonScanState::Outside
                };
                offset = next_offset;
            }
            JsonScanState::String => {
                if character == '\\' {
                    offset = next_offset;
                    if offset < input.len() {
                        offset += input[offset..]
                            .chars()
                            .next()
                            .expect("offset is before input end")
                            .len_utf8();
                    }
                } else {
                    offset = next_offset;
                    if character == '"' {
                        state = JsonScanState::Outside;
                    }
                }
            }
            JsonScanState::LineComment => {
                if character == '\r' {
                    state = JsonScanState::Outside;
                    if !input[next_offset..].starts_with('\n') {
                        replace_jsonc_comment_line_break(
                            &mut normalized,
                            input,
                            &mut copied_until,
                            offset,
                            next_offset,
                        );
                    }
                } else if character == '\n' {
                    state = JsonScanState::Outside;
                } else if matches!(character, '\u{2028}' | '\u{2029}') {
                    replace_jsonc_comment_line_break(
                        &mut normalized,
                        input,
                        &mut copied_until,
                        offset,
                        next_offset,
                    );
                    state = JsonScanState::Outside;
                }
                offset = next_offset;
            }
            JsonScanState::BlockComment => {
                if (character == '\r' && !input[next_offset..].starts_with('\n'))
                    || matches!(character, '\u{2028}' | '\u{2029}')
                {
                    replace_jsonc_comment_line_break(
                        &mut normalized,
                        input,
                        &mut copied_until,
                        offset,
                        next_offset,
                    );
                }
                offset = next_offset;
            }
        }
    }

    let Some(mut normalized) = normalized else {
        return Cow::Borrowed(input);
    };
    normalized.push_str(&input[copied_until..]);
    Cow::Owned(normalized)
}

fn replace_jsonc_comment_line_break(
    normalized: &mut Option<String>,
    input: &str,
    copied_until: &mut usize,
    start: usize,
    end: usize,
) {
    let output = normalized.get_or_insert_with(|| String::with_capacity(input.len()));
    output.push_str(&input[*copied_until..start]);
    output.push('\n');
    *copied_until = end;
}

fn render_json(input: &str) -> Result<String> {
    let value = parse_json_value(input, 0)?;
    let mut output = String::with_capacity(input.len().saturating_add(1));
    emit_json_value(&mut output, &value);
    output.push('\n');
    Ok(output)
}

fn render_json_lines(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(YamarkError::new(
            "invalid JSON Lines: expected a JSON value",
        ));
    }

    let mut output = String::with_capacity(input.len().saturating_add(4));
    for (line_index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            return Err(YamarkError::at(
                "invalid JSON Lines: records cannot be blank",
                line_index + 1,
                1,
            ));
        }
        let value = parse_json_value(line, line_index)?;
        output.push_str("---\n");
        emit_json_value(&mut output, &value);
        output.push('\n');
    }
    Ok(output)
}

fn parse_json_value(input: &str, line_offset: usize) -> Result<Value<'_>> {
    check_json_nesting(input, "JSON", line_offset)?;
    let parsed =
        parse_to_ast(input, &strict_collect_options(), &strict_parse_options()).map_err(|err| {
            YamarkError::at(
                format!("invalid JSON: {}", err.kind()),
                err.line_display() + line_offset,
                err.column_display(),
            )
        })?;
    parsed
        .value
        .ok_or_else(|| YamarkError::at("invalid JSON: expected a JSON value", line_offset + 1, 1))
}

fn check_json_nesting(input: &str, dialect: &str, line_offset: usize) -> Result<()> {
    const MAX_NESTING: usize = 256;

    let bytes = input.as_bytes();
    let mut state = JsonScanState::Outside;
    let mut nesting = 0usize;
    let mut offset = 0usize;
    while offset < bytes.len() {
        match state {
            JsonScanState::Outside => match bytes[offset] {
                b'"' => {
                    state = JsonScanState::String;
                    offset += 1;
                }
                b'/' if bytes.get(offset + 1) == Some(&b'/') => {
                    state = JsonScanState::LineComment;
                    offset += 2;
                }
                b'/' if bytes.get(offset + 1) == Some(&b'*') => {
                    state = JsonScanState::BlockComment;
                    offset += 2;
                }
                b'[' | b'{' => {
                    nesting += 1;
                    if nesting > MAX_NESTING {
                        let (line, column) = line_column(input, offset);
                        return Err(YamarkError::at(
                            format!("invalid {dialect}: nesting exceeds {MAX_NESTING} levels"),
                            line + line_offset,
                            column,
                        ));
                    }
                    offset += 1;
                }
                b']' | b'}' => {
                    nesting = nesting.saturating_sub(1);
                    offset += 1;
                }
                _ => offset += 1,
            },
            JsonScanState::String => match bytes[offset] {
                b'\\' => offset = (offset + 2).min(bytes.len()),
                b'"' => {
                    state = JsonScanState::Outside;
                    offset += 1;
                }
                _ => offset += 1,
            },
            JsonScanState::LineComment => {
                if matches!(bytes[offset], b'\r' | b'\n') {
                    state = JsonScanState::Outside;
                }
                offset += 1;
            }
            JsonScanState::BlockComment => {
                if bytes[offset] == b'*' && bytes.get(offset + 1) == Some(&b'/') {
                    state = JsonScanState::Outside;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    let mut previous_was_cr = false;
    for (byte_offset, character) in source.char_indices() {
        if byte_offset >= offset {
            break;
        }
        match character {
            '\r' => {
                line += 1;
                column = 1;
                previous_was_cr = true;
            }
            '\n' if previous_was_cr => {
                previous_was_cr = false;
            }
            '\n' | '\u{2028}' | '\u{2029}' => {
                line += 1;
                column = 1;
                previous_was_cr = false;
            }
            _ => {
                column += 1;
                previous_was_cr = false;
            }
        }
    }
    (line, column)
}

fn strict_collect_options() -> CollectOptions {
    CollectOptions {
        comments: CommentCollectionStrategy::Off,
        tokens: false,
    }
}

fn jsonc_collect_options() -> CollectOptions {
    CollectOptions {
        comments: CommentCollectionStrategy::Separate,
        tokens: false,
    }
}

fn strict_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: false,
        allow_loose_object_property_names: false,
        allow_trailing_commas: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn jsonc_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

fn emit_json_value(output: &mut String, value: &Value<'_>) {
    match value {
        Value::StringLit(value) => emit_yaml_double_quoted(output, value.value.as_ref()),
        Value::NumberLit(value) => output.push_str(value.value),
        Value::BooleanLit(value) => output.push_str(if value.value { "true" } else { "false" }),
        Value::NullKeyword(_) => output.push_str("null"),
        Value::Array(array) => {
            output.push('[');
            for (index, value) in array.elements.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                emit_json_value(output, value);
            }
            output.push(']');
        }
        Value::Object(object) => {
            output.push('{');
            for (index, property) in object.properties.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                let key = match &property.name {
                    ObjectPropName::String(value) => value.value.as_ref(),
                    ObjectPropName::Word(value) => value.value,
                };
                emit_yaml_double_quoted(output, key);
                output.push(':');
                emit_json_value(output, &property.value);
            }
            output.push('}');
        }
    }
}

fn emit_jsonc_block_value(
    output: &mut String,
    value: &Value<'_>,
    indent: usize,
    comments: &mut JsoncCommentEmitter<'_, '_>,
) {
    match value {
        Value::Array(array) => {
            comments.emit_at(output, array.range.start + 1, indent);
            if array.elements.is_empty() {
                push_yaml_indent(output, indent);
                output.push_str("[]\n");
            }
            for value in &array.elements {
                let range = value.range();
                comments.emit_at(output, range.start, indent);
                comments.emit_empty_container_comments(output, value, indent);
                push_yaml_indent(output, indent);
                if json_value_is_inline(value) {
                    output.push_str("- ");
                    emit_json_value(output, value);
                    output.push('\n');
                } else {
                    output.push_str("-\n");
                    emit_jsonc_block_value(output, value, indent + 2, comments);
                }
                comments.emit_at(output, range.end, indent);
            }
            comments.emit_at(output, array.range.end - 1, indent);
        }
        Value::Object(object) => {
            comments.emit_at(output, object.range.start + 1, indent);
            if object.properties.is_empty() {
                push_yaml_indent(output, indent);
                output.push_str("{}\n");
            }
            for property in &object.properties {
                let key_range = property.name.range();
                let value_range = property.value.range();
                comments.emit_at(output, key_range.start, indent);
                comments.emit_at(output, key_range.end, indent);
                comments.emit_at(output, value_range.start, indent);
                comments.emit_empty_container_comments(output, &property.value, indent);
                push_yaml_indent(output, indent);
                let key = match &property.name {
                    ObjectPropName::String(value) => value.value.as_ref(),
                    ObjectPropName::Word(value) => value.value,
                };
                emit_yaml_mapping_key(output, key);
                if json_value_is_inline(&property.value) {
                    output.push_str(": ");
                    emit_json_value(output, &property.value);
                    output.push('\n');
                } else {
                    output.push_str(":\n");
                    emit_jsonc_block_value(output, &property.value, indent + 2, comments);
                }
                comments.emit_at(output, value_range.end, indent);
            }
            comments.emit_at(output, object.range.end - 1, indent);
        }
        _ => {
            push_yaml_indent(output, indent);
            emit_json_value(output, value);
            output.push('\n');
        }
    }
}

fn emit_yaml_mapping_key(output: &mut String, value: &str) {
    let mut characters = value.chars();
    let starts_plain = characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_');
    let continues_plain = characters
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'));
    let reserved = value.eq_ignore_ascii_case("null")
        || value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false");
    if starts_plain && continues_plain && !reserved {
        output.push_str(value);
    } else {
        emit_yaml_double_quoted(output, value);
    }
}

fn json_value_is_inline(value: &Value<'_>) -> bool {
    match value {
        Value::Array(array) => array.elements.is_empty(),
        Value::Object(object) => object.properties.is_empty(),
        Value::StringLit(_)
        | Value::NumberLit(_)
        | Value::BooleanLit(_)
        | Value::NullKeyword(_) => true,
    }
}

fn push_yaml_indent(output: &mut String, indent: usize) {
    output.extend(std::iter::repeat_n(' ', indent));
}

struct JsoncCommentEmitter<'map, 'input> {
    comments: &'map CommentMap<'input>,
    emitted_through: usize,
}

impl<'map, 'input> JsoncCommentEmitter<'map, 'input> {
    fn new(comments: &'map CommentMap<'input>) -> Self {
        Self {
            comments,
            emitted_through: 0,
        }
    }

    fn emit_at(&mut self, output: &mut String, position: usize, indent: usize) {
        if self.comments.is_empty() {
            return;
        }
        let Some(comments) = self.comments.get(&position) else {
            return;
        };
        for comment in comments.iter() {
            let range = comment.range();
            if range.end <= self.emitted_through {
                continue;
            }
            self.emitted_through = range.end;
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            match comment {
                Comment::Line(comment) => {
                    emit_yaml_comment_line(output, comment.text, indent, "jsonc")
                }
                Comment::Block(comment) => {
                    for_each_comment_line(comment.text, |line| {
                        let line = trim_comment_line(line);
                        emit_yaml_comment_line(output, line, indent, "jsonc");
                    });
                }
            }
        }
    }

    fn emit_empty_container_comments(
        &mut self,
        output: &mut String,
        value: &Value<'_>,
        indent: usize,
    ) {
        let range = match value {
            Value::Array(array) if array.elements.is_empty() => array.range,
            Value::Object(object) if object.properties.is_empty() => object.range,
            _ => return,
        };
        self.emit_at(output, range.start + 1, indent);
        self.emit_at(output, range.end.saturating_sub(1), indent);
    }
}

pub(crate) fn emit_yaml_comment_line(
    output: &mut String,
    text: &str,
    indent: usize,
    dialect: &str,
) {
    let text = text.trim_matches([' ', '\t']);
    push_yaml_indent(output, indent);
    output.push('#');
    if !text.is_empty() {
        output.push(' ');
        if text.starts_with("fmt:") {
            output.push('[');
            output.push_str(dialect);
            output.push_str("] ");
        }
        for character in text.chars() {
            if yaml_comment_character_is_safe(character) {
                output.push(character);
            } else {
                use std::fmt::Write;
                let value = character as u32;
                if value <= 0xff {
                    write!(output, "\\x{value:02X}").expect("writing to a String cannot fail");
                } else if value <= 0xffff {
                    write!(output, "\\u{value:04X}").expect("writing to a String cannot fail");
                } else {
                    write!(output, "\\U{value:08X}").expect("writing to a String cannot fail");
                }
            }
        }
    }
    output.push('\n');
}

pub(crate) fn for_each_comment_line(text: &str, mut emit: impl FnMut(&str)) {
    if text.is_empty() {
        emit("");
        return;
    }

    let mut start = 0;
    let mut characters = text.char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        if !matches!(
            character,
            '\n' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        ) {
            continue;
        }
        emit(&text[start..offset]);
        start = offset + character.len_utf8();
        if character == '\r'
            && characters
                .peek()
                .is_some_and(|(_, character)| *character == '\n')
        {
            let (offset, character) = characters.next().expect("peeked at a line feed");
            start = offset + character.len_utf8();
        }
    }
    if start < text.len() {
        emit(&text[start..]);
    }
}

fn trim_comment_line(text: &str) -> &str {
    let text = text.trim_matches([' ', '\t']);
    text.strip_prefix('*')
        .unwrap_or(text)
        .trim_matches([' ', '\t'])
}

fn yaml_comment_character_is_safe(character: char) -> bool {
    let value = character as u32;
    (character == '\t' || value >= 0x20)
        && !(0x7f..=0x9f).contains(&value)
        && !matches!(character, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
}

pub(crate) fn emit_yaml_double_quoted(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\0' => output.push_str("\\0"),
            '\u{0007}' => output.push_str("\\a"),
            '\u{0008}' => output.push_str("\\b"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\u{000b}' => output.push_str("\\v"),
            '\u{000c}' => output.push_str("\\f"),
            '\r' => output.push_str("\\r"),
            '\u{001b}' => output.push_str("\\e"),
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0085}' => output.push_str("\\N"),
            '\u{00a0}' => output.push_str("\\_"),
            '\u{2028}' => output.push_str("\\L"),
            '\u{2029}' => output.push_str("\\P"),
            character if ('\u{007f}'..='\u{009f}').contains(&character) => {
                use std::fmt::Write;
                write!(output, "\\x{:02X}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            '\u{fffe}' | '\u{ffff}' => {
                use std::fmt::Write;
                write!(output, "\\u{:04X}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            character if character <= '\u{001f}' => {
                use std::fmt::Write;
                write!(output, "\\u{:04X}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}
