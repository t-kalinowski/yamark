use std::borrow::Cow;
use std::ops::Range;

use crate::core::document::{Document, DocumentKind};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlBlockChomp, YamlBlockScalarHeader, YamlDocumentAst, YamlEmitPlan,
    YamlFlowMapping, YamlFlowSequence, YamlMapping, YamlNodeId, YamlRenderedKind, YamlScalar,
    YamlScalarSemantic, YamlScalarStyle, YamlSequence,
};
use crate::diagnostic::{Result, YamarkError};

const INVALID_OUTPUT: &str = "formatter produced invalid YAML";
const CHANGED_VALUE: &str = "formatter changed the YAML value";

pub(crate) struct YamlValidationSnapshot {
    sources: Vec<String>,
    documents: Vec<BeforeYamlDocument>,
    roots: Vec<Option<YamlNodeId>>,
    nodes: Vec<BeforeYamlNode>,
    children: Vec<Option<YamlNodeId>>,
    pairs: Vec<BeforeYamlPair>,
    root_yaml_node_capacity: Option<usize>,
}

impl YamlValidationSnapshot {
    pub(crate) fn node_capacity_hint_for_root_yaml(&self) -> Option<usize> {
        self.root_yaml_node_capacity
    }
}

struct BeforeYamlDocument {
    source: usize,
    range: Span,
    roots: Option<Range<usize>>,
}

struct BeforeYamlNode {
    kind: BeforeYamlKind,
}

