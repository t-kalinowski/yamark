use std::borrow::Cow;

use json_five::rt::parser::{self, JSONKeyValuePair, JSONText, JSONValue, UnaryOperator};
use json_five::tokenize::{TokType, tokenize_rt_str};
use num_bigint::BigUint;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::diagnostic::{Result, YamarkError};
use crate::json_to_yaml::{emit_yaml_comment_line, emit_yaml_double_quoted, for_each_comment_line};

pub(crate) fn json5_to_yaml_source(input: &str) -> Result<String> {
    let normalized = normalize_line_separators(input)?;
    let mut tokens = tokenize_rt_str(&normalized.source).map_err(|error| {
        invalid_json5_at(
            input,
            normalized.original_offset(error.index),
            error.message,
        )
    })?;

    // json-five 0.3.1 leaves the final slash outside block-comment spans. Its
    // round-trip parser consequently sees that slash as part of the following
    // whitespace/comment context unless the span is repaired first.
    let mut has_comments = false;
    let mut nesting: usize = 0;
    let mut previous_token_was_unary = false;
    for span in &mut tokens.tok_spans {
        if span.1 == TokType::BlockComment
            && normalized.source.as_bytes().get(span.2) == Some(&b'/')
        {
            span.2 += 1;
        }
        match span.1 {
            TokType::LeftBrace | TokType::LeftBracket => {
                nesting += 1;
                if nesting > 256 {
                    return Err(invalid_json5_at(
                        input,
                        normalized.original_offset(span.0),
                        "nesting exceeds 256 levels",
                    ));
                }
            }
            TokType::RightBrace | TokType::RightBracket => {
                nesting = nesting.saturating_sub(1);
            }
            TokType::LineComment | TokType::BlockComment => has_comments = true,
            _ => {}
        }
        if !is_trivia(&span.1) {
            let is_unary = matches!(span.1, TokType::Plus | TokType::Minus);
            if is_unary && previous_token_was_unary {
                return Err(invalid_json5_at(
                    input,
                    normalized.original_offset(span.0),
                    "Only one unary operator is allowed",
                ));
            }
            previous_token_was_unary = is_unary;
        }
    }

    // The tokenizer gives reserved words their value token types in every
    // position, while JSON5 permits them as IdentifierName object keys.
    for index in 0..tokens.tok_spans.len() {
        if !matches!(
            tokens.tok_spans[index].1,
            TokType::True | TokType::False | TokType::Null | TokType::Infinity | TokType::Nan
        ) {
            continue;
        }
        let followed_by_colon = tokens.tok_spans[index + 1..]
            .iter()
            .find(|span| !is_trivia(&span.1))
            .is_some_and(|span| span.1 == TokType::Colon);
        if followed_by_colon {
            tokens.tok_spans[index].1 = TokType::Name;
        }
    }

    let document = parser::from_tokens(&tokens).map_err(|error| {
        invalid_json5_at(
            input,
            normalized.original_offset(error.index),
            error.message,
        )
    })?;

    let mut cursor = TokenCursor::new(&tokens.tok_spans);
    let root = convert_document(document, &mut cursor, &normalized, input)?;
    let mut output = String::new();
    if has_comments {
        emit_block_node(&mut output, &root, 0);
    } else {
        emit_flow_kind(&mut output, &root.kind);
        output.push('\n');
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

#[derive(Clone, Copy)]
enum ScanState {
    Outside,
    SingleQuoted,
    DoubleQuoted,
    LineComment,
    BlockComment,
}

struct NormalizedInput<'a> {
    source: Cow<'a, str>,
    replacements: Vec<NormalizedReplacement>,
}

struct NormalizedReplacement {
    normalized_start: usize,
    normalized_end: usize,
    original_start: usize,
    original_end: usize,
}

impl NormalizedInput<'_> {
    fn original_offset(&self, normalized_offset: usize) -> usize {
        let mut adjustment = 0_isize;
        for replacement in &self.replacements {
            if normalized_offset < replacement.normalized_start {
                break;
            }
            if normalized_offset <= replacement.normalized_end {
                return if normalized_offset == replacement.normalized_end {
                    replacement.original_end
                } else {
                    replacement.original_start
                };
            }
            adjustment += (replacement.original_end - replacement.original_start) as isize
                - (replacement.normalized_end - replacement.normalized_start) as isize;
        }
        normalized_offset.saturating_add_signed(adjustment)
    }
}

