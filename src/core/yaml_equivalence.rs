use std::borrow::Cow;

use crate::core::document::{Document, DocumentKind};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlAstNode, YamlBlockChomp, YamlDocumentAst, YamlEmitPlan, YamlFlowMapping,
    YamlFlowSequence, YamlMapping, YamlNodeId, YamlRenderedKind, YamlScalar, YamlScalarSemantic,
    YamlScalarStyle, YamlSequence,
};
use crate::diagnostic::{Result, YamarkError};

const INVALID_OUTPUT: &str = "formatter produced invalid YAML";
const CHANGED_VALUE: &str = "formatter changed the YAML value";

pub(crate) fn validate_yaml_documents_equivalent(
    before_source: &SourceBuffer,
    before_document: &Document<'_>,
    after_source: &SourceBuffer,
    after_document: &Document<'_>,
) -> Result<()> {
    let mut before_documents = Vec::new();
    let mut after_documents = Vec::new();
    collect_yaml_documents(before_document, before_source, &mut before_documents);
    collect_yaml_documents(after_document, after_source, &mut after_documents);

    if before_documents.len() != after_documents.len() {
        return Err(YamarkError::new(CHANGED_VALUE));
    }

    for document in &after_documents {
        if document
            .source
            .slice(document.document.range)
            .chars()
            .any(yaml_forbidden_character)
        {
            return Err(YamarkError::new(INVALID_OUTPUT));
        }
    }

    for (before, after) in before_documents.into_iter().zip(after_documents) {
        if !yaml_documents_equivalent(before, after)? {
            return Err(YamarkError::new(CHANGED_VALUE));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct YamlDocumentRef<'doc, 'src> {
    source: &'doc SourceBuffer,
    document: &'doc Document<'src>,
}

fn collect_yaml_documents<'doc, 'src>(
    document: &'doc Document<'src>,
    inherited_source: &'doc SourceBuffer,
    documents: &mut Vec<YamlDocumentRef<'doc, 'src>>,
) {
    let source = document.source.as_ref().unwrap_or(inherited_source);
    if document.kind == DocumentKind::Yaml {
        documents.push(YamlDocumentRef { source, document });
    }
    for nested in &document.nested {
        collect_yaml_documents(nested, source, documents);
    }
}

fn yaml_forbidden_character(ch: char) -> bool {
    matches!(
        ch,
        '\u{0000}'..='\u{0008}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000e}'..='\u{001f}'
            | '\u{007f}'..='\u{0084}'
            | '\u{0086}'..='\u{009f}'
            | '\u{fffe}'
            | '\u{ffff}'
    )
}

#[derive(Clone, Copy)]
struct YamlContext<'doc, 'src> {
    source: &'doc SourceBuffer,
    ast: &'doc YamlDocumentAst<'src>,
}

fn yaml_documents_equivalent(
    before_document: YamlDocumentRef<'_, '_>,
    after_document: YamlDocumentRef<'_, '_>,
) -> Result<bool> {
    let (Some(before_ast), Some(after_ast)) = (
        before_document.document.yaml.as_ref(),
        after_document.document.yaml.as_ref(),
    ) else {
        return Ok(before_document.document.yaml.is_none()
            && after_document.document.yaml.is_none()
            && before_document.source.slice(before_document.document.range)
                == after_document.source.slice(after_document.document.range));
    };
    if before_ast.roots.len() != after_ast.roots.len() {
        return Ok(false);
    }

    let before = YamlContext {
        source: before_document.source,
        ast: before_ast,
    };
    let after = YamlContext {
        source: after_document.source,
        ast: after_ast,
    };
    for (before_root, after_root) in before_ast.roots.iter().zip(&after_ast.roots) {
        if !optional_nodes_equivalent(before, before_root.node, 0, after, after_root.node, 0)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn optional_nodes_equivalent(
    before: YamlContext<'_, '_>,
    before_id: Option<YamlNodeId>,
    before_parent_indent: usize,
    after: YamlContext<'_, '_>,
    after_id: Option<YamlNodeId>,
    after_parent_indent: usize,
) -> Result<bool> {
    match (before_id, after_id) {
        (Some(before_id), Some(after_id)) => nodes_equivalent(
            before,
            before_id,
            before_parent_indent,
            after,
            after_id,
            after_parent_indent,
        ),
        (None, Some(after_id)) => Ok(node_is_unadorned_null(after, after_id)),
        (Some(before_id), None) => Ok(node_is_unadorned_null(before, before_id)),
        (None, None) => Ok(true),
    }
}

fn nodes_equivalent(
    before: YamlContext<'_, '_>,
    before_id: YamlNodeId,
    before_parent_indent: usize,
    after: YamlContext<'_, '_>,
    after_id: YamlNodeId,
    after_parent_indent: usize,
) -> Result<bool> {
    let before_node = before.ast.node(before_id);
    let after_node = after.ast.node(after_id);

    if node_is_unadorned_null(before, before_id) && node_is_unadorned_null(after, after_id) {
        return Ok(true);
    }
    if matches!(
        before_node.emit,
        YamlEmitPlan::Rendered(YamlRenderedKind::EmptyMarkdownScalar)
    ) {
        let (YamlAstKind::Scalar(before_scalar), YamlAstKind::Scalar(after_scalar)) =
            (&before_node.kind, &after_node.kind)
        else {
            return Ok(false);
        };
        return Ok(optional_span_text_equal(
            before.source,
            before_scalar.tag,
            after.source,
            after_scalar.tag,
        ) && optional_span_text_equal(
            before.source,
            before_scalar.anchor,
            after.source,
            after_scalar.anchor,
        ) && scalar_is_string_like(after_scalar)
            && comparable_scalar_value(after, after_scalar, after_parent_indent)
                .is_some_and(|value| comparable_scalar_is_empty_string(&value)));
    }
    if let (YamlAstKind::Scalar(before_scalar), YamlAstKind::Scalar(after_scalar)) =
        (&before_node.kind, &after_node.kind)
    {
        return scalars_equivalent(
            before,
            before_node,
            before_scalar,
            before_parent_indent,
            after,
            after_scalar,
            after_parent_indent,
        );
    }
    if let (Some(before_sequence), Some(after_sequence)) = (
        SequenceRef::from_kind(&before_node.kind),
        SequenceRef::from_kind(&after_node.kind),
    ) {
        return sequences_equivalent(before, before_sequence, after, after_sequence);
    }
    if let (Some(before_mapping), Some(after_mapping)) = (
        MappingRef::from_kind(&before_node.kind),
        MappingRef::from_kind(&after_node.kind),
    ) {
        return mappings_equivalent(before, before_mapping, after, after_mapping);
    }

    Ok(match (&before_node.kind, &after_node.kind) {
        (YamlAstKind::Empty, YamlAstKind::Empty) => true,
        (YamlAstKind::Alias(before_alias), YamlAstKind::Alias(after_alias)) => {
            before.source.slice(before_alias.value).trim_ascii()
                == after.source.slice(after_alias.value).trim_ascii()
        }
        (YamlAstKind::Opaque(_), YamlAstKind::Opaque(_)) => {
            before.source.slice(before_node.span) == after.source.slice(after_node.span)
        }
        _ => false,
    })
}

fn node_is_unadorned_null(context: YamlContext<'_, '_>, id: YamlNodeId) -> bool {
    match &context.ast.node(id).kind {
        YamlAstKind::Empty => true,
        YamlAstKind::Scalar(scalar) => {
            scalar.semantic == YamlScalarSemantic::Null
                && scalar.anchor.is_none()
                && scalar
                    .tag
                    .is_none_or(|tag| context.source.slice(tag) == "!!null")
        }
        _ => false,
    }
}

fn scalars_equivalent(
    before: YamlContext<'_, '_>,
    before_node: &YamlAstNode<'_>,
    before_scalar: &YamlScalar<'_>,
    before_parent_indent: usize,
    after: YamlContext<'_, '_>,
    after_scalar: &YamlScalar<'_>,
    after_parent_indent: usize,
) -> Result<bool> {
    if !optional_span_text_equal(
        before.source,
        before_scalar.tag,
        after.source,
        after_scalar.tag,
    ) || !optional_span_text_equal(
        before.source,
        before_scalar.anchor,
        after.source,
        after_scalar.anchor,
    ) {
        return Ok(false);
    }

    let raw_equal = before_scalar.style == after_scalar.style
        && before_scalar.semantic == after_scalar.semantic
        && before_source_scalar_text(before, before_scalar)
            == before_source_scalar_text(after, after_scalar)
        && optional_span_text_equal(
            before.source,
            before_scalar.body,
            after.source,
            after_scalar.body,
        )
        && (before_scalar.body.is_none() || before_parent_indent == after_parent_indent);
    if raw_equal {
        return Ok(true);
    }

    let after_value = comparable_scalar_value(after, after_scalar, after_parent_indent)
        .ok_or_else(|| YamarkError::new(INVALID_OUTPUT))?;
    if scalar_content_may_change(before_node, before_scalar, after_scalar) {
        return Ok(true);
    }
    let before_value = comparable_scalar_value(before, before_scalar, before_parent_indent)
        .ok_or_else(|| YamarkError::new(CHANGED_VALUE))?;
    Ok(before_value == after_value)
}

fn scalar_content_may_change(
    node: &YamlAstNode<'_>,
    before: &YamlScalar<'_>,
    after: &YamlScalar<'_>,
) -> bool {
    match node.emit {
        YamlEmitPlan::NestedMarkdownBlockScalar { .. }
        | YamlEmitPlan::ExternalBlockScalar
        | YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar) => {
            scalar_is_string_like(before) && scalar_is_string_like(after)
        }
        _ => false,
    }
}

fn scalar_is_string_like(scalar: &YamlScalar<'_>) -> bool {
    scalar.semantic == YamlScalarSemantic::String
        || scalar.semantic == YamlScalarSemantic::Unknown && scalar.tag.is_some()
}

fn before_source_scalar_text<'a>(context: YamlContext<'a, '_>, scalar: &YamlScalar<'_>) -> &'a str {
    let start = [scalar.tag, scalar.anchor]
        .into_iter()
        .flatten()
        .map(SourceSpan::end)
        .max()
        .unwrap_or_else(|| scalar.value.start());
    context
        .source
        .slice(Span::new(start.min(scalar.value.end()), scalar.value.end()))
        .trim_ascii()
}

#[derive(Debug, PartialEq, Eq)]
enum ComparableScalar<'a> {
    String(Cow<'a, str>),
    Null,
    Boolean(bool),
    Integer(Cow<'a, str>),
    Float(Cow<'a, str>),
    Unknown(Cow<'a, str>),
}

fn comparable_scalar_is_empty_string(value: &ComparableScalar<'_>) -> bool {
    matches!(
        value,
        ComparableScalar::String(value) | ComparableScalar::Unknown(value) if value.is_empty()
    )
}

fn comparable_scalar_value<'a>(
    context: YamlContext<'a, '_>,
    scalar: &YamlScalar<'_>,
    parent_indent: usize,
) -> Option<ComparableScalar<'a>> {
    let decoded = if scalar.body.is_some() {
        Cow::Owned(decode_block_scalar(context, scalar, parent_indent)?)
    } else {
        let raw = before_source_scalar_text(context, scalar);
        match scalar.style {
            YamlScalarStyle::Plain => {
                if raw.contains('\u{feff}') {
                    return None;
                }
                decode_plain_scalar(raw)
            }
            YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
                decode_quoted_scalar(raw)?
            }
            YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => return None,
        }
    };

    Some(match scalar.semantic {
        YamlScalarSemantic::String => ComparableScalar::String(decoded),
        YamlScalarSemantic::Null => ComparableScalar::Null,
        YamlScalarSemantic::Boolean => ComparableScalar::Boolean(yaml_bool(decoded.as_ref())?),
        YamlScalarSemantic::Integer => ComparableScalar::Integer(decoded),
        YamlScalarSemantic::Float => ComparableScalar::Float(decoded),
        YamlScalarSemantic::Unknown => ComparableScalar::Unknown(decoded),
    })
}

fn decode_plain_scalar(raw: &str) -> Cow<'_, str> {
    if !raw.contains(['\r', '\n']) {
        return Cow::Borrowed(raw);
    }
    Cow::Owned(decode_multiline_plain_scalar(raw))
}

fn decode_multiline_plain_scalar(raw: &str) -> String {
    let normalized = normalize_line_endings(raw);
    let mut out = String::new();
    let mut blank_lines = 0usize;
    let mut has_content = false;
    for line in normalized.lines().map(str::trim_ascii) {
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
    out
}

fn decode_quoted_scalar(raw: &str) -> Option<Cow<'_, str>> {
    if raw.starts_with('"') {
        let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
        if !inner.contains(['\\', '\r', '\n']) {
            Some(Cow::Borrowed(inner))
        } else {
            decode_double_quoted_scalar(raw).map(Cow::Owned)
        }
    } else if raw.starts_with('\'') {
        let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
        if !inner.contains(['\'', '\r', '\n']) {
            Some(Cow::Borrowed(inner))
        } else {
            decode_single_quoted_scalar(raw).map(Cow::Owned)
        }
    } else {
        None
    }
}

fn decode_single_quoted_scalar(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut out = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.next() == Some('\'') {
                out.push('\'');
            } else {
                return None;
            }
        } else if matches!(ch, '\r' | '\n') {
            return None;
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

fn decode_double_quoted_scalar(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            if matches!(ch, '\r' | '\n') {
                return None;
            }
            out.push(ch);
            continue;
        }
        match chars.next()? {
            '0' => out.push('\0'),
            'a' => out.push('\u{0007}'),
            'b' => out.push('\u{0008}'),
            't' | '\t' => out.push('\t'),
            'n' => out.push('\n'),
            'v' => out.push('\u{000b}'),
            'f' => out.push('\u{000c}'),
            'r' => out.push('\r'),
            'e' => out.push('\u{001b}'),
            '"' => out.push('"'),
            '/' => out.push('/'),
            '\\' => out.push('\\'),
            'x' => out.push(decode_hex_escape(&mut chars, 2)?),
            'u' => out.push(decode_hex_escape(&mut chars, 4)?),
            'U' => out.push(decode_hex_escape(&mut chars, 8)?),
            _ => return None,
        }
    }
    Some(out)
}

