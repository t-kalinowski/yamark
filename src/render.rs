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
}

impl JsonSourceKind {
    pub(crate) fn for_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "json" => Some(Self::Json),
            "jsonl" | "ndjson" => Some(Self::JsonLines),
            "jsonc" => Some(Self::Jsonc),
            _ => None,
        }
    }
}

pub(crate) fn json_to_yaml_source(input: &str, kind: JsonSourceKind) -> Result<String> {
    match kind {
        JsonSourceKind::Json => render_json(input),
        JsonSourceKind::JsonLines => render_json_lines(input),
        JsonSourceKind::Jsonc => render_jsonc(input),
    }
}

fn render_jsonc(input: &str) -> Result<String> {
    let parsed =
        parse_to_ast(input, &jsonc_collect_options(), &jsonc_parse_options()).map_err(|err| {
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
                Comment::Line(comment) => emit_yaml_comment_line(output, comment.text, indent),
                Comment::Block(comment) => {
                    if comment.text.is_empty() {
                        emit_yaml_comment_line(output, "", indent);
                    } else {
                        for line in comment.text.lines() {
                            let line = line.trim().strip_prefix('*').unwrap_or(line.trim()).trim();
                            emit_yaml_comment_line(output, line, indent);
                        }
                    }
                }
            }
        }
    }
}

fn emit_yaml_comment_line(output: &mut String, text: &str, indent: usize) {
    let text = text.trim();
    push_yaml_indent(output, indent);
    output.push('#');
    if !text.is_empty() {
        output.push(' ');
        if text.starts_with("fmt:") {
            output.push_str("[jsonc] ");
        }
        output.push_str(text);
    }
    output.push('\n');
}

fn emit_yaml_double_quoted(output: &mut String, value: &str) {
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
