//! Source-level YAML indicators, scalar boundaries, properties, and preservation checks.

use super::scalar::decode_quoted_scalar;
use super::simple::{simple_mapping_colon, strip_newline};
use crate::core::directives::DirectiveTargetKind;
use crate::core::source::{SourceBuffer, Span};
use crate::core::yaml_model::{YamlBlockChomp, YamlBlockScalarHeader, YamlScalarStyle};
use crate::core::yaml_scan::YamlLineScan;
use crate::diagnostic::YamarkError;
use memchr::memchr2;

pub(super) fn yaml_scan_has_active_fmt_directive(
    source: &SourceBuffer,
    scan: &YamlLineScan,
) -> bool {
    let mut block_scalar_indent = None::<usize>;
    for line in &scan.lines {
        let text = source.slice(line.content);
        if let Some(indent) = block_scalar_indent {
            if yaml_line_is_block_scalar_body(text, line.indent, indent) {
                continue;
            }
            block_scalar_indent = None;
        }
        if yaml_line_has_fmt_directive_comment(text) {
            return true;
        }
        if line.scalar.is_some_and(|scalar| {
            scalar.kind == crate::core::yaml_scan::YamlScalarTokenKind::BlockScalarHeader
        }) {
            block_scalar_indent = Some(line.indent);
        }
    }
    false
}

pub(super) fn yaml_scan_has_unsupported_preserve_syntax(
    source: &SourceBuffer,
    scan: &YamlLineScan,
) -> bool {
    let mut line = scan.start_line;
    while line < scan.end_line {
        let text = source.line_text(line);
        if let Some(marker) = document_marker_line_info(text)
            && document_marker_inline_content_requires_preservation(text, marker)
        {
            return true;
        }
        if let Some(block) = block_scalar_at(source, line, scan.end_line) {
            line = source.line_at_byte(block.full.end.saturating_sub(1)) + 1;
            continue;
        }
        let quoted_block = supported_quoted_scalar_block_at(source, line, scan.end_line);
        if unsupported_yaml_line_syntax(source, line, scan.end_line, quoted_block.is_some()) {
            return true;
        }
        if let Some(block) = quoted_block {
            line = source.line_at_byte(block.full.end.saturating_sub(1)) + 1;
            continue;
        }
        line += 1;
    }
    false
}

fn unsupported_yaml_line_syntax(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    supported_multiline_quote: bool,
) -> bool {
    let text = source.line_text(line);
    let (body, _) = strip_newline(text);
    let trimmed = body
        .strip_prefix('\u{feff}')
        .unwrap_or(body)
        .trim_ascii_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    if single_line_flow_value_needs_no_preserve_check(text) {
        return false;
    }
    if trimmed.starts_with('%') || unsupported_multiline_flow_collection(source, line, end) {
        return true;
    }
    if !supported_multiline_quote && line_starts_unclosed_quoted_scalar(text) {
        return true;
    }

    let indent = indentation(text);
    if trimmed.starts_with(':') && explicit_value_line(text, indent).is_none() {
        return true;
    }
    if explicit_key_line(text, indent).is_some() {
        return explicit_key_has_unsupported_multiline_key(source, line, end, indent);
    }
    if trimmed.starts_with(['&', '!'])
        && (mapping_colon_at(text, indent).is_some()
            || property_only_line_requires_preservation(trimmed))
    {
        return true;
    }
    if trimmed.starts_with('*') && mapping_colon_at(text, indent).is_some() {
        return true;
    }
    if mapping_value_has_unsupported_property_token(text, indent) {
        return true;
    }
    if unsupported_plain_mapping_key_line(text, indent) {
        return true;
    }
    if unsupported_sequence_entry_line(text, indent) {
        return true;
    }
    false
}

fn single_line_flow_value_needs_no_preserve_check(text: &str) -> bool {
    let Some(collection_start) = first_flow_opener(text) else {
        return false;
    };
    !(text[..collection_start].contains("&:") || text[..collection_start].contains("*:"))
        && flow_collection_closes_at_line_end(text, collection_start)
}

fn first_flow_opener(text: &str) -> Option<usize> {
    text.as_bytes()
        .iter()
        .position(|byte| matches!(byte, b'[' | b'{'))
}

fn unsupported_multiline_flow_collection(source: &SourceBuffer, line: usize, end: usize) -> bool {
    let text = source.line_text(line);
    let Some(value_start) = line_flow_value_start(text) else {
        return false;
    };
    let collection_start = node_properties_content_start(text, value_start);
    if flow_collection_closes_at_line_end(text, collection_start) {
        return false;
    }
    let Some(FlowCollectionScan::Complete(block)) = flow_collection_block_from_value(
        source,
        line,
        end,
        value_start,
        source.lines[line].full.start(),
    ) else {
        return false;
    };
    if source.line_at_byte(block.collection.end.saturating_sub(1)) == line {
        return false;
    }
    let collection = source.slice(block.collection);
    collection.contains("\n?")
        || collection.contains("\n:")
        || collection.contains(" : ")
        || collection.contains("::")
        || collection.contains('#')
        || flow_collection_contains_multiline_quote(collection)
}