fn decode_hex_escape(chars: &mut std::str::Chars<'_>, digits: usize) -> Option<char> {
    let mut value = 0u32;
    for _ in 0..digits {
        value = value.checked_mul(16)?;
        value += chars.next()?.to_digit(16)?;
    }
    char::from_u32(value)
}

fn decode_block_scalar(
    context: YamlContext<'_, '_>,
    scalar: &YamlScalar<'_>,
    parent_indent: usize,
) -> Option<String> {
    let body = normalize_line_endings(context.source.slice(scalar.body?));
    if body.contains('\u{feff}') {
        return None;
    }
    let header = scalar.block_header?;
    let source_lines = body.split('\n').collect::<Vec<_>>();
    let first_content_line = source_lines
        .iter()
        .position(|line| !line.trim_matches(' ').is_empty());
    let content_indent = match header.indent {
        Some(indent) => parent_indent.checked_add(indent as usize)?,
        None => first_content_line
            .map(|index| leading_spaces(source_lines[index]))
            .unwrap_or_else(|| {
                source_lines
                    .iter()
                    .map(|line| leading_spaces(line))
                    .max()
                    .unwrap_or(parent_indent.saturating_add(1))
            }),
    };
    let invalid_content_indent = first_content_line.is_some() && content_indent <= parent_indent;
    let over_indented_leading_blank = header.indent.is_none()
        && first_content_line.is_some_and(|index| {
            source_lines[..index]
                .iter()
                .any(|line| leading_spaces(line) > content_indent)
        });
    if invalid_content_indent || over_indented_leading_blank {
        return None;
    }

    let mut lines = Vec::with_capacity(source_lines.len());
    for line in source_lines {
        let indent = leading_spaces(line);
        let strip = indent.min(content_indent);
        let text = &line[strip..];
        if indent < content_indent && !text.is_empty() {
            return None;
        }
        lines.push(BlockLine {
            text: text.to_owned(),
            more_indented: text.starts_with([' ', '\t']),
        });
    }

    let mut value = match scalar.style {
        YamlScalarStyle::LiteralBlock => lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        YamlScalarStyle::FoldedBlock => fold_block_lines(&lines),
        YamlScalarStyle::Plain | YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            return None;
        }
    };
    apply_block_chomp(&mut value, header.chomp);
    Some(value)
}