struct SourceReplacement {
    start: usize,
    end: usize,
    text: &'static str,
}

fn normalize_line_separators(input: &str) -> Result<NormalizedInput<'_>> {
    let mut replacements = Vec::new();
    let mut state = ScanState::Outside;
    let mut offset = 0;

    while offset < input.len() {
        let rest = &input[offset..];
        match state {
            ScanState::Outside if rest.starts_with("//") => {
                offset += 2;
                state = ScanState::LineComment;
                continue;
            }
            ScanState::Outside if rest.starts_with("/*") => {
                offset += 2;
                state = ScanState::BlockComment;
                continue;
            }
            ScanState::BlockComment if rest.starts_with("*/") => {
                offset += 2;
                state = ScanState::Outside;
                continue;
            }
            _ => {}
        }

        let character = rest.chars().next().expect("offset is before input end");
        let next_offset = offset + character.len_utf8();
        match state {
            ScanState::Outside => {
                if character == '\u{0085}' {
                    return Err(invalid_json5_at(
                        input,
                        offset,
                        "U+0085 is not JSON5 whitespace",
                    ));
                }
                if matches!(character, '\u{2028}' | '\u{2029}') {
                    replacements.push(SourceReplacement {
                        start: offset,
                        end: next_offset,
                        text: "\n",
                    });
                }
                state = match character {
                    '\'' => ScanState::SingleQuoted,
                    '"' => ScanState::DoubleQuoted,
                    _ => ScanState::Outside,
                };
                offset = next_offset;
            }
            ScanState::SingleQuoted | ScanState::DoubleQuoted => {
                let quote = match state {
                    ScanState::SingleQuoted => '\'',
                    ScanState::DoubleQuoted => '"',
                    _ => unreachable!(),
                };
                if character == '\\' {
                    offset = next_offset;
                    if offset < input.len() {
                        let escaped = input[offset..]
                            .chars()
                            .next()
                            .expect("offset is before input end");
                        let escaped_end = offset + escaped.len_utf8();
                        offset = escaped_end;
                        if escaped == '\r' && input[offset..].starts_with('\n') {
                            offset += 1;
                        }
                    }
                } else if matches!(character, '\u{2028}' | '\u{2029}') {
                    replacements.push(SourceReplacement {
                        start: offset,
                        end: next_offset,
                        text: if character == '\u{2028}' {
                            "\\u2028"
                        } else {
                            "\\u2029"
                        },
                    });
                    offset = next_offset;
                } else {
                    offset = next_offset;
                    if character == quote {
                        state = ScanState::Outside;
                    }
                }
            }
            ScanState::LineComment => {
                if matches!(character, '\u{2028}' | '\u{2029}') {
                    replacements.push(SourceReplacement {
                        start: offset,
                        end: next_offset,
                        text: "\n",
                    });
                }
                offset = next_offset;
                if matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                    state = ScanState::Outside;
                }
            }
            ScanState::BlockComment => {
                if matches!(character, '\u{2028}' | '\u{2029}') {
                    replacements.push(SourceReplacement {
                        start: offset,
                        end: next_offset,
                        text: "\n",
                    });
                }
                offset = next_offset;
            }
        }
    }

    if replacements.is_empty() {
        return Ok(NormalizedInput {
            source: Cow::Borrowed(input),
            replacements: Vec::new(),
        });
    }

    let extra_capacity = replacements
        .iter()
        .map(|replacement| {
            replacement
                .text
                .len()
                .saturating_sub(replacement.end - replacement.start)
        })
        .sum::<usize>();
    let mut source = String::with_capacity(input.len() + extra_capacity);
    let mut normalized_replacements = Vec::with_capacity(replacements.len());
    let mut copied_through = 0;
    for replacement in replacements {
        source.push_str(&input[copied_through..replacement.start]);
        let normalized_start = source.len();
        source.push_str(replacement.text);
        normalized_replacements.push(NormalizedReplacement {
            normalized_start,
            normalized_end: source.len(),
            original_start: replacement.start,
            original_end: replacement.end,
        });
        copied_through = replacement.end;
    }
    source.push_str(&input[copied_through..]);
    Ok(NormalizedInput {
        source: Cow::Owned(source),
        replacements: normalized_replacements,
    })
}