enum BeforeYamlKind {
    Empty,
    Scalar(BeforeYamlScalar),
    Sequence(BeforeYamlSequence),
    Mapping(BeforeYamlMapping),
    Alias(SourceSpan<'static>),
    Opaque(SourceSpan<'static>),
}

struct BeforeYamlScalar {
    style: YamlScalarStyle,
    semantic: YamlScalarSemantic,
    value: SourceSpan<'static>,
    block_header: Option<YamlBlockScalarHeader>,
    body: Option<SourceSpan<'static>>,
    tag: Option<SourceSpan<'static>>,
    anchor: Option<SourceSpan<'static>>,
    emit_rule: ScalarEmitRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScalarEmitRule {
    Exact,
    ContentMayChange,
    EmptyMarkdown,
}

struct BeforeYamlSequence {
    indent: usize,
    children: Range<usize>,
    tag: Option<SourceSpan<'static>>,
    anchor: Option<SourceSpan<'static>>,
}

struct BeforeYamlMapping {
    indent: usize,
    pairs: Range<usize>,
    tag: Option<SourceSpan<'static>>,
    anchor: Option<SourceSpan<'static>>,
}

struct BeforeYamlPair {
    key: Option<YamlNodeId>,
    value: Option<YamlNodeId>,
}

pub(crate) fn capture_yaml_validation_snapshot(
    root_source: String,
    root_document: Document<'static>,
) -> YamlValidationSnapshot {
    let root_yaml_node_capacity = (root_document.kind == DocumentKind::Yaml)
        .then(|| root_document.yaml.as_ref().map(|ast| ast.nodes.len()))
        .flatten();
    let mut builder = YamlValidationSnapshotBuilder::new(root_source);
    builder.capture_document(root_document, 0);
    builder.finish(root_yaml_node_capacity)
}

struct YamlValidationSnapshotBuilder {
    sources: Vec<String>,
    documents: Vec<BeforeYamlDocument>,
    roots: Vec<Option<YamlNodeId>>,
    nodes: Vec<BeforeYamlNode>,
    children: Vec<Option<YamlNodeId>>,
    pairs: Vec<BeforeYamlPair>,
}

impl YamlValidationSnapshotBuilder {
    fn new(root_source: String) -> Self {
        Self {
            sources: vec![root_source],
            documents: Vec::new(),
            roots: Vec::new(),
            nodes: Vec::new(),
            children: Vec::new(),
            pairs: Vec::new(),
        }
    }

    fn capture_document(&mut self, document: Document<'static>, inherited_source_id: usize) {
        let Document {
            kind,
            range,
            source,
            nested,
            yaml,
            ..
        } = document;

        let source_id = source.map_or(inherited_source_id, |source| {
            let source_id = self.sources.len();
            self.sources.push(source.into_string());
            source_id
        });
        self.capture_document_parts(kind, range, yaml, nested, source_id);
    }

    fn capture_document_parts(
        &mut self,
        kind: DocumentKind,
        range: Span,
        yaml: Option<YamlDocumentAst<'static>>,
        nested: Vec<Document<'static>>,
        source_id: usize,
    ) {
        if kind == DocumentKind::Yaml {
            let roots = yaml.map(|ast| self.capture_ast(ast));
            self.documents.push(BeforeYamlDocument {
                source: source_id,
                range,
                roots,
            });
        }
        for nested_document in nested {
            self.capture_document(nested_document, source_id);
        }
    }

    fn capture_ast(&mut self, ast: YamlDocumentAst<'static>) -> Range<usize> {
        let node_base = self.nodes.len();
        self.nodes.reserve(ast.nodes.len());
        self.roots.reserve(ast.roots.len());

        let roots_start = self.roots.len();
        self.roots.extend(
            ast.roots
                .iter()
                .map(|root| remap_optional_node(root.node, node_base)),
        );
        let roots_end = self.roots.len();

        for node in ast.nodes {
            let emit_rule = scalar_emit_rule(&node.emit);
            let kind = match node.kind {
                YamlAstKind::Empty => BeforeYamlKind::Empty,
                YamlAstKind::Scalar(scalar) => BeforeYamlKind::Scalar(BeforeYamlScalar {
                    style: scalar.style,
                    semantic: scalar.semantic,
                    value: scalar.value.retag(),
                    block_header: scalar.block_header,
                    body: scalar.body.map(SourceSpan::retag),
                    tag: scalar.tag.map(SourceSpan::retag),
                    anchor: scalar.anchor.map(SourceSpan::retag),
                    emit_rule,
                }),
                YamlAstKind::Sequence(sequence) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to a sequence"
                    );
                    let start = self.children.len();
                    self.children.extend(
                        sequence
                            .items
                            .into_iter()
                            .map(|item| remap_optional_node(item.value, node_base)),
                    );
                    BeforeYamlKind::Sequence(BeforeYamlSequence {
                        indent: sequence.indent,
                        children: start..self.children.len(),
                        tag: sequence.tag.map(SourceSpan::retag),
                        anchor: sequence.anchor.map(SourceSpan::retag),
                    })
                }
                YamlAstKind::FlowSequence(sequence) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to a flow sequence"
                    );
                    let start = self.children.len();
                    self.children.extend(
                        sequence
                            .entries
                            .into_iter()
                            .map(|id| Some(remap_node(id, node_base))),
                    );
                    BeforeYamlKind::Sequence(BeforeYamlSequence {
                        indent: 0,
                        children: start..self.children.len(),
                        tag: sequence.tag.map(SourceSpan::retag),
                        anchor: sequence.anchor.map(SourceSpan::retag),
                    })
                }
                YamlAstKind::Mapping(mapping) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to a mapping"
                    );
                    let start = self.pairs.len();
                    self.pairs
                        .extend(mapping.pairs.into_iter().map(|pair| BeforeYamlPair {
                            key: remap_optional_node(pair.key_node, node_base),
                            value: remap_optional_node(pair.value, node_base),
                        }));
                    BeforeYamlKind::Mapping(BeforeYamlMapping {
                        indent: mapping.indent,
                        pairs: start..self.pairs.len(),
                        tag: mapping.tag.map(SourceSpan::retag),
                        anchor: mapping.anchor.map(SourceSpan::retag),
                    })
                }
                YamlAstKind::FlowMapping(mapping) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to a flow mapping"
                    );
                    let start = self.pairs.len();
                    self.pairs
                        .extend(mapping.pairs.into_iter().map(|pair| BeforeYamlPair {
                            key: Some(remap_node(pair.key, node_base)),
                            value: remap_optional_node(pair.value, node_base),
                        }));
                    BeforeYamlKind::Mapping(BeforeYamlMapping {
                        indent: 0,
                        pairs: start..self.pairs.len(),
                        tag: mapping.tag.map(SourceSpan::retag),
                        anchor: mapping.anchor.map(SourceSpan::retag),
                    })
                }
                YamlAstKind::Alias(alias) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to an alias"
                    );
                    BeforeYamlKind::Alias(alias.value.retag())
                }
                YamlAstKind::Opaque(_) => {
                    assert_eq!(
                        emit_rule,
                        ScalarEmitRule::Exact,
                        "scalar YAML emit rule was attached to an opaque node"
                    );
                    BeforeYamlKind::Opaque(node.span.retag())
                }
            };
            self.nodes.push(BeforeYamlNode { kind });
        }

        roots_start..roots_end
    }

    fn finish(self, root_yaml_node_capacity: Option<usize>) -> YamlValidationSnapshot {
        YamlValidationSnapshot {
            sources: self.sources,
            documents: self.documents,
            roots: self.roots,
            nodes: self.nodes,
            children: self.children,
            pairs: self.pairs,
            root_yaml_node_capacity,
        }
    }
}

