use std::path::Path;

use jsonc_parser::ast::{ObjectPropName, Value};
use jsonc_parser::{CollectOptions, CommentCollectionStrategy, ParseOptions, parse_to_ast};

use crate::diagnostic::{Result, YamarkError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonSourceKind {
    Json,
    JsonLines,
}

impl JsonSourceKind {
    pub(crate) fn for_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "json" => Some(Self::Json),
            "jsonl" | "ndjson" => Some(Self::JsonLines),
            _ => None,
        }
    }
}

pub(crate) fn json_to_yaml_source(input: &str, kind: JsonSourceKind) -> Result<String> {
    match kind {
        JsonSourceKind::Json => render_json(input),
        JsonSourceKind::JsonLines => render_json_lines(input),
    }
}

fn render_json(input: &str) -> Result<String> {
    let value = parse_json_value(input, 0)?;
    let mut output = String::with_capacity(input.len());
    emit_json_value(&mut output, &value);
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