struct TokenCursor<'a> {
    spans: &'a [(usize, TokType, usize)],
    index: usize,
}

impl<'a> TokenCursor<'a> {
    fn new(spans: &'a [(usize, TokType, usize)]) -> Self {
        Self { spans, index: 0 }
    }

    fn next(&mut self) -> Option<&'a (usize, TokType, usize)> {
        while let Some(span) = self.spans.get(self.index) {
            self.index += 1;
            if !is_trivia(&span.1) {
                return Some(span);
            }
        }
        None
    }

    fn peek(&mut self) -> Option<&'a (usize, TokType, usize)> {
        let saved = self.index;
        let value = self.next();
        self.index = saved;
        value
    }

    fn consume_if(&mut self, kind: TokType) {
        if self.peek().is_some_and(|span| span.1 == kind) {
            self.next();
        }
    }
}

fn is_trivia(kind: &TokType) -> bool {
    matches!(
        kind,
        TokType::Whitespace | TokType::LineComment | TokType::BlockComment
    )
}

#[derive(Default)]
struct Node {
    leading: Vec<String>,
    kind: NodeKind,
    trailing: Vec<String>,
}

#[derive(Default)]
enum NodeKind {
    #[default]
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array {
        items: Vec<Node>,
        tail: Vec<String>,
    },
    Object {
        members: Vec<Member>,
        tail: Vec<String>,
    },
}

struct Member {
    leading: Vec<String>,
    key: String,
    value: Node,
}

fn convert_document(
    document: JSONText,
    cursor: &mut TokenCursor<'_>,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<Node> {
    let mut node = convert_value(document.value, cursor, normalized, original)?;
    if let Some(context) = document.context {
        node.leading = comments_from_wsc(&context.wsc.0)
            .into_iter()
            .chain(node.leading)
            .collect();
        node.trailing.extend(comments_from_wsc(&context.wsc.1));
    }
    Ok(node)
}

fn convert_value(
    value: JSONValue,
    cursor: &mut TokenCursor<'_>,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<Node> {
    match value {
        JSONValue::JSONObject {
            key_value_pairs,
            context,
        } => {
            cursor.consume_if(TokType::LeftBrace);
            convert_object(key_value_pairs, context, cursor, normalized, original)
        }
        JSONValue::JSONArray { values, context } => {
            cursor.consume_if(TokType::LeftBracket);
            let mut pending = context
                .map(|context| comments_from_wsc(&context.wsc.0))
                .unwrap_or_default();
            let mut items = Vec::with_capacity(values.len());
            for item in values {
                let mut value = convert_value(item.value, cursor, normalized, original)?;
                value.leading = pending.into_iter().chain(value.leading).collect();
                pending = Vec::new();
                if let Some(context) = item.context {
                    value.trailing.extend(comments_from_wsc(&context.wsc.0));
                    if let Some(after_comma) = context.wsc.1 {
                        cursor.consume_if(TokType::Comma);
                        pending = comments_from_wsc(&after_comma);
                    }
                } else {
                    cursor.consume_if(TokType::Comma);
                }
                items.push(value);
            }
            cursor.consume_if(TokType::RightBracket);
            Ok(Node {
                kind: NodeKind::Array {
                    items,
                    tail: pending,
                },
                ..Node::default()
            })
        }
        JSONValue::Integer(raw) | JSONValue::Float(raw) | JSONValue::Exponent(raw) => {
            cursor.next();
            Ok(Node {
                kind: NodeKind::Number(normalize_decimal(&raw)),
                ..Node::default()
            })
        }
        JSONValue::Hexadecimal(raw) => {
            cursor.next();
            Ok(Node {
                kind: NodeKind::Number(normalize_hexadecimal(&raw)),
                ..Node::default()
            })
        }
        JSONValue::Null => {
            cursor.next();
            Ok(Node::default())
        }
        JSONValue::Infinity => {
            cursor.next();
            Ok(Node {
                kind: NodeKind::Number(".inf".to_owned()),
                ..Node::default()
            })
        }
        JSONValue::NaN => {
            cursor.next();
            Ok(Node {
                kind: NodeKind::Number(".nan".to_owned()),
                ..Node::default()
            })
        }
        JSONValue::Bool(value) => {
            cursor.next();
            Ok(Node {
                kind: NodeKind::Bool(value),
                ..Node::default()
            })
        }
        JSONValue::DoubleQuotedString(raw) | JSONValue::SingleQuotedString(raw) => {
            let start = cursor.next().map_or(0, |span| span.0 + 1);
            let value = decode_string(&raw, start, normalized, original)?;
            Ok(Node {
                kind: NodeKind::String(value),
                ..Node::default()
            })
        }
        JSONValue::Unary { operator, value } => {
            let hexadecimal = matches!(&*value, JSONValue::Hexadecimal(_));
            cursor.next();
            let mut node = convert_value(*value, cursor, normalized, original)?;
            let NodeKind::Number(number) = &mut node.kind else {
                return Err(YamarkError::new(
                    "invalid JSON5: unary operator requires a number",
                ));
            };
            if operator == UnaryOperator::Minus && number != ".nan" {
                if hexadecimal {
                    *number = negative_hexadecimal_to_decimal(number);
                } else {
                    number.insert(0, '-');
                }
            }
            Ok(node)
        }
        JSONValue::Identifier(raw) => {
            let start = cursor.next().map_or(0, |span| span.0);
            let value = decode_identifier(&raw, start, normalized, original)?;
            Ok(Node {
                kind: NodeKind::String(value),
                ..Node::default()
            })
        }
    }
}