fn remap_node(id: YamlNodeId, base: usize) -> YamlNodeId {
    YamlNodeId::new(
        base.checked_add(id.index())
            .expect("YAML snapshot node index overflowed usize"),
    )
}

fn remap_optional_node(id: Option<YamlNodeId>, base: usize) -> Option<YamlNodeId> {
    id.map(|id| remap_node(id, base))
}

fn scalar_emit_rule(emit: &YamlEmitPlan) -> ScalarEmitRule {
    match emit {
        YamlEmitPlan::Rendered(YamlRenderedKind::EmptyMarkdownScalar) => {
            ScalarEmitRule::EmptyMarkdown
        }
        YamlEmitPlan::NestedMarkdownBlockScalar { .. }
        | YamlEmitPlan::ExternalBlockScalar
        | YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar) => {
            ScalarEmitRule::ContentMayChange
        }
        _ => ScalarEmitRule::Exact,
    }
}

pub(crate) fn validate_yaml_documents_equivalent(
    before: &YamlValidationSnapshot,
    after_source: &SourceBuffer,
    after_document: &Document<'_>,
) -> Result<()> {
    let mut after_documents = Vec::new();
    collect_yaml_documents(after_document, after_source, &mut after_documents);

    if before.documents.len() != after_documents.len() {
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

    for (before_document, after_document) in before.documents.iter().zip(after_documents) {
        if !yaml_documents_equivalent(before, before_document, after_document)? {
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
struct BeforeYamlContext<'snapshot> {
    source: &'snapshot str,
    snapshot: &'snapshot YamlValidationSnapshot,
}

#[derive(Clone, Copy)]
struct AfterYamlContext<'doc, 'src> {
    source: &'doc SourceBuffer,
    ast: &'doc YamlDocumentAst<'src>,
}

fn yaml_documents_equivalent(
    before_snapshot: &YamlValidationSnapshot,
    before_document: &BeforeYamlDocument,
    after_document: YamlDocumentRef<'_, '_>,
) -> Result<bool> {
    let before_source = &before_snapshot.sources[before_document.source];
    let Some(before_roots) = before_document.roots.as_ref() else {
        return Ok(after_document.document.yaml.is_none()
            && before_document.range.slice(before_source)
                == after_document.source.slice(after_document.document.range));
    };
    let Some(after_ast) = after_document.document.yaml.as_ref() else {
        return Ok(false);
    };
    if before_roots.len() != after_ast.roots.len() {
        return Ok(false);
    }

    let before = BeforeYamlContext {
        source: before_source,
        snapshot: before_snapshot,
    };
    let after = AfterYamlContext {
        source: after_document.source,
        ast: after_ast,
    };
    for (before_root, after_root) in before_snapshot.roots[before_roots.clone()]
        .iter()
        .zip(&after_ast.roots)
    {
        if !optional_nodes_equivalent(before, *before_root, 0, after, after_root.node, 0)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn optional_nodes_equivalent(
    before: BeforeYamlContext<'_>,
    before_id: Option<YamlNodeId>,
    before_parent_indent: usize,
    after: AfterYamlContext<'_, '_>,
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
        (None, Some(after_id)) => Ok(after_node_is_unadorned_null(after, after_id)),
        (Some(before_id), None) => Ok(before_node_is_unadorned_null(before, before_id)),
        (None, None) => Ok(true),
    }
}

fn nodes_equivalent(
    before: BeforeYamlContext<'_>,
    before_id: YamlNodeId,
    before_parent_indent: usize,
    after: AfterYamlContext<'_, '_>,
    after_id: YamlNodeId,
    after_parent_indent: usize,
) -> Result<bool> {
    let before_node = &before.snapshot.nodes[before_id.index()];
    let after_node = after.ast.node(after_id);

    if before_node_is_unadorned_null(before, before_id)
        && after_node_is_unadorned_null(after, after_id)
    {
        return Ok(true);
    }
    if matches!(
        &before_node.kind,
        BeforeYamlKind::Scalar(BeforeYamlScalar {
            emit_rule: ScalarEmitRule::EmptyMarkdown,
            ..
        })
    ) {
        let (BeforeYamlKind::Scalar(before_scalar), YamlAstKind::Scalar(after_scalar)) =
            (&before_node.kind, &after_node.kind)
        else {
            return Ok(false);
        };
        return Ok(optional_span_text_equal_cross_source(
            before.source,
            before_scalar.tag,
            after.source,
            after_scalar.tag,
        ) && optional_span_text_equal_cross_source(
            before.source,
            before_scalar.anchor,
            after.source,
            after_scalar.anchor,
        ) && scalar_is_string_like(after_scalar)
            && comparable_after_scalar_value(after, after_scalar, after_parent_indent)
                .is_some_and(|value| comparable_scalar_is_empty_string(&value)));
    }
    if let (BeforeYamlKind::Scalar(before_scalar), YamlAstKind::Scalar(after_scalar)) =
        (&before_node.kind, &after_node.kind)
    {
        return scalars_equivalent(
            before,
            before_scalar,
            before_parent_indent,
            after,
            after_scalar,
            after_parent_indent,
        );
    }
    if let (BeforeYamlKind::Sequence(before_sequence), Some(after_sequence)) = (
        &before_node.kind,
        AfterSequenceRef::from_kind(&after_node.kind),
    ) {
        return sequences_equivalent(before, before_sequence, after, after_sequence);
    }
    if let (BeforeYamlKind::Mapping(before_mapping), Some(after_mapping)) = (
        &before_node.kind,
        AfterMappingRef::from_kind(&after_node.kind),
    ) {
        return mappings_equivalent(before, before_mapping, after, after_mapping);
    }

    Ok(match (&before_node.kind, &after_node.kind) {
        (BeforeYamlKind::Empty, YamlAstKind::Empty) => true,
        (BeforeYamlKind::Alias(before_alias), YamlAstKind::Alias(after_alias)) => {
            before_alias.span().slice(before.source).trim_ascii()
                == after.source.slice(after_alias.value).trim_ascii()
        }
        (BeforeYamlKind::Opaque(before_span), YamlAstKind::Opaque(_)) => {
            before_span.span().slice(before.source) == after.source.slice(after_node.span)
        }
        _ => false,
    })
}

fn before_node_is_unadorned_null(context: BeforeYamlContext<'_>, id: YamlNodeId) -> bool {
    match &context.snapshot.nodes[id.index()].kind {
        BeforeYamlKind::Empty => true,
        BeforeYamlKind::Scalar(scalar) => {
            scalar.semantic == YamlScalarSemantic::Null
                && scalar.anchor.is_none()
                && scalar
                    .tag
                    .is_none_or(|tag| tag.span().slice(context.source) == "!!null")
        }
        _ => false,
    }
}

fn after_node_is_unadorned_null(context: AfterYamlContext<'_, '_>, id: YamlNodeId) -> bool {
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
    before: BeforeYamlContext<'_>,
    before_scalar: &BeforeYamlScalar,
    before_parent_indent: usize,
    after: AfterYamlContext<'_, '_>,
    after_scalar: &YamlScalar<'_>,
    after_parent_indent: usize,
) -> Result<bool> {
    if !optional_span_text_equal_cross_source(
        before.source,
        before_scalar.tag,
        after.source,
        after_scalar.tag,
    ) || !optional_span_text_equal_cross_source(
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
            == after_source_scalar_text(after, after_scalar)
        && optional_span_text_equal_cross_source(
            before.source,
            before_scalar.body,
            after.source,
            after_scalar.body,
        )
        && (before_scalar.body.is_none() || before_parent_indent == after_parent_indent);
    if raw_equal {
        return Ok(true);
    }

    let after_value = comparable_after_scalar_value(after, after_scalar, after_parent_indent)
        .ok_or_else(|| YamarkError::new(INVALID_OUTPUT))?;
    if before_scalar.emit_rule == ScalarEmitRule::ContentMayChange
        && before_scalar_is_string_like(before_scalar)
        && scalar_is_string_like(after_scalar)
    {
        return Ok(true);
    }
    let before_value = comparable_before_scalar_value(before, before_scalar, before_parent_indent)
        .ok_or_else(|| YamarkError::new(CHANGED_VALUE))?;
    Ok(before_value == after_value)
}

fn scalar_is_string_like(scalar: &YamlScalar<'_>) -> bool {
    scalar.semantic == YamlScalarSemantic::String
        || scalar.semantic == YamlScalarSemantic::Unknown && scalar.tag.is_some()
}

fn before_scalar_is_string_like(scalar: &BeforeYamlScalar) -> bool {
    scalar.semantic == YamlScalarSemantic::String
        || scalar.semantic == YamlScalarSemantic::Unknown && scalar.tag.is_some()
}

fn before_source_scalar_text<'a>(
    context: BeforeYamlContext<'a>,
    scalar: &BeforeYamlScalar,
) -> &'a str {
    let start = [scalar.tag, scalar.anchor]
        .into_iter()
        .flatten()
        .map(SourceSpan::end)
        .max()
        .unwrap_or_else(|| scalar.value.start());
    Span::new(start.min(scalar.value.end()), scalar.value.end())
        .slice(context.source)
        .trim_ascii()
}

fn after_source_scalar_text<'a>(
    context: AfterYamlContext<'a, '_>,
    scalar: &YamlScalar<'_>,
) -> &'a str {
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

fn comparable_before_scalar_value<'a>(
    context: BeforeYamlContext<'a>,
    scalar: &BeforeYamlScalar,
    parent_indent: usize,
) -> Option<ComparableScalar<'a>> {
    comparable_scalar_value(
        before_source_scalar_text(context, scalar),
        scalar.body.map(|body| body.span().slice(context.source)),
        scalar.style,
        scalar.semantic,
        scalar.block_header,
        parent_indent,
    )
}