fn flow_collection_closes_at_line_end(text: &str, collection_start: usize) -> bool {
    let close = match text.as_bytes().get(collection_start) {
        Some(b'[') => b']',
        Some(b'{') => b'}',
        _ => return false,
    };
    let comment_start = find_trailing_comment(text, collection_start).unwrap_or(text.len());
    let value_end = trim_end_before(text, collection_start, comment_start);
    value_end > collection_start && text.as_bytes().get(value_end - 1) == Some(&close)
}

fn document_marker_inline_content_requires_preservation(
    text: &str,
    marker: DocumentMarkerLine,
) -> bool {
    let Some(content_start) = marker.inline_content_start else {
        return false;
    };
    text[content_start..]
        .trim_ascii_start()
        .starts_with(['|', '>'])
}

fn explicit_key_has_unsupported_multiline_key(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    indent: usize,
) -> bool {
    let mut next = line + 1;
    while next < end {
        let next_text = source.line_text(next);
        let next_trimmed = next_text.trim_ascii_start();
        if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
            next += 1;
            continue;
        }
        return explicit_value_line(next_text, indent).is_none();
    }
    false
}

fn property_only_line_requires_preservation(trimmed: &str) -> bool {
    let mut tokens = trimmed.split_whitespace();
    let Some(first) = tokens.next() else {
        return false;
    };
    (first.starts_with('&') || first.starts_with('!')) && tokens.next().is_none()
}

fn mapping_value_has_unsupported_property_token(text: &str, indent: usize) -> bool {
    let Some(colon) = mapping_colon_at(text, indent) else {
        return false;
    };
    let value_start = skip_ascii_whitespace(text, colon + 1);
    let token_end = text[value_start..]
        .find(char::is_whitespace)
        .map_or(text.len(), |offset| value_start + offset);
    let token = &text[value_start..token_end];
    matches!(token.as_bytes().first(), Some(b'&' | b'*')) && token[1..].contains(':')
}

fn unsupported_plain_mapping_key_line(text: &str, indent: usize) -> bool {
    if sequence_line(text, indent).is_some()
        || explicit_key_line(text, indent).is_some()
        || explicit_value_line(text, indent).is_some()
        || mapping_colon_at(text, indent).is_some()
    {
        return false;
    }
    text[indent..].contains(": ")
}

fn unsupported_sequence_entry_line(text: &str, indent: usize) -> bool {
    let Some(marker) = sequence_line(text, indent) else {
        return false;
    };
    let value_start = skip_ascii_whitespace(text, marker + 1);
    if value_start >= text.len() {
        return false;
    }
    let value = &text[value_start..];
    if value.starts_with(['[', '{']) {
        return false;
    }
    if let Some(rest) = value.strip_prefix('?') {
        let rest = rest.trim_ascii_start();
        return !rest.starts_with(['[', '{']);
    }
    let content_start = node_properties_content_start(text, value_start);
    if matches!(text.as_bytes().get(content_start), Some(b'\'' | b'"')) {
        return false;
    }
    if value.starts_with(['&', '!']) && value.contains(" : ") {
        return true;
    }
    mapping_colon_from(text, value_start).is_none() && value.contains(": ")
}

fn line_starts_unclosed_quoted_scalar(text: &str) -> bool {
    let end = find_trailing_comment(text, 0).unwrap_or(text.len());
    let Some(start) = yaml_line_value_content_start(text) else {
        return false;
    };
    if start >= end {
        return false;
    }
    let start = skip_ascii_whitespace(text, start);
    let start = node_properties_content_start(&text[..end], start);
    let Some(quote @ (b'\'' | b'"')) = text.as_bytes().get(start).copied() else {
        return false;
    };
    quoted_scalar_close(&text[..end], start, quote).is_none()
}

fn yaml_line_value_content_start(text: &str) -> Option<usize> {
    let indent = indentation(text);
    if let Some(marker) = sequence_line(text, indent) {
        Some(skip_ascii_whitespace(text, marker + 1))
    } else if let Some(colon) = mapping_colon_at(text, indent) {
        Some(skip_ascii_whitespace(text, colon + 1))
    } else if let Some(colon) = explicit_value_line(text, indent) {
        Some(skip_ascii_whitespace(text, colon + 1))
    } else {
        Some(indent)
    }
}

fn flow_collection_contains_multiline_quote(text: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_double {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_double = false;
            } else if ch == '\n' || ch == '\r' {
                return true;
            }
            continue;
        }
        if in_single {
            if ch == '\'' {
                if chars.peek().is_some_and(|next| *next == '\'') {
                    chars.next();
                } else {
                    in_single = false;
                }
            } else if ch == '\n' || ch == '\r' {
                return true;
            }
            continue;
        }
        if ch == '\'' {
            in_single = true;
        } else if ch == '"' {
            in_double = true;
        }
    }
    false
}

pub(super) fn compact_nested_document_index(index: usize) -> u32 {
    assert!(
        u32::try_from(index).is_ok(),
        "nested document count exceeded u32::MAX"
    );
    index as u32
}

fn yaml_line_is_block_scalar_body(text: &str, indent: usize, block_scalar_indent: usize) -> bool {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.trim_ascii().is_empty() || text.starts_with('\t') || indent > block_scalar_indent
}