fn convert_object(
    pairs: Vec<JSONKeyValuePair>,
    context: Option<parser::JSONObjectContext>,
    cursor: &mut TokenCursor<'_>,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<Node> {
    let mut pending = context
        .map(|context| comments_from_wsc(&context.wsc.0))
        .unwrap_or_default();
    let mut members = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let key = convert_key(pair.key, cursor, normalized, original)?;
        cursor.consume_if(TokType::Colon);
        let mut value = convert_value(pair.value, cursor, normalized, original)?;
        let mut leading = std::mem::take(&mut pending);
        if let Some(context) = pair.context {
            leading.extend(comments_from_wsc(&context.wsc.0));
            leading.extend(comments_from_wsc(&context.wsc.1));
            value.trailing.extend(comments_from_wsc(&context.wsc.2));
            if let Some(after_comma) = context.wsc.3 {
                cursor.consume_if(TokType::Comma);
                pending = comments_from_wsc(&after_comma);
            }
        } else {
            cursor.consume_if(TokType::Comma);
        }
        members.push(Member {
            leading,
            key,
            value,
        });
    }
    cursor.consume_if(TokType::RightBrace);
    Ok(Node {
        kind: NodeKind::Object {
            members,
            tail: pending,
        },
        ..Node::default()
    })
}

fn convert_key(
    value: JSONValue,
    cursor: &mut TokenCursor<'_>,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<String> {
    let span = cursor.next();
    let start = span.map_or(0, |span| span.0);
    match value {
        JSONValue::Identifier(raw) => decode_identifier(&raw, start, normalized, original),
        JSONValue::DoubleQuotedString(raw) | JSONValue::SingleQuotedString(raw) => {
            decode_string(&raw, start + 1, normalized, original)
        }
        _ => Err(invalid_json5_at(
            original,
            normalized.original_offset(start),
            "object key is not an IdentifierName or string",
        )),
    }
}