fn comparable_after_scalar_value<'a>(
    context: AfterYamlContext<'a, '_>,
    scalar: &YamlScalar<'_>,
    parent_indent: usize,
) -> Option<ComparableScalar<'a>> {
    comparable_scalar_value(
        after_source_scalar_text(context, scalar),
        scalar.body.map(|body| context.source.slice(body)),
        scalar.style,
        scalar.semantic,
        scalar.block_header,
        parent_indent,
    )
}

fn comparable_scalar_value<'a>(
    raw: &'a str,
    body: Option<&str>,
    style: YamlScalarStyle,
    semantic: YamlScalarSemantic,
    block_header: Option<YamlBlockScalarHeader>,
    parent_indent: usize,
) -> Option<ComparableScalar<'a>> {
    let decoded = if let Some(body) = body {
        Cow::Owned(decode_block_scalar(
            body,
            style,
            block_header?,
            parent_indent,
        )?)
    } else {
        match style {
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

    Some(match semantic {
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
    let mut chars = inner.chars().peekable();
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
            '\n' => {
                while chars.peek().is_some_and(|ch| matches!(ch, ' ' | '\t')) {
                    chars.next();
                }
            }
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                while chars.peek().is_some_and(|ch| matches!(ch, ' ' | '\t')) {
                    chars.next();
                }
            }
            _ => return None,
        }
    }
    Some(out)
}