struct BlockLine {
    text: String,
    more_indented: bool,
}

fn fold_block_lines(lines: &[BlockLine]) -> String {
    let mut out = String::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = &lines[index];
        if line.text.is_empty() {
            let start = index;
            while index < lines.len() && lines[index].text.is_empty() {
                index += 1;
            }
            let blank_lines = index - start;
            let newlines = if index == lines.len() {
                blank_lines.saturating_sub(1)
            } else {
                blank_lines
            };
            out.extend(std::iter::repeat_n('\n', newlines));
            continue;
        }

        out.push_str(&line.text);
        index += 1;
        let blank_start = index;
        while index < lines.len() && lines[index].text.is_empty() {
            index += 1;
        }
        let blank_lines = index - blank_start;
        let Some(next) = lines.get(index) else {
            out.extend(std::iter::repeat_n('\n', blank_lines));
            break;
        };
        if blank_lines == 0 {
            if line.more_indented || next.more_indented {
                out.push('\n');
            } else {
                out.push(' ');
            }
        } else {
            let newlines = blank_lines + usize::from(line.more_indented || next.more_indented);
            out.extend(std::iter::repeat_n('\n', newlines));
        }
    }
    out
}

fn apply_block_chomp(value: &mut String, chomp: YamlBlockChomp) {
    match chomp {
        YamlBlockChomp::Strip => {
            value.truncate(value.trim_end_matches('\n').len());
        }
        YamlBlockChomp::Clip => {
            let content_end = value.trim_end_matches('\n').len();
            let has_final_line_break = content_end < value.len();
            value.truncate(content_end);
            if has_final_line_break && !value.is_empty() {
                value.push('\n');
            }
        }
        YamlBlockChomp::Keep => {}
    }
}