fn decode_string(
    raw: &str,
    base_offset: usize,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<String> {
    let mut decoded = String::with_capacity(raw.len());
    let mut offset = 0;
    while offset < raw.len() {
        let character = raw[offset..]
            .chars()
            .next()
            .expect("offset is before string end");
        if character != '\\' {
            decoded.push(character);
            offset += character.len_utf8();
            continue;
        }

        let escape_offset = offset;
        offset += 1;
        if offset == raw.len() {
            return Err(decode_error(
                original,
                normalized,
                base_offset + escape_offset,
                "unterminated escape sequence",
            ));
        }
        let escaped = raw[offset..]
            .chars()
            .next()
            .expect("offset is before string end");
        offset += escaped.len_utf8();
        match escaped {
            '\'' => decoded.push('\''),
            '"' => decoded.push('"'),
            '\\' => decoded.push('\\'),
            '/' => decoded.push('/'),
            'b' => decoded.push('\u{0008}'),
            'f' => decoded.push('\u{000c}'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            'v' => decoded.push('\u{000b}'),
            '0' => {
                if raw[offset..]
                    .chars()
                    .next()
                    .is_some_and(|next| next.is_ascii_digit())
                {
                    return Err(decode_error(
                        original,
                        normalized,
                        base_offset + escape_offset,
                        "legacy numeric escape sequences are not supported",
                    ));
                }
                decoded.push('\0');
            }
            '1'..='9' => {
                return Err(decode_error(
                    original,
                    normalized,
                    base_offset + escape_offset,
                    "legacy numeric escape sequences are not supported",
                ));
            }
            'x' => {
                let (code_unit, end) = decode_hex(raw, offset, 2).ok_or_else(|| {
                    decode_error(
                        original,
                        normalized,
                        base_offset + escape_offset,
                        "invalid hexadecimal escape sequence",
                    )
                })?;
                offset = end;
                decoded.push(char::from_u32(code_unit).expect("two hex digits fit in char"));
            }
            'u' => {
                let (first, end) = decode_hex(raw, offset, 4).ok_or_else(|| {
                    decode_error(
                        original,
                        normalized,
                        base_offset + escape_offset,
                        "invalid Unicode escape sequence",
                    )
                })?;
                offset = end;
                if (0xD800..=0xDBFF).contains(&first) {
                    if !raw[offset..].starts_with("\\u") {
                        return Err(decode_error(
                            original,
                            normalized,
                            base_offset + escape_offset,
                            "lone high surrogate in Unicode escape sequence",
                        ));
                    }
                    let second_offset = offset;
                    let (second, end) = decode_hex(raw, offset + 2, 4).ok_or_else(|| {
                        decode_error(
                            original,
                            normalized,
                            base_offset + second_offset,
                            "invalid low surrogate in Unicode escape sequence",
                        )
                    })?;
                    if !(0xDC00..=0xDFFF).contains(&second) {
                        return Err(decode_error(
                            original,
                            normalized,
                            base_offset + second_offset,
                            "invalid low surrogate in Unicode escape sequence",
                        ));
                    }
                    offset = end;
                    let scalar = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
                    decoded.push(char::from_u32(scalar).expect("valid surrogate pair"));
                } else if (0xDC00..=0xDFFF).contains(&first) {
                    return Err(decode_error(
                        original,
                        normalized,
                        base_offset + escape_offset,
                        "lone low surrogate in Unicode escape sequence",
                    ));
                } else {
                    decoded.push(char::from_u32(first).expect("non-surrogate code unit"));
                }
            }
            '\n' | '\u{2028}' | '\u{2029}' => {}
            '\r' => {
                if raw[offset..].starts_with('\n') {
                    offset += 1;
                }
            }
            other => decoded.push(other),
        }
    }
    Ok(decoded)
}

fn decode_identifier(
    raw: &str,
    base_offset: usize,
    normalized: &NormalizedInput,
    original: &str,
) -> Result<String> {
    let mut decoded = String::with_capacity(raw.len());
    let mut offset = 0;
    while offset < raw.len() {
        let source_offset = offset;
        let character = raw[offset..]
            .chars()
            .next()
            .expect("offset is before identifier end");
        let value = if character == '\\' {
            if !raw[offset..].starts_with("\\u") {
                return Err(decode_error(
                    original,
                    normalized,
                    base_offset + offset,
                    "identifier escapes must use \\u followed by four hex digits",
                ));
            }
            let (code_unit, end) = decode_hex(raw, offset + 2, 4).ok_or_else(|| {
                decode_error(
                    original,
                    normalized,
                    base_offset + offset,
                    "invalid Unicode escape in identifier",
                )
            })?;
            offset = end;
            char::from_u32(code_unit).ok_or_else(|| {
                decode_error(
                    original,
                    normalized,
                    base_offset + source_offset,
                    "surrogate escapes are not valid in identifiers",
                )
            })?
        } else {
            offset += character.len_utf8();
            character
        };
        let valid = if decoded.is_empty() {
            is_identifier_start(value)
        } else {
            is_identifier_part(value)
        };
        if !valid {
            return Err(decode_error(
                original,
                normalized,
                base_offset + source_offset,
                "escaped character is not valid in a JSON5 IdentifierName",
            ));
        }
        decoded.push(value);
    }
    Ok(decoded)
}

fn decode_hex(raw: &str, offset: usize, digits: usize) -> Option<(u32, usize)> {
    let end = offset.checked_add(digits)?;
    let value = raw.get(offset..end)?;
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some((u32::from_str_radix(value, 16).ok()?, end))
}

fn is_identifier_start(character: char) -> bool {
    matches!(character, '$' | '_')
        || matches!(
            get_general_category(character),
            GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
                | GeneralCategory::ModifierLetter
                | GeneralCategory::OtherLetter
                | GeneralCategory::LetterNumber
        )
}

fn is_identifier_part(character: char) -> bool {
    is_identifier_start(character)
        || matches!(character, '\u{200c}' | '\u{200d}')
        || matches!(
            get_general_category(character),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::DecimalNumber
                | GeneralCategory::ConnectorPunctuation
        )
}

fn normalize_decimal(raw: &str) -> String {
    let exponent = raw.find(['e', 'E']);
    let (mantissa, suffix) = exponent.map_or((raw, ""), |index| raw.split_at(index));
    let mut normalized = String::with_capacity(raw.len() + 1);
    if mantissa.starts_with('.') {
        normalized.push('0');
    }
    normalized.push_str(mantissa);
    if mantissa.ends_with('.') {
        normalized.push('0');
    }
    normalized.push_str(suffix);
    normalized
}

fn normalize_hexadecimal(raw: &str) -> String {
    if let Some(digits) = raw.strip_prefix("0X") {
        format!("0x{digits}")
    } else {
        raw.to_owned()
    }
}

fn negative_hexadecimal_to_decimal(raw: &str) -> String {
    let digits = raw
        .strip_prefix("0x")
        .expect("JSON5 hexadecimal numbers have a normalized prefix");
    let value = BigUint::parse_bytes(digits.as_bytes(), 16)
        .expect("JSON5 hexadecimal numbers contain only hexadecimal digits");
    let mut decimal = value.to_str_radix(10);
    decimal.insert(0, '-');
    decimal
}

fn comments_from_wsc(wsc: &str) -> Vec<String> {
    let mut comments = Vec::new();
    let mut offset = 0;
    while offset < wsc.len() {
        let rest = &wsc[offset..];
        if let Some(body) = rest.strip_prefix("//") {
            let end = body
                .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                .unwrap_or(body.len());
            comments.push(body[..end].trim().to_owned());
            offset += 2 + end;
        } else if let Some(body) = rest.strip_prefix("/*") {
            let end = body.find("*/").unwrap_or(body.len());
            let body = &body[..end];
            for_each_comment_line(body, |line| {
                let line = line.trim_matches([' ', '\t']);
                comments.push(
                    line.strip_prefix('*')
                        .unwrap_or(line)
                        .trim_matches([' ', '\t'])
                        .to_owned(),
                );
            });
            offset += 2 + end + usize::from(body.len() < rest.len().saturating_sub(2)) * 2;
        } else {
            let character = rest.chars().next().expect("offset is before WSC end");
            offset += character.len_utf8();
        }
    }
    comments
}

fn emit_flow_kind(output: &mut String, kind: &NodeKind) {
    match kind {
        NodeKind::Null => output.push_str("null"),
        NodeKind::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        NodeKind::Number(value) => output.push_str(value),
        NodeKind::String(value) => emit_yaml_double_quoted(output, value),
        NodeKind::Array { items, .. } => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                emit_flow_kind(output, &item.kind);
            }
            output.push(']');
        }
        NodeKind::Object { members, .. } => {
            output.push('{');
            for (index, member) in members.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                emit_yaml_mapping_key(output, &member.key);
                output.push_str(": ");
                emit_flow_kind(output, &member.value.kind);
            }
            output.push('}');
        }
    }
}