fn yaml_line_has_fmt_directive_comment(text: &str) -> bool {
    let trimmed = text.strip_prefix('\u{feff}').unwrap_or(text);
    let value_start = text.len() - trimmed.len();
    let Some(comment_start) = find_trailing_comment(text, value_start) else {
        return false;
    };
    text[comment_start..]
        .strip_prefix('#')
        .is_some_and(|rest| rest.trim_ascii_start().starts_with("fmt:"))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LineValue {
    pub(super) value: Span,
    pub(super) trailing_comment: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ScalarMetadata {
    pub(super) tag: Option<Span>,
    pub(super) anchor: Option<Span>,
    pub(super) content: Span,
}

pub(super) fn scalar_content_flow_collection_target(
    source: &SourceBuffer,
    content: Span,
) -> Option<DirectiveTargetKind> {
    match source.slice(content).trim_ascii_start().as_bytes().first() {
        Some(b'[') => Some(DirectiveTargetKind::YamlFlowSequence),
        Some(b'{') => Some(DirectiveTargetKind::YamlFlowMapping),
        _ => None,
    }
}

pub(super) fn flow_collection_directive_target(
    source: &SourceBuffer,
    collection: Span,
) -> DirectiveTargetKind {
    scalar_content_flow_collection_target(source, collection)
        .expect("flow collection starts with a flow collection indicator")
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PlainScalarContinuation {
    None,
    Inline { parent_indent: usize },
    Block { indent: usize },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PlainScalarExtension {
    pub(super) span_end: usize,
    pub(super) value_end: usize,
    pub(super) trailing_comment: Option<Span>,
}

pub(super) fn plain_scalar_continuation_line(
    text: &str,
    continuation: PlainScalarContinuation,
) -> bool {
    if text.trim_ascii_start().starts_with('#')
        || document_marker(text)
        || standard_yaml_directive(text)
    {
        return false;
    }

    let indent = indentation(text);
    match continuation {
        PlainScalarContinuation::None => false,
        PlainScalarContinuation::Inline { parent_indent } => indent > parent_indent,
        PlainScalarContinuation::Block {
            indent: scalar_indent,
        } => {
            if indent < scalar_indent {
                return false;
            }
            if indent == scalar_indent
                && (mapping_colon_at(text, scalar_indent).is_some()
                    || sequence_line(text, scalar_indent).is_some()
                    || explicit_key_line(text, scalar_indent).is_some()
                    || explicit_value_line(text, scalar_indent).is_some())
            {
                return false;
            }
            true
        }
    }
}

pub(super) fn split_line_value_comment(
    text: &str,
    value_start: usize,
    line_start: usize,
) -> LineValue {
    let comment_start = find_trailing_comment(text, value_start);
    let value_end = comment_start
        .map(|comment_start| trim_end_before(text, value_start, comment_start))
        .unwrap_or(text.len());
    LineValue {
        value: Span::new(line_start + value_start, line_start + value_end),
        trailing_comment: comment_start
            .map(|comment_start| Span::new(line_start + comment_start, line_start + text.len())),
    }
}

pub(super) fn find_trailing_comment(text: &str, value_start: usize) -> Option<usize> {
    if !text.as_bytes()[value_start..].contains(&b'#') {
        return None;
    }

    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut flow_depth = 0usize;
    let mut chars = text[value_start..].char_indices().peekable();

    while let Some((relative, ch)) = chars.next() {
        let index = value_start + relative;
        if in_double {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_double = false;
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
            '#' if flow_depth == 0 && comment_can_start(text, value_start, index) => {
                return Some(index);
            }
            _ => {}
        }
    }
    None
}

pub(super) fn comment_can_start(text: &str, value_start: usize, index: usize) -> bool {
    index == value_start
        || text[..index]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
}

pub(super) fn flow_colon_is_value_indicator(
    text: &str,
    colon: usize,
    adjacent_allowed: bool,
) -> bool {
    if adjacent_allowed {
        return true;
    }
    let Some(next) = text.as_bytes().get(colon + 1).copied() else {
        return true;
    };
    next.is_ascii_whitespace() || matches!(next, b',' | b']' | b'}')
}

fn trim_end_before(text: &str, start: usize, mut end: usize) -> usize {
    while end > start {
        let Some(ch) = text[..end].chars().next_back() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        end -= ch.len_utf8();
    }
    end
}

pub(super) fn scalar_metadata(source: &SourceBuffer, value: impl Into<Span>) -> ScalarMetadata {
    let value = value.into();
    let text = source.slice(value);
    if let (Some(first), Some(last)) = (text.as_bytes().first(), text.as_bytes().last())
        && !first.is_ascii_whitespace()
        && !last.is_ascii_whitespace()
        && !matches!(first, b'!' | b'&')
    {
        return ScalarMetadata {
            tag: None,
            anchor: None,
            content: value,
        };
    }
    let (trimmed_start, trimmed_end) = trim_ascii_bounds(text);
    let mut cursor = trimmed_start;
    let mut tag = None;
    let mut anchor = None;
    let mut content_start = trimmed_start;

    while cursor < trimmed_end {
        while cursor < trimmed_end && text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= trimmed_end {
            content_start = trimmed_end;
            break;
        }

        let token_start = cursor;
        while cursor < trimmed_end && !text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let token = &text[token_start..cursor];
        if token.starts_with('!') {
            tag = Some(Span::new(value.start + token_start, value.start + cursor));
            content_start = cursor;
            continue;
        }
        if token.starts_with('&') {
            anchor = Some(Span::new(value.start + token_start, value.start + cursor));
            content_start = cursor;
            continue;
        }

        content_start = token_start;
        break;
    }

    while content_start < trimmed_end && text.as_bytes()[content_start].is_ascii_whitespace() {
        content_start += 1;
    }

    ScalarMetadata {
        tag,
        anchor,
        content: Span::new(value.start + content_start, value.start + trimmed_end),
    }
}

fn trim_ascii_bounds(text: &str) -> (usize, usize) {
    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut end = text.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start, end)
}

pub(super) fn trim_span_ascii(source: &SourceBuffer, span: Span) -> Span {
    let (start, end) = trim_ascii_bounds(source.slice(span));
    Span::new(span.start + start, span.start + end)
}

pub(super) fn indentation(text: &str) -> usize {
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    bom + text[bom..].bytes().take_while(|byte| *byte == b' ').count()
}

pub(super) fn tab_in_indentation(text: &str) -> Option<usize> {
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    for (offset, byte) in text[bom..].bytes().enumerate() {
        match byte {
            b' ' => {}
            b'\t' => return Some(bom + offset),
            _ => return None,
        }
    }
    None
}

pub(super) fn tab_indented_block_at(source: &SourceBuffer, line: usize, end: usize) -> Span {
    let start = source.lines[line].full.start();
    let first_text = source.line_text(line);
    let base_indent = tab_in_indentation(first_text).unwrap_or_else(|| indentation(first_text));
    let mut scan = line + 1;
    while scan < end {
        let text = source.line_text(scan);
        if text.trim_ascii().is_empty()
            || tab_in_indentation(text).is_some()
            || indentation(text) > base_indent
        {
            scan += 1;
            continue;
        }
        break;
    }
    Span::new(start, source.lines[scan - 1].full.end())
}

pub(super) fn mapping_colon_at(text: &str, indent: usize) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    let trimmed = text[indent..].trim_ascii_end();
    if trimmed.starts_with('-') {
        return None;
    }
    mapping_colon_from(text, indent)
}

pub(super) fn mapping_colon_from(text: &str, key_start_column: usize) -> Option<usize> {
    if key_start_column >= text.len() {
        return None;
    }
    let relative = simple_mapping_colon(&text[key_start_column..])?;
    let colon = key_start_column + relative;
    if text
        .as_bytes()
        .get(colon + 1)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    Some(colon)
}

pub(super) fn explicit_key_line(text: &str, indent: usize) -> Option<usize> {
    explicit_indicator_line(text, indent, b'?')
}

pub(super) fn explicit_value_line(text: &str, indent: usize) -> Option<usize> {
    explicit_indicator_line(text, indent, b':')
}

fn explicit_indicator_line(text: &str, indent: usize, indicator: u8) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    explicit_indicator_at(text, indent, indicator)
}

pub(super) fn explicit_indicator_at(text: &str, marker: usize, indicator: u8) -> Option<usize> {
    if text.as_bytes().get(marker) != Some(&indicator) {
        return None;
    }
    if text
        .as_bytes()
        .get(marker + 1)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    Some(marker)
}

pub(super) fn sequence_line(text: &str, indent: usize) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    compact_sequence_marker_at(text, indent)
}