fn decode_hex_escape(chars: &mut impl Iterator<Item = char>, digits: usize) -> Option<char> {
    let mut value = 0u32;
    for _ in 0..digits {
        value = value.checked_mul(16)?;
        value += chars.next()?.to_digit(16)?;
    }
    char::from_u32(value)
}

fn decode_block_scalar(
    body: &str,
    style: YamlScalarStyle,
    header: YamlBlockScalarHeader,
    parent_indent: usize,
) -> Option<String> {
    let body = normalize_line_endings(body);
    if body.contains('\u{feff}') {
        return None;
    }
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

    let mut value = match style {
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
enum AfterSequenceRef<'a, 'src> {
    Block(&'a YamlSequence<'src>),
    Flow(&'a YamlFlowSequence<'src>),
}

impl<'a, 'src> AfterSequenceRef<'a, 'src> {
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
    before: BeforeYamlContext<'_>,
    before_sequence: &BeforeYamlSequence,
    after: AfterYamlContext<'_, '_>,
    after_sequence: AfterSequenceRef<'_, '_>,
) -> Result<bool> {
    if before_sequence.children.len() != after_sequence.len()
        || !collection_tag_equal_cross_source(
            before.source,
            before_sequence.tag,
            "!!seq",
            after.source,
            after_sequence.tag(),
        )
        || !optional_span_text_equal_cross_source(
            before.source,
            before_sequence.anchor,
            after.source,
            after_sequence.anchor(),
        )
    {
        return Ok(false);
    }
    for (index, before_item) in before.snapshot.children[before_sequence.children.clone()]
        .iter()
        .enumerate()
    {
        if !optional_nodes_equivalent(
            before,
            *before_item,
            before_sequence.indent,
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
enum AfterMappingRef<'a, 'src> {
    Block(&'a YamlMapping<'src>),
    Flow(&'a YamlFlowMapping<'src>),
}

impl<'a, 'src> AfterMappingRef<'a, 'src> {
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
    before: BeforeYamlContext<'_>,
    before_mapping: &BeforeYamlMapping,
    after: AfterYamlContext<'_, '_>,
    after_mapping: AfterMappingRef<'_, '_>,
) -> Result<bool> {
    if before_mapping.pairs.len() != after_mapping.len()
        || !collection_tag_equal_cross_source(
            before.source,
            before_mapping.tag,
            "!!map",
            after.source,
            after_mapping.tag(),
        )
        || !optional_span_text_equal_cross_source(
            before.source,
            before_mapping.anchor,
            after.source,
            after_mapping.anchor(),
        )
    {
        return Ok(false);
    }
    for (index, before_pair) in before.snapshot.pairs[before_mapping.pairs.clone()]
        .iter()
        .enumerate()
    {
        let (after_key, after_value) = after_mapping.pair(index);
        let keys_equivalent = optional_nodes_equivalent(
            before,
            before_pair.key,
            before_mapping.indent,
            after,
            after_key,
            after_mapping.indent(),
        )?;
        let values_equivalent = optional_nodes_equivalent(
            before,
            before_pair.value,
            before_mapping.indent,
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

fn collection_tag_equal_cross_source(
    before_source: &str,
    before: Option<SourceSpan<'static>>,
    removable: &str,
    after_source: &SourceBuffer,
    after: Option<SourceSpan<'_>>,
) -> bool {
    let before = before
        .map(|span| span.span().slice(before_source))
        .filter(|tag| *tag != removable);
    let after = after
        .map(|span| after_source.slice(span))
        .filter(|tag| *tag != removable);
    before == after
}

fn optional_span_text_equal_cross_source(
    before_source: &str,
    before: Option<SourceSpan<'static>>,
    after_source: &SourceBuffer,
    after: Option<SourceSpan<'_>>,
) -> bool {
    before.map(|span| span.span().slice(before_source))
        == after.map(|span| after_source.slice(span))
}