fn emit_block_node(output: &mut String, node: &Node, indent: usize) {
    emit_comments(output, &node.leading, indent);
    emit_block_kind(output, &node.kind, indent);
    emit_comments(output, &node.trailing, indent);
}

fn emit_block_kind(output: &mut String, kind: &NodeKind, indent: usize) {
    match kind {
        NodeKind::Object { members, tail } if !members.is_empty() => {
            for member in members {
                emit_comments(output, &member.leading, indent);
                if let Some((syntax, comments)) = empty_collection(&member.value.kind) {
                    emit_comments(output, &member.value.leading, indent);
                    emit_comments(output, comments, indent);
                    push_indent(output, indent);
                    emit_yaml_mapping_key(output, &member.key);
                    output.push_str(": ");
                    output.push_str(syntax);
                    output.push('\n');
                    emit_comments(output, &member.value.trailing, indent);
                    continue;
                }
                push_indent(output, indent);
                emit_yaml_mapping_key(output, &member.key);
                if node_is_inline(&member.value) {
                    output.push_str(": ");
                    emit_flow_kind(output, &member.value.kind);
                    output.push('\n');
                    emit_comments(output, &member.value.trailing, indent);
                } else {
                    output.push_str(":\n");
                    emit_block_node(output, &member.value, indent + 2);
                }
            }
            emit_comments(output, tail, indent);
        }
        NodeKind::Object { tail, .. } => {
            emit_comments(output, tail, indent);
            push_indent(output, indent);
            output.push_str("{}\n");
        }
        NodeKind::Array { items, tail } if !items.is_empty() => {
            for item in items {
                emit_comments(output, &item.leading, indent);
                if let Some((syntax, comments)) = empty_collection(&item.kind) {
                    emit_comments(output, comments, indent);
                    push_indent(output, indent);
                    output.push_str("- ");
                    output.push_str(syntax);
                    output.push('\n');
                    emit_comments(output, &item.trailing, indent);
                    continue;
                }
                push_indent(output, indent);
                if kind_is_inline(&item.kind) {
                    output.push_str("- ");
                    emit_flow_kind(output, &item.kind);
                    output.push('\n');
                } else {
                    output.push_str("-\n");
                    emit_block_kind(output, &item.kind, indent + 2);
                }
                emit_comments(output, &item.trailing, indent);
            }
            emit_comments(output, tail, indent);
        }
        NodeKind::Array { tail, .. } => {
            emit_comments(output, tail, indent);
            push_indent(output, indent);
            output.push_str("[]\n");
        }
        kind => {
            push_indent(output, indent);
            emit_flow_kind(output, kind);
            output.push('\n');
        }
    }
}