pub(super) fn compact_sequence_marker_at(text: &str, marker: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(marker) != Some(&b'-') {
        return None;
    }
    if bytes
        .get(marker + 1)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        return None;
    }
    Some(marker)
}

pub(super) fn scalar_style(value: &str) -> YamlScalarStyle {
    if value.starts_with('\'') {
        YamlScalarStyle::SingleQuoted
    } else if value.starts_with('"') {
        YamlScalarStyle::DoubleQuoted
    } else {
        YamlScalarStyle::Plain
    }
}

pub(super) fn alias_scalar(value: &str) -> bool {
    let value = value.trim_ascii();
    value
        .strip_prefix('*')
        .is_some_and(|anchor| !anchor.is_empty() && anchor.chars().all(is_anchor_char))
}

fn is_anchor_char(ch: char) -> bool {
    !ch.is_whitespace()
        && !matches!(
            ch,
            '[' | ']'
                | '{'
                | '}'
                | ','
                | ':'
                | '?'
                | '#'
                | '&'
                | '*'
                | '!'
                | '|'
                | '>'
                | '\''
                | '"'
                | '%'
                | '@'
                | '`'
        )
}

pub(super) fn block_scalar_style(header: &str) -> YamlScalarStyle {
    if header.contains('|') {
        YamlScalarStyle::LiteralBlock
    } else {
        YamlScalarStyle::FoldedBlock
    }
}