fn normalize_line_endings(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn leading_spaces(value: &str) -> usize {
    value.bytes().take_while(|byte| *byte == b' ').count()
}

fn yaml_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "True" | "TRUE" => Some(true),
        "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum SequenceRef<'a, 'src> {
    Block(&'a YamlSequence<'src>),
    Flow(&'a YamlFlowSequence<'src>),
}

impl<'a, 'src> SequenceRef<'a, 'src> {
    fn from_kind(kind: &'a YamlAstKind<'src>) -> Option<Self> {
        match kind {
            YamlAstKind::Sequence(sequence) => Some(Self::Block(sequence)),
            YamlAstKind::FlowSequence(sequence) => Some(Self::Flow(sequence)),
            _ => None,
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Block(sequence) => sequence.items.len(),
            Self::Flow(sequence) => sequence.entries.len(),
        }
    }

    fn item(self, index: usize) -> Option<YamlNodeId> {
        match self {
            Self::Block(sequence) => sequence.items[index].value,
            Self::Flow(sequence) => Some(sequence.entries[index]),
        }
    }

    fn indent(self) -> usize {
        match self {
            Self::Block(sequence) => sequence.indent,
            Self::Flow(_) => 0,
        }
    }

    fn tag(self) -> Option<SourceSpan<'src>> {
        match self {
            Self::Block(sequence) => sequence.tag,
            Self::Flow(sequence) => sequence.tag,
        }
    }

    fn anchor(self) -> Option<SourceSpan<'src>> {
        match self {
            Self::Block(sequence) => sequence.anchor,
            Self::Flow(sequence) => sequence.anchor,
        }
    }
}