fn node_is_inline(node: &Node) -> bool {
    node.leading.is_empty() && kind_is_inline(&node.kind)
}

fn empty_collection(kind: &NodeKind) -> Option<(&'static str, &[String])> {
    match kind {
        NodeKind::Object { members, tail } if members.is_empty() => Some(("{}", tail)),
        NodeKind::Array { items, tail } if items.is_empty() => Some(("[]", tail)),
        _ => None,
    }
}

fn kind_is_inline(kind: &NodeKind) -> bool {
    match kind {
        NodeKind::Null | NodeKind::Bool(_) | NodeKind::Number(_) | NodeKind::String(_) => true,
        NodeKind::Array { items, tail } => items.is_empty() && tail.is_empty(),
        NodeKind::Object { members, tail } => members.is_empty() && tail.is_empty(),
    }
}

fn emit_comments(output: &mut String, comments: &[String], indent: usize) {
    for comment in comments {
        emit_yaml_comment_line(output, comment, indent, "json5");
    }
}

fn emit_yaml_mapping_key(output: &mut String, value: &str) {
    let mut characters = value.chars();
    let starts_plain = characters.next().is_some_and(is_identifier_start);
    let continues_plain = characters.all(is_identifier_part);
    let reserved = value.eq_ignore_ascii_case("null")
        || value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false");
    if starts_plain && continues_plain && !reserved {
        output.push_str(value);
    } else {
        emit_yaml_double_quoted(output, value);
    }
}

fn push_indent(output: &mut String, indent: usize) {
    output.extend(std::iter::repeat_n(' ', indent));
}

fn decode_error(
    original: &str,
    normalized: &NormalizedInput,
    normalized_offset: usize,
    message: impl Into<String>,
) -> YamarkError {
    invalid_json5_at(
        original,
        normalized.original_offset(normalized_offset),
        message,
    )
}

fn invalid_json5_at(
    original: &str,
    original_offset: usize,
    message: impl Into<String>,
) -> YamarkError {
    let (line, column) = line_column(original, original_offset);
    YamarkError::at(format!("invalid JSON5: {}", message.into()), line, column)
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
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