pub(super) fn document_marker(text: &str) -> bool {
    document_marker_kind(text).is_some()
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DocumentMarkerLine {
    kind: DocumentMarkerKind,
    pub(super) inline_content_start: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocumentMarkerKind {
    Start,
    End,
}

pub(super) fn document_marker_kind(text: &str) -> Option<DocumentMarkerKind> {
    document_marker_line_info(text).map(|line| line.kind)
}

pub(super) fn document_marker_line_info(text: &str) -> Option<DocumentMarkerLine> {
    let (body, _) = strip_newline(text);
    let marker_start = body.len() - body.trim_ascii_start().len();
    let rest = &body[marker_start..];
    let (kind, marker) = if rest.starts_with("---") {
        (DocumentMarkerKind::Start, "---")
    } else if rest.starts_with("...") {
        (DocumentMarkerKind::End, "...")
    } else {
        return None;
    };
    let marker_end = marker_start + marker.len();
    let rest = &body[marker_end..];
    if rest.is_empty() {
        return Some(DocumentMarkerLine {
            kind,
            inline_content_start: None,
        });
    }
    if !rest
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_whitespace())
    {
        return None;
    }
    let inline_content_start = if kind == DocumentMarkerKind::Start {
        document_marker_inline_content_start(body, marker_end)
    } else {
        None
    };
    Some(DocumentMarkerLine {
        kind,
        inline_content_start,
    })
}

fn document_marker_inline_content_start(body: &str, marker_end: usize) -> Option<usize> {
    let content_start = skip_ascii_whitespace(body, marker_end);
    let content = &body[content_start..];
    if content.is_empty() || content.starts_with('#') || document_marker_property_only(content) {
        None
    } else {
        Some(content_start)
    }
}

fn document_marker_property_only(content: &str) -> bool {
    let comment_start = find_trailing_comment(content, 0).unwrap_or(content.len());
    let content = content[..comment_start].trim_ascii();
    !content.is_empty()
        && content
            .split_whitespace()
            .all(|token| token.starts_with('!') || token.starts_with('&'))
}

pub(super) fn standard_yaml_directive(text: &str) -> bool {
    let trimmed = text.trim_ascii_start();
    trimmed.starts_with("%YAML ") || trimmed.starts_with("%TAG ")
}

pub(super) fn line_ending_for_span(source: &SourceBuffer, span: impl Into<Span>) -> &'static str {
    let span = span.into();
    let text = source.slice(span);
    if text.ends_with("\r\n") {
        "\r\n"
    } else if text.ends_with('\n') {
        "\n"
    } else if text.ends_with('\r') {
        "\r"
    } else {
        ""
    }
}

pub(super) fn preserves_yaml_template_span(
    source: &str,
    delimiters: &[crate::core::directives::TemplateDelimiter],
) -> bool {
    delimiters.iter().any(|delimiter| {
        if delimiter.open.is_empty() || delimiter.close.is_empty() {
            return false;
        }
        let Some(open) = source.find(&delimiter.open) else {
            return false;
        };
        source[open + delimiter.open.len()..].contains(&delimiter.close)
    })
}

#[derive(Debug, Clone, Copy)]
pub(super) struct QuotedScalarBlock {
    pub(super) full: Span,
    pub(super) value: Span,
    pub(super) trailing_comment: Option<Span>,
}

fn supported_quoted_scalar_block_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> Option<QuotedScalarBlock> {
    let line_info = source.lines.get(line)?;
    let text = source.line_text(line);
    memchr2(b'\'', b'"', text.as_bytes())?;
    let value_start = yaml_line_value_content_start(text)?;
    if let Some(block) = supported_quoted_scalar_block_from_value(
        source,
        line,
        end,
        value_start,
        line_info.full.start(),
    ) {
        return Some(block);
    }

    let indent = indentation(text);
    let marker = sequence_line(text, indent)?;
    let key_start = skip_ascii_whitespace(text, marker + 1);
    let colon = mapping_colon_from(text, key_start)?;
    let value_start = skip_ascii_whitespace(text, colon + 1);
    supported_quoted_scalar_block_from_value(source, line, end, value_start, line_info.full.start())
}

fn supported_quoted_scalar_block_from_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: usize,
    span_start: usize,
) -> Option<QuotedScalarBlock> {
    let block = quoted_scalar_block_from_value(source, line, end, value_start, span_start)?;
    let metadata = scalar_metadata(source, block.value);
    let raw = source.slice(metadata.content).trim_ascii();
    decode_quoted_scalar(raw)?;
    Some(block)
}

pub(super) fn quoted_scalar_block_from_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: usize,
    span_start: usize,
) -> Option<QuotedScalarBlock> {
    let line_info = source.lines.get(line)?;
    let text = source.line_text(line);
    let line_value = split_line_value_comment(text, value_start, line_info.text.start());
    let metadata = scalar_metadata(source, line_value.value);
    let content = trim_span_ascii(source, metadata.content);
    let first = source.as_str().as_bytes().get(content.start).copied()?;
    if !matches!(first, b'\'' | b'"') {
        return None;
    }

    let close = quoted_scalar_close(source.as_str(), content.start, first)?;
    let close_line = source.line_at_byte(close);
    if close_line == line || close_line >= end {
        return None;
    }

    let close_line_info = source.lines[close_line];
    let close_text = source.line_text(close_line);
    let value_end = close + 1;
    let close_local_end = value_end.checked_sub(close_line_info.text.start())?;
    let trailing_comment = trailing_comment_after_flow_close(
        close_text,
        close_local_end,
        close_line_info.text.start(),
    )?;

    Some(QuotedScalarBlock {
        full: Span::new(span_start, close_line_info.full.end()),
        value: Span::new(line_info.text.start() + value_start, value_end),
        trailing_comment,
    })
}