fn sequences_equivalent(
    before: YamlContext<'_, '_>,
    before_sequence: SequenceRef<'_, '_>,
    after: YamlContext<'_, '_>,
    after_sequence: SequenceRef<'_, '_>,
) -> Result<bool> {
    if before_sequence.len() != after_sequence.len()
        || !collection_tag_equal(
            before.source,
            before_sequence.tag(),
            "!!seq",
            after.source,
            after_sequence.tag(),
        )
        || !optional_span_text_equal(
            before.source,
            before_sequence.anchor(),
            after.source,
            after_sequence.anchor(),
        )
    {
        return Ok(false);
    }
    for index in 0..before_sequence.len() {
        if !optional_nodes_equivalent(
            before,
            before_sequence.item(index),
            before_sequence.indent(),
            after,
            after_sequence.item(index),
            after_sequence.indent(),
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Clone, Copy)]
enum MappingRef<'a, 'src> {
    Block(&'a YamlMapping<'src>),
    Flow(&'a YamlFlowMapping<'src>),
}

impl<'a, 'src> MappingRef<'a, 'src> {
    fn from_kind(kind: &'a YamlAstKind<'src>) -> Option<Self> {
        match kind {
            YamlAstKind::Mapping(mapping) => Some(Self::Block(mapping)),
            YamlAstKind::FlowMapping(mapping) => Some(Self::Flow(mapping)),
            _ => None,
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Block(mapping) => mapping.pairs.len(),
            Self::Flow(mapping) => mapping.pairs.len(),
        }
    }

    fn pair(self, index: usize) -> (Option<YamlNodeId>, Option<YamlNodeId>) {
        match self {
            Self::Block(mapping) => {
                let pair = &mapping.pairs[index];
                (pair.key_node, pair.value)
            }
            Self::Flow(mapping) => {
                let pair = &mapping.pairs[index];
                (Some(pair.key), pair.value)
            }
        }
    }

    fn indent(self) -> usize {
        match self {
            Self::Block(mapping) => mapping.indent,
            Self::Flow(_) => 0,
        }
    }

    fn tag(self) -> Option<SourceSpan<'src>> {
        match self {
            Self::Block(mapping) => mapping.tag,
            Self::Flow(mapping) => mapping.tag,
        }
    }

    fn anchor(self) -> Option<SourceSpan<'src>> {
        match self {
            Self::Block(mapping) => mapping.anchor,
            Self::Flow(mapping) => mapping.anchor,
        }
    }
}