pub(super) fn quoted_scalar_close(text: &str, start: usize, quote: u8) -> Option<usize> {
    let mut pos = start + 1;
    let mut escaped = false;

    while pos < text.len() {
        let ch = text[pos..].chars().next()?;
        let ch_len = ch.len_utf8();
        if quote == b'"' {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                return Some(pos);
            }
            pos += ch_len;
            continue;
        }

        if ch == '\'' {
            let next = pos + ch_len;
            if text[next..].starts_with('\'') {
                pos = next + '\''.len_utf8();
                continue;
            }
            return Some(pos);
        }
        pos += ch_len;
    }

    None
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BlockScalar {
    pub(super) full: Span,
    pub(super) header: Span,
    pub(super) header_info: YamlBlockScalarHeader,
    pub(super) value: Span,
    pub(super) body: Span,
    pub(super) trailing_comment: Option<Span>,
}

pub(super) fn block_scalar_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> Option<BlockScalar> {
    block_scalar_at_impl(source, line, end, None)
}

pub(super) fn block_scalar_at_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: usize,
    base_indent: usize,
) -> Option<BlockScalar> {
    block_scalar_at_impl(source, line, end, Some((value_start, base_indent)))
}

fn block_scalar_at_impl(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: Option<(usize, usize)>,
) -> Option<BlockScalar> {
    let text = source.line_text(line);
    memchr2(b'|', b'>', text.as_bytes())?;
    let trailing_comment = find_trailing_comment(text, 0);
    let content_end = trailing_comment
        .map(|comment_start| trim_end_before(text, 0, comment_start))
        .unwrap_or(text.len());
    let trimmed_end = trim_end_before(text, 0, content_end);
    let trimmed = &text[..trimmed_end];
    let marker_index = trimmed.rfind(['|', '>'])?;
    if !trimmed[marker_index + 1..]
        .chars()
        .all(|ch| matches!(ch, '+' | '-' | '0'..='9' | ' '))
    {
        return None;
    }
    let mut header_info = block_scalar_header_info(&trimmed[marker_index + 1..])?;
    let (value_start, base_indent) = match value_start {
        Some((value_start, base_indent)) if value_start <= marker_index => {
            (value_start, base_indent)
        }
        Some(_) => return None,
        None => block_scalar_value_start(text, marker_index)?,
    };
    if !block_scalar_metadata_prefix(&text[value_start..marker_index]) {
        return None;
    }
    header_info.base_indent = base_indent;
    let mut i = line + 1;
    while i < end {
        let candidate = source.line_text(i);
        if candidate.trim_ascii().is_empty() {
            i += 1;
            continue;
        }
        let indent = candidate.bytes().take_while(|byte| *byte == b' ').count();
        if indent <= base_indent {
            break;
        }
        i += 1;
    }
    let body = if i == line + 1 {
        Span::empty(source.lines[line].full.end())
    } else {
        Span::new(
            source.lines[line].full.end(),
            source.lines[i - 1].full.end(),
        )
    };
    Some(BlockScalar {
        full: Span::new(source.lines[line].full.start(), body.end),
        header: source.lines[line].full.into(),
        header_info,
        value: Span::new(
            source.lines[line].text.start() + value_start,
            source.lines[line].text.start() + trimmed_end,
        ),
        body,
        trailing_comment: trailing_comment.map(|comment_start| {
            Span::new(
                source.lines[line].text.start() + comment_start,
                source.lines[line].text.end(),
            )
        }),
    })
}

fn block_scalar_header_info(indicators: &str) -> Option<YamlBlockScalarHeader> {
    let mut indent = None;
    let mut chomp = YamlBlockChomp::Clip;
    for ch in indicators.trim_ascii().chars() {
        match ch {
            '+' if chomp == YamlBlockChomp::Clip => chomp = YamlBlockChomp::Keep,
            '-' if chomp == YamlBlockChomp::Clip => chomp = YamlBlockChomp::Strip,
            '1'..='9' if indent.is_none() => indent = ch.to_digit(10).map(|value| value as u8),
            '0' => return None,
            _ => return None,
        }
    }
    Some(YamlBlockScalarHeader {
        indent,
        chomp,
        base_indent: 0,
    })
}

fn block_scalar_value_start(text: &str, marker_index: usize) -> Option<(usize, usize)> {
    let indent = indentation(text);
    if let Some(colon) = mapping_colon_at(text, indent)
        && colon < marker_index
    {
        let mut value_start = colon + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some((value_start, indent));
    }
    if let Some(marker) = sequence_line(text, indent)
        && marker < marker_index
    {
        let mut value_start = marker + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some((value_start, indent));
    }
    if let Some(marker) = explicit_key_line(text, indent)
        && marker < marker_index
    {
        let mut value_start = marker + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some((value_start, indent));
    }
    Some((indent, indent))
}