fn mappings_equivalent(
    before: YamlContext<'_, '_>,
    before_mapping: MappingRef<'_, '_>,
    after: YamlContext<'_, '_>,
    after_mapping: MappingRef<'_, '_>,
) -> Result<bool> {
    if before_mapping.len() != after_mapping.len()
        || !collection_tag_equal(
            before.source,
            before_mapping.tag(),
            "!!map",
            after.source,
            after_mapping.tag(),
        )
        || !optional_span_text_equal(
            before.source,
            before_mapping.anchor(),
            after.source,
            after_mapping.anchor(),
        )
    {
        return Ok(false);
    }
    for index in 0..before_mapping.len() {
        let (before_key, before_value) = before_mapping.pair(index);
        let (after_key, after_value) = after_mapping.pair(index);
        let keys_equivalent = optional_nodes_equivalent(
            before,
            before_key,
            before_mapping.indent(),
            after,
            after_key,
            after_mapping.indent(),
        )?;
        let values_equivalent = optional_nodes_equivalent(
            before,
            before_value,
            before_mapping.indent(),
            after,
            after_value,
            after_mapping.indent(),
        )?;
        if !keys_equivalent || !values_equivalent {
            return Ok(false);
        }
    }
    Ok(true)
}

fn collection_tag_equal(
    before_source: &SourceBuffer,
    before: Option<SourceSpan<'_>>,
    removable: &str,
    after_source: &SourceBuffer,
    after: Option<SourceSpan<'_>>,
) -> bool {
    let before = before
        .map(|span| before_source.slice(span))
        .filter(|tag| *tag != removable);
    let after = after
        .map(|span| after_source.slice(span))
        .filter(|tag| *tag != removable);
    before == after
}

fn optional_span_text_equal(
    before_source: &SourceBuffer,
    before: Option<SourceSpan<'_>>,
    after_source: &SourceBuffer,
    after: Option<SourceSpan<'_>>,
) -> bool {
    before.map(|span| before_source.slice(span)) == after.map(|span| after_source.slice(span))
}