fn block_scalar_metadata_prefix(prefix: &str) -> bool {
    let prefix = prefix.trim_ascii();
    prefix.is_empty()
        || prefix
            .split_whitespace()
            .all(|token| token.starts_with('!') || token.starts_with('&'))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FlowCollectionBlock {
    pub(super) span: Span,
    pub(super) value: Span,
    pub(super) collection: Span,
    pub(super) trailing_comment: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FlowCollapseHint {
    pub(super) opener: Span,
    pub(super) rest: FlowCollapseRest,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum FlowCollapseRest {
    Empty,
    InlineMapping { key_start: usize },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum FlowCollectionScan {
    Complete(FlowCollectionBlock),
    Incomplete,
}

impl FlowCollectionScan {
    pub(super) fn complete(self) -> Option<FlowCollectionBlock> {
        match self {
            Self::Complete(block) => Some(block),
            Self::Incomplete => None,
        }
    }
}

pub(super) fn flow_collapse_hint_from_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: usize,
    allow_inline_mapping: bool,
) -> Option<FlowCollapseHint> {
    let line_info = source.lines.get(line)?;
    let value = Span::new(line_info.text.start() + value_start, line_info.text.end());
    let metadata = scalar_metadata(source, value);
    if metadata.tag.is_some() || metadata.anchor.is_some() {
        return None;
    }
    let opener_column = metadata.content.start.checked_sub(line_info.text.start())?;
    let opener = flow_collapse_hint_at(source, line, end, opener_column, allow_inline_mapping)?;
    Some(opener)
}

pub(super) fn line_has_flow_collapse_hint_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> bool {
    if line >= end {
        return false;
    }
    let text = source.line_text(line);
    let indent = indentation(text);
    let value_start = if let Some(marker) = sequence_line(text, indent) {
        skip_ascii_whitespace(text, marker + 1)
    } else if let Some(colon) = mapping_colon_at(text, indent) {
        skip_ascii_whitespace(text, colon + 1)
    } else if let Some(colon) = explicit_value_line(text, indent) {
        skip_ascii_whitespace(text, colon + 1)
    } else {
        return false;
    };
    flow_collapse_hint_from_value(source, line, end, value_start, true).is_some()
}

pub(super) fn flow_collapse_hint_from_standalone_opener(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    min_indent: usize,
) -> Option<FlowCollapseHint> {
    if line >= end {
        return None;
    }
    let text = source.line_text(line);
    let indent = indentation(text);
    if indent < min_indent {
        return None;
    }
    let hint = flow_collapse_hint_at(source, line, end, indent, false)?;
    matches!(hint.rest, FlowCollapseRest::Empty).then_some(hint)
}

fn flow_collapse_hint_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    opener_column: usize,
    allow_inline_mapping: bool,
) -> Option<FlowCollapseHint> {
    let line_info = source.lines.get(line)?;
    let text = source.line_text(line);
    let opener_byte = text.as_bytes().get(opener_column).copied()?;
    if !matches!(opener_byte, b'[' | b'{') {
        return None;
    }
    if matches!(
        flow_collection_block_from_value(source, line, end, opener_column, line_info.full.start()),
        Some(FlowCollectionScan::Complete(_))
    ) {
        return None;
    }

    let after_opener = skip_ascii_whitespace(text, opener_column + 1);
    let rest = if after_opener == text.len() {
        flow_collapse_hint_next_line_is_block_collection(source, line + 1, end)?;
        FlowCollapseRest::Empty
    } else if opener_byte == b'{'
        && allow_inline_mapping
        && mapping_colon_from(text, after_opener).is_some()
    {
        FlowCollapseRest::InlineMapping {
            key_start: after_opener,
        }
    } else {
        return None;
    };
    Some(FlowCollapseHint {
        opener: Span::new(
            line_info.text.start() + opener_column,
            line_info.text.start() + opener_column + 1,
        ),
        rest,
    })
}

fn flow_collapse_hint_next_line_is_block_collection(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> Option<()> {
    if line >= end {
        return None;
    }
    let text = source.line_text(line);
    if text.trim_ascii().is_empty()
        || text.trim_ascii_start().starts_with('#')
        || tab_in_indentation(text).is_some()
    {
        return None;
    }
    let indent = indentation(text);
    (sequence_line(text, indent).is_some()
        || mapping_colon_at(text, indent).is_some()
        || explicit_key_line(text, indent).is_some())
    .then_some(())
}

fn flow_collection_block_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    collection_start: usize,
    span_start: usize,
) -> Option<FlowCollectionScan> {
    let first_line = source.lines.get(line)?;
    let first_text = source.line_text(line);
    if !matches!(
        first_text.as_bytes().get(collection_start),
        Some(b'[' | b'{')
    ) {
        return None;
    }

    let mut depth = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for i in line..end {
        let line_info = source.lines[i];
        let text = source.line_text(i);
        let scan_start = if i == line { collection_start } else { 0 };
        let bytes = text.as_bytes();
        let mut index = scan_start;

        while index < bytes.len() {
            let byte = bytes[index];
            if in_double {
                if escaped {
                    escaped = false;
                    index += 1;
                    continue;
                }
                if byte == b'\\' {
                    escaped = true;
                    index += 1;
                    continue;
                }
                if byte == b'"' {
                    in_double = false;
                }
                index += 1;
                continue;
            }
            if in_single {
                if byte == b'\'' {
                    if bytes.get(index + 1) == Some(&b'\'') {
                        index += 2;
                        continue;
                    } else {
                        in_single = false;
                    }
                }
                index += 1;
                continue;
            }

            match byte {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b'#' if depth > 0 && comment_can_start(text, scan_start, index) => break,
                b'[' | b'{' => depth += 1,
                b']' | b'}' if depth > 0 => {
                    depth -= 1;
                    if depth == 0 {
                        let value_end = index + 1;
                        let trailing_comment = trailing_comment_after_flow_close(
                            text,
                            value_end,
                            line_info.text.start(),
                        )?;
                        return Some(FlowCollectionScan::Complete(FlowCollectionBlock {
                            span: Span::new(span_start, line_info.full.end()),
                            value: Span::new(
                                first_line.text.start() + collection_start,
                                line_info.text.start() + value_end,
                            ),
                            collection: Span::new(
                                first_line.text.start() + collection_start,
                                line_info.text.start() + value_end,
                            ),
                            trailing_comment,
                        }));
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }

    Some(FlowCollectionScan::Incomplete)
}

pub(super) fn flow_collection_block_from_value(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    value_start: usize,
    span_start: usize,
) -> Option<FlowCollectionScan> {
    let line_info = source.lines.get(line)?;
    let text = source.line_text(line);
    if value_start >= text.len() {
        return None;
    }
    match text.as_bytes()[value_start] {
        b'[' | b'{' => {
            return flow_collection_block_at(source, line, end, value_start, span_start).map(
                |scan| match scan {
                    FlowCollectionScan::Complete(mut block) => {
                        block.value =
                            Span::new(line_info.text.start() + value_start, block.collection.end);
                        FlowCollectionScan::Complete(block)
                    }
                    FlowCollectionScan::Incomplete => FlowCollectionScan::Incomplete,
                },
            );
        }
        b'!' | b'&' => {}
        byte if byte.is_ascii_whitespace() => {}
        _ => return None,
    }
    let line_value = split_line_value_comment(text, value_start, line_info.text.start());
    let metadata = scalar_metadata(source, line_value.value);
    let collection_start = metadata.content.start.checked_sub(line_info.text.start())?;
    let scan = flow_collection_block_at(source, line, end, collection_start, span_start)?;
    Some(match scan {
        FlowCollectionScan::Complete(mut block) => {
            block.value = Span::new(line_info.text.start() + value_start, block.collection.end);
            FlowCollectionScan::Complete(block)
        }
        FlowCollectionScan::Incomplete => FlowCollectionScan::Incomplete,
    })
}

fn trailing_comment_after_flow_close(
    text: &str,
    value_end: usize,
    line_start: usize,
) -> Option<Option<Span>> {
    let mut cursor = value_end;
    while cursor < text.len() {
        let ch = text[cursor..].chars().next()?;
        if ch == '#' && comment_can_start(text, value_end, cursor) {
            return Some(Some(Span::new(
                line_start + cursor,
                line_start + text.len(),
            )));
        }
        if !ch.is_whitespace() {
            return None;
        }
        cursor += ch.len_utf8();
    }
    Some(None)
}

pub(super) fn line_flow_value_start(text: &str) -> Option<usize> {
    let indent = indentation(text);
    let value_start = if let Some(marker) = sequence_line(text, indent) {
        skip_ascii_whitespace(text, marker + 1)
    } else if let Some(colon) = mapping_colon_at(text, indent) {
        skip_ascii_whitespace(text, colon + 1)
    } else {
        indent
    };
    let collection_start = node_properties_content_start(text, value_start);
    matches!(text.as_bytes().get(collection_start), Some(b'[' | b'{')).then_some(value_start)
}

pub(super) fn skip_ascii_whitespace(text: &str, mut cursor: usize) -> usize {
    while cursor < text.len() && text.as_bytes()[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn node_properties_content_start(text: &str, mut cursor: usize) -> usize {
    while cursor < text.len() {
        cursor = skip_ascii_whitespace(text, cursor);
        let token_start = cursor;
        while cursor < text.len() && !text.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let token = &text[token_start..cursor];
        if token.starts_with('!') || token.starts_with('&') {
            continue;
        }
        return token_start;
    }
    cursor
}

pub(super) fn unsupported_flow_block_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> Option<Span> {
    let start = source.lines[line].full.start();
    let mut depth = 0usize;
    let mut saw_flow = false;
    let mut unsupported = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut i = line;

    while i < end {
        let text = source.line_text(i);
        for ch in text.chars() {
            if in_double {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    in_double = false;
                }
                continue;
            }
            if in_single {
                if ch == '\'' {
                    in_single = false;
                }
                continue;
            }

            match ch {
                '\'' => in_single = true,
                '"' => in_double = true,
                '#' if depth > 0 => {
                    unsupported = true;
                    break;
                }
                '[' | '{' => {
                    saw_flow = true;
                    depth += 1;
                    if i > line {
                        unsupported = true;
                    }
                }
                ']' | '}' if depth > 0 => {
                    depth -= 1;
                    if i > line {
                        unsupported = true;
                    }
                    if depth == 0 && saw_flow && (i > line || unsupported) {
                        return Some(Span::new(start, source.lines[i].full.end()));
                    }
                }
                _ => {}
            }
        }

        if saw_flow && depth == 0 {
            return (i > line || unsupported).then(|| Span::new(start, source.lines[i].full.end()));
        }
        if !saw_flow {
            return None;
        }
        if saw_flow && i > line {
            unsupported = true;
        }
        i += 1;
    }

    (saw_flow && (depth > 0 || unsupported))
        .then(|| Span::new(start, source.lines[end.saturating_sub(1)].full.end()))
}

pub(super) fn yaml_error_at(
    source: &SourceBuffer,
    byte: usize,
    message: impl Into<String>,
) -> YamarkError {
    let (line, column) = source.line_column_at_byte(byte);
    YamarkError::at(message, line, column)
}
