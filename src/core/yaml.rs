use crate::config::Config;
use crate::core::directives::{
    Directive, DirectiveDelta, DirectiveEngine, DirectiveState, DirectiveTargetKind, StateId,
    TemplateDelimiter, directive_delta_affects_markdown, file_scope_delta,
    parse_yaml_hash_directive,
};
use crate::core::document::{
    Document, DocumentKind, EmitPlan, FormatOptions, Node, NodeKind, YamlNodeKind,
};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAlias, YamlAstKind, YamlAstNode, YamlBlockChomp, YamlBlockScalarHeader, YamlDocumentAst,
    YamlEmitPlan, YamlFlowMapping, YamlFlowPair, YamlFlowSequence, YamlMapping, YamlMappingPair,
    YamlNodeId, YamlOpaque, YamlOpaqueReason, YamlRenderedKind, YamlRoot, YamlScalar,
    YamlScalarSemantic, YamlScalarStyle, YamlSequence, YamlSequenceItem, YamlTrivia,
    YamlTriviaKind,
};
use crate::core::yaml_scan::{YamlLineScan, scan_yaml_lines, scan_yaml_lines_basic};
use crate::diagnostic::{Result, YamarkError};
use crate::plugins::PluginRegistry;
use memchr::memchr2;
use std::borrow::Cow;
use std::cell::RefCell;

pub fn parse_yaml<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_yaml_impl(source, range, options, config, true, false)
}

pub(crate) fn parse_yaml_for_formatting<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_yaml_impl(source, range, options, config, false, true)
}

pub(crate) fn parse_yaml_for_formatting_with_trace<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_yaml_impl(source, range, options, config, true, true)
}

fn parse_yaml_impl<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
    collect_trace: bool,
    preserve_unsupported_for_formatting: bool,
) -> Result<Document<'src>> {
    crate::core::parser::validate_compact_source_range(range)?;
    let mut options = options;
    if !matches!(
        source.dominant_line_ending,
        crate::core::source::LineEnding::None
    ) {
        options.default_line_ending = source.dominant_line_ending.as_str();
    }
    let mut scan = scan_yaml_lines_basic(source, range);
    if preserve_unsupported_for_formatting && scan.has_tab_indentation(source).is_some() {
        scan = scan_yaml_lines(source, range);
        scan.source_scans += 1;
        if !yaml_scan_has_active_fmt_directive(source, &scan) {
            return Ok(preserved_yaml_document(range, options, &scan));
        }
    }
    if preserve_unsupported_for_formatting
        && yaml_scan_has_unsupported_preserve_syntax(source, &scan)
        && !yaml_scan_has_active_fmt_directive(source, &scan)
    {
        return Ok(preserved_yaml_document(range, options, &scan));
    }
    let parser = YamlParser::new(source, range, options, config, scan, collect_trace);
    let mut doc = parser.parse()?;
    doc.push_node(Node {
        kind: NodeKind::Yaml(YamlNodeKind::Document),
        span: range,
        state: StateId(0),
        emit: EmitPlan::YamlDocument,
    });
    Ok(doc)
}

fn preserved_yaml_document<'src>(
    range: Span,
    options: FormatOptions,
    scan: &YamlLineScan,
) -> Document<'src> {
    let mut doc = Document::new(DocumentKind::Yaml, range);
    doc.options = options;
    doc.trace.source_scans = scan.source_scans;
    doc.trace.parse_passes = 1;
    doc.trace.yaml_scanned_lines = scan.scanned_lines;
    doc.skip_file = true;
    doc
}

pub(crate) fn apply_file_scope_delta_to_yaml_document<'src>(
    source: &'src SourceBuffer,
    document: &mut Document<'src>,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    let owned_source = document.source.take();
    let placeholder = Document::new(DocumentKind::Yaml, document.range);
    let mut doc = std::mem::replace(document, placeholder);
    doc.source = None;
    if let Some(owned_source) = owned_source {
        let doc = doc.retag_source_lifetime();
        let (doc, result) =
            replan_yaml_document_with_delta(&owned_source, doc, delta, options, config);
        let mut doc = doc.retag_source_lifetime();
        doc.source = Some(owned_source);
        *document = doc;
        return result;
    }

    let (doc, result) = replan_yaml_document_with_delta(source, doc, delta, options, config);
    *document = doc;
    result
}

fn replan_yaml_document_with_delta<'src>(
    plan_source: &'src SourceBuffer,
    mut doc: Document<'src>,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> (Document<'src>, Result<()>) {
    doc.patch_all_states(delta.clone());
    let Some(ast) = doc.yaml.take() else {
        return (doc, Ok(()));
    };
    let mut parser = YamlParser {
        source: plan_source,
        options,
        config,
        doc,
        ast,
        engine: DirectiveEngine::new_with_template_delimiters(config.template_delimiters.clone()),
        file_scope_delta: DirectiveDelta::default(),
        start: 0,
        line: 0,
        end: 0,
        held_trivia: Vec::new(),
        flow_collection_nodes: Vec::new(),
        default_template_openers_present: source_contains_any_template_opener(
            plan_source.as_str(),
            &config.template_delimiters,
        ),
        template_spans_possible_by_state: RefCell::new(Vec::new()),
        collect_trace: false,
    };
    parser.patch_existing_ast_states(delta);
    let result = if directive_delta_affects_markdown(delta) {
        parser.patch_nested_markdown_documents(delta)
    } else {
        Ok(())
    };
    if result.is_ok() {
        parser.plan_yaml_emits(true);
        parser.doc.trace.yaml_semantic_nodes = parser.ast.nodes.len();
    }
    let YamlParser {
        mut doc,
        ast,
        source: _,
        options: _,
        config: _,
        engine: _,
        file_scope_delta: _,
        start: _,
        line: _,
        end: _,
        held_trivia: _,
        flow_collection_nodes: _,
        default_template_openers_present: _,
        template_spans_possible_by_state: _,
        collect_trace: _,
    } = parser;
    doc.yaml = Some(ast);
    (doc, result)
}

fn yaml_scan_has_active_fmt_directive(source: &SourceBuffer, scan: &YamlLineScan) -> bool {
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

fn yaml_scan_has_unsupported_preserve_syntax(source: &SourceBuffer, scan: &YamlLineScan) -> bool {
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
        if unsupported_yaml_line_syntax(source, line, scan.end_line) {
            return true;
        }
        line += 1;
    }
    false
}

fn unsupported_yaml_line_syntax(source: &SourceBuffer, line: usize, end: usize) -> bool {
    let text = source.line_text(line);
    let (body, _) = strip_newline(text);
    let trimmed = body.strip_prefix('\u{feff}').unwrap_or(body).trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    if single_line_flow_value_needs_no_preserve_check(text) {
        return false;
    }
    if trimmed.starts_with('%') || unsupported_multiline_flow_collection(source, line, end) {
        return true;
    }
    if line_starts_unclosed_quoted_scalar(text) {
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
    text[content_start..].trim_start().starts_with(['|', '>'])
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
        let next_trimmed = next_text.trim_start();
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
        let rest = rest.trim_start();
        return !rest.starts_with(['[', '{']);
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

fn compact_nested_document_index(index: usize) -> u32 {
    assert!(
        u32::try_from(index).is_ok(),
        "nested document count exceeded u32::MAX"
    );
    index as u32
}

fn yaml_line_is_block_scalar_body(text: &str, indent: usize, block_scalar_indent: usize) -> bool {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.trim().is_empty() || text.starts_with('\t') || indent > block_scalar_indent
}

fn yaml_line_has_fmt_directive_comment(text: &str) -> bool {
    let trimmed = text.strip_prefix('\u{feff}').unwrap_or(text);
    let value_start = text.len() - trimmed.len();
    let Some(comment_start) = find_trailing_comment(text, value_start) else {
        return false;
    };
    text[comment_start..]
        .strip_prefix('#')
        .is_some_and(|rest| rest.trim_start().starts_with("fmt:"))
}

struct YamlParser<'src, 'cfg> {
    source: &'src SourceBuffer,
    options: FormatOptions,
    config: &'cfg Config,
    doc: Document<'src>,
    ast: YamlDocumentAst<'src>,
    engine: DirectiveEngine,
    file_scope_delta: DirectiveDelta,
    start: usize,
    line: usize,
    end: usize,
    held_trivia: Vec<YamlTrivia<'src>>,
    flow_collection_nodes: Vec<YamlNodeId>,
    default_template_openers_present: bool,
    template_spans_possible_by_state: RefCell<Vec<Option<bool>>>,
    collect_trace: bool,
}

impl<'src, 'cfg> YamlParser<'src, 'cfg> {
    fn new(
        source: &'src SourceBuffer,
        range: Span,
        options: FormatOptions,
        config: &'cfg Config,
        scan: YamlLineScan,
        collect_trace: bool,
    ) -> Self {
        let source_scans = scan.source_scans;
        let yaml_scanned_lines = scan.scanned_lines;
        let start_line = scan.start_line;
        let end_line = scan.end_line;
        let mut doc = Document::new(DocumentKind::Yaml, range);
        let mut ast = YamlDocumentAst::new(range);
        ast.nodes
            .reserve(yaml_scanned_lines.saturating_add(yaml_scanned_lines / 16));
        doc.options = options;
        doc.trace.source_scans = source_scans;
        doc.trace.parse_passes = 1;
        doc.trace.yaml_scanned_lines = yaml_scanned_lines;
        Self {
            source,
            options,
            config,
            doc,
            ast,
            engine: DirectiveEngine::new_with_template_delimiters(
                config.template_delimiters.clone(),
            ),
            file_scope_delta: DirectiveDelta::default(),
            start: start_line,
            line: start_line,
            end: end_line,
            held_trivia: Vec::new(),
            flow_collection_nodes: Vec::new(),
            default_template_openers_present: source_contains_any_template_opener(
                source.as_str(),
                &config.template_delimiters,
            ),
            template_spans_possible_by_state: RefCell::new(Vec::new()),
            collect_trace,
        }
    }

    fn parse(mut self) -> Result<Document<'src>> {
        while self.line < self.end || !self.held_trivia.is_empty() {
            let leading = self.take_leading_trivia()?;
            if self.doc.skip_file {
                self.finish_ast();
                self.doc.yaml = Some(self.ast);
                return Ok(self.doc);
            }
            let Some(node) = self.parse_block(0, leading, false)? else {
                break;
            };
            self.ast.roots.push(YamlRoot {
                node: Some(node),
                start_marker: None,
                end_marker: None,
            });
        }
        self.ast.trailing_trivia = self.take_leading_trivia()?;
        if self.doc.skip_file {
            self.finish_ast();
            self.doc.yaml = Some(self.ast);
            return Ok(self.doc);
        }
        if self.engine.formatting_disabled() {
            return Err(yaml_error_at(
                self.source,
                self.doc.range.end.saturating_sub(1),
                "unterminated fmt: off",
            ));
        }
        if let Some(message) = self.engine.pending_target_error() {
            return Err(yaml_error_at(
                self.source,
                self.doc.range.end.saturating_sub(1),
                message,
            ));
        }
        let clear_inline_width_cache = if self.file_scope_delta != DirectiveDelta::default() {
            let delta = self.file_scope_delta.clone();
            self.patch_existing_ast_states(&delta);
            if directive_delta_affects_markdown(&delta) {
                self.patch_nested_markdown_documents(&delta)?;
            }
            true
        } else {
            false
        };
        self.populate_document_root_markers();
        self.plan_yaml_emits(clear_inline_width_cache);
        if self.collect_trace {
            self.record_plan_trace();
        }
        self.finish_ast();
        self.doc.yaml = Some(self.ast);
        Ok(self.doc)
    }

    fn record_plan_trace(&mut self) {
        let mut rendered_scalars = 0usize;
        let mut rendered_flow = 0usize;
        let mut rendered_block_flow = 0usize;
        for node in &self.ast.nodes {
            match &node.emit {
                YamlEmitPlan::Rendered(
                    YamlRenderedKind::EmptyMarkdownScalar
                    | YamlRenderedKind::InlineMarkdownScalar
                    | YamlRenderedKind::Scalar,
                ) => rendered_scalars += 1,
                YamlEmitPlan::Rendered(
                    YamlRenderedKind::CompactCollection | YamlRenderedKind::FlowCollection,
                ) => rendered_flow += 1,
                YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection) => {
                    rendered_block_flow += 1;
                }
                _ => {}
            }
        }
        self.doc.trace.planned_rendered_scalars = rendered_scalars;
        self.doc.trace.planned_rendered_flow_collections = rendered_flow;
        self.doc.trace.planned_rendered_block_flow_collections = rendered_block_flow;
    }

    fn finish_ast(&mut self) {
        self.doc.trace.yaml_semantic_nodes = self.ast.nodes.len();
    }

    fn plan_yaml_emits(&mut self, clear_inline_width_cache: bool) {
        if clear_inline_width_cache {
            self.clear_flow_inline_width_cache();
        }
        let node_count = self.ast.nodes.len();
        for index in 0..node_count {
            let id = YamlNodeId::new(index);
            let needs_plan =
                clear_inline_width_cache || self.ast.node(id).must_preserve_source.is_none();
            if needs_plan {
                self.plan_yaml_node_emit(id);
            }
        }
        let root_count = self.ast.roots.len();
        for index in 0..root_count {
            if let Some(root) = self.ast.roots[index].node {
                self.plan_yaml_contextual_subtree(root, None);
            }
        }
    }

    fn clear_flow_inline_width_cache(&self) {
        for node in &self.ast.nodes {
            node.clear_inline_width();
            node.clear_flow_inline_width();
        }
    }

    fn push_planned_yaml_node(&mut self, node: YamlAstNode<'src>) -> YamlNodeId {
        let id = self.ast.push_node(node);
        self.plan_yaml_node_emit(id);
        id
    }

    fn plan_yaml_node_emit(&mut self, id: YamlNodeId) {
        if id.index() >= self.ast.nodes.len() {
            return;
        }
        let must_preserve_source =
            if let Some(must_preserve_source) = self.ast.node(id).must_preserve_source {
                must_preserve_source
            } else {
                let node = self.ast.node(id);
                self.yaml_node_should_preserve_uncached(node)
            };
        self.ast.node_mut(id).must_preserve_source = Some(must_preserve_source);
        let plan = self.yaml_emit_plan_for(id);
        self.ast.node_mut(id).emit = plan;
    }

    fn yaml_node_should_preserve_uncached(&self, node: &YamlAstNode<'_>) -> bool {
        yaml_node_should_preserve_uncached_with_template_possible(
            self.source,
            &self.doc,
            node,
            self.template_spans_possible_for_state(node.state),
        )
    }

    fn template_spans_possible_for_state(&self, state: StateId) -> bool {
        if self.default_template_openers_present {
            return true;
        }
        let mut cache = self.template_spans_possible_by_state.borrow_mut();
        let state = state.index();
        if state >= cache.len() {
            cache.resize(state + 1, None);
        }
        if let Some(template_spans_possible) = cache[state] {
            return template_spans_possible;
        }
        let template_spans_possible = self.doc.state(StateId::new(state)).template_delimiters
            != self.config.template_delimiters;
        cache[state] = Some(template_spans_possible);
        template_spans_possible
    }

    fn plan_parsed_yaml_flow_nodes(&mut self) {
        let mut ids = std::mem::take(&mut self.flow_collection_nodes);
        for id in ids.iter().copied() {
            self.plan_yaml_node_emit(id);
        }
        ids.clear();
        self.flow_collection_nodes = ids;
    }

    fn plan_yaml_contextual_child_emits(&mut self, id: YamlNodeId) {
        let node = self.ast.node(id);
        let options = self.doc.state(node.state).yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Mapping(mapping) => {
                let indent = mapping.indent;
                let pair_count = mapping.pairs.len();
                for index in 0..pair_count {
                    let (key, trailing_comment, value) = {
                        let YamlAstKind::Mapping(mapping) = &self.ast.node(id).kind else {
                            unreachable!("YAML mapping node changed during planning");
                        };
                        let pair = &mapping.pairs[index];
                        (pair.key, pair.trailing_comment, pair.value)
                    };
                    if let Some((value, plan)) = self.yaml_mapping_value_context_plan(
                        key,
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.ast.node_mut(value).emit = plan;
                    }
                }
            }
            YamlAstKind::Sequence(sequence) => {
                let indent = sequence.indent;
                let item_count = sequence.items.len();
                for index in 0..item_count {
                    let (trailing_comment, value) = {
                        let YamlAstKind::Sequence(sequence) = &self.ast.node(id).kind else {
                            unreachable!("YAML sequence node changed during planning");
                        };
                        let item = &sequence.items[index];
                        (item.trailing_comment, item.value)
                    };
                    if let Some((value, plan)) = self.yaml_sequence_item_context_plan(
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.ast.node_mut(value).emit = plan;
                    }
                }
            }
            _ => {}
        }
    }

    fn plan_yaml_contextual_subtree(&mut self, id: YamlNodeId, forced_indent: Option<usize>) {
        let node = self.ast.node(id);
        let options = self.doc.state(node.state).yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Mapping(mapping) => {
                let indent = forced_indent.unwrap_or(mapping.indent);
                let pair_count = mapping.pairs.len();
                for index in 0..pair_count {
                    let (key, trailing_comment, value) = {
                        let YamlAstKind::Mapping(mapping) = &self.ast.node(id).kind else {
                            unreachable!("YAML mapping node changed during planning");
                        };
                        let pair = &mapping.pairs[index];
                        (pair.key, pair.trailing_comment, pair.value)
                    };
                    if let Some((value, plan)) = self.yaml_mapping_value_context_plan(
                        key,
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.ast.node_mut(value).emit = plan;
                    }
                    if let Some(value) = value {
                        let child_forced_indent = self.yaml_mapping_child_forced_indent(
                            value,
                            trailing_comment,
                            indent,
                            options,
                        );
                        self.plan_yaml_contextual_subtree(value, child_forced_indent);
                    }
                }
            }
            YamlAstKind::Sequence(sequence) => {
                let indent = forced_indent.unwrap_or(sequence.indent);
                let item_count = sequence.items.len();
                for index in 0..item_count {
                    let (trailing_comment, value) = {
                        let YamlAstKind::Sequence(sequence) = &self.ast.node(id).kind else {
                            unreachable!("YAML sequence node changed during planning");
                        };
                        let item = &sequence.items[index];
                        (item.trailing_comment, item.value)
                    };
                    if let Some((value, plan)) = self.yaml_sequence_item_context_plan(
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.ast.node_mut(value).emit = plan;
                    }
                    if let Some(value) = value {
                        let child_forced_indent = self.yaml_sequence_child_forced_indent(
                            value,
                            trailing_comment,
                            indent,
                            options,
                        );
                        self.plan_yaml_contextual_subtree(value, child_forced_indent);
                    }
                }
            }
            _ => {}
        }
    }

    fn populate_document_root_markers(&mut self) {
        let original_roots = std::mem::take(&mut self.ast.roots);
        let mut roots = Vec::with_capacity(original_roots.len());
        let mut pending_start = None;

        for root in original_roots {
            if let Some(node) = root.node {
                let markers = self
                    .ast
                    .node(node)
                    .leading_trivia
                    .iter()
                    .filter(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
                    .map(|trivia| trivia.span)
                    .collect::<Vec<_>>();
                for marker in markers {
                    self.apply_document_marker_to_roots(&mut roots, &mut pending_start, marker);
                }
                roots.push(YamlRoot {
                    node: Some(node),
                    start_marker: pending_start.take(),
                    end_marker: root.end_marker,
                });
            } else {
                roots.push(root);
            }
        }

        let trailing_markers = self
            .ast
            .trailing_trivia
            .iter()
            .filter(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
            .map(|trivia| trivia.span)
            .collect::<Vec<_>>();
        for marker in trailing_markers {
            self.apply_document_marker_to_roots(&mut roots, &mut pending_start, marker);
        }
        if let Some(start_marker) = pending_start.take() {
            let empty = self.push_empty_document_node(Span::empty(start_marker.end()));
            roots.push(YamlRoot {
                node: Some(empty),
                start_marker: Some(start_marker),
                end_marker: None,
            });
        }

        self.ast.roots = roots;
    }

    fn apply_document_marker_to_roots(
        &mut self,
        roots: &mut Vec<YamlRoot<'src>>,
        pending_start: &mut Option<SourceSpan<'src>>,
        marker: SourceSpan<'src>,
    ) {
        match document_marker_kind(self.source.slice(marker)) {
            Some(DocumentMarkerKind::Start) => {
                if let Some(start_marker) = pending_start.take() {
                    let empty = self.push_empty_document_node(Span::new(
                        start_marker.end().min(marker.start()),
                        marker.start(),
                    ));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: Some(start_marker),
                        end_marker: None,
                    });
                }
                *pending_start = Some(marker);
            }
            Some(DocumentMarkerKind::End) => {
                if let Some(start_marker) = pending_start.take() {
                    let empty = self.push_empty_document_node(Span::new(
                        start_marker.end().min(marker.start()),
                        marker.start(),
                    ));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: Some(start_marker),
                        end_marker: Some(marker),
                    });
                } else if let Some(root) = roots
                    .iter_mut()
                    .rev()
                    .find(|root| root.end_marker.is_none())
                {
                    root.end_marker = Some(marker);
                } else {
                    let empty = self.push_empty_document_node(Span::empty(marker.start()));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: None,
                        end_marker: Some(marker),
                    });
                }
            }
            None => {}
        }
    }

    fn push_empty_document_node(&mut self, span: Span) -> YamlNodeId {
        self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::Empty,
            span,
            Vec::new(),
            StateId(0),
        ))
    }

    fn yaml_mapping_value_context_plan(
        &self,
        key: SourceSpan<'src>,
        trailing_comment: Option<SourceSpan<'src>>,
        value: Option<YamlNodeId>,
        mapping_indent: usize,
        options: FormatOptions,
    ) -> Option<(YamlNodeId, YamlEmitPlan)> {
        let value = value?;
        let value_node = self.ast.node(value);
        match &value_node.kind {
            YamlAstKind::Scalar(scalar)
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(
                        YamlRenderedKind::EmptyMarkdownScalar
                            | YamlRenderedKind::InlineMarkdownScalar
                            | YamlRenderedKind::Scalar,
                    )
                ) =>
            {
                let state = self.doc.state(value_node.state);
                let value_options = state.yaml_options(options);
                let body_indent =
                    mapping_pair_child_indent(value_node, mapping_indent + options.indent_width);
                Some((
                    value,
                    self.yaml_scalar_emit_plan(
                        scalar,
                        value_node,
                        state,
                        value_options,
                        Some(body_indent),
                    ),
                ))
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                let rendered_width =
                    planned_yaml_inline_width_or_source(self.source, &self.doc, &self.ast, value);
                let key = self.source.slice(key).trim();
                let inline_width = mapping_indent + key.chars().count() + 2 + rendered_width;
                if yaml_flow_collection_should_expand(
                    self.source,
                    value_node,
                    inline_width,
                    options,
                ) {
                    if trailing_comment.is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            value,
                            mapping_pair_child_indent(
                                value_node,
                                mapping_indent + options.indent_width,
                            ),
                            options,
                        )
                        .is_some()
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection),
                        ));
                    }
                    if yaml_flow_collection_has_multiline_intent(self.source, value_node, options) {
                        return Some((value, YamlEmitPlan::PreserveSource));
                    }
                }
                None
            }
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
                if yaml_block_collection_has_flow_collapse_hint(value_node) {
                    let available_width = options.line_width.saturating_sub(
                        mapping_indent + self.source.slice(key).trim().chars().count() + 2,
                    );
                    return Some((
                        value,
                        self.yaml_flow_collapse_hint_emit_plan(value, available_width),
                    ));
                }
                if trailing_comment.is_none()
                    && self
                        .doc
                        .state(value_node.state)
                        .yaml_options(options)
                        .yaml_compact
                    && !yaml_node_has_properties(self.source, value_node)
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                {
                    let available_width = options.line_width.saturating_sub(
                        mapping_indent + self.source.slice(key).trim().chars().count() + 2,
                    );
                    if compact_yaml_node_width(self.source, &self.doc, &self.ast, value)
                        .is_some_and(|width| width <= available_width)
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection),
                        ));
                    }
                }
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                ) {
                    return Some((value, YamlEmitPlan::None));
                }
                None
            }
            _ => None,
        }
    }

    fn yaml_sequence_item_context_plan(
        &self,
        trailing_comment: Option<SourceSpan<'src>>,
        value: Option<YamlNodeId>,
        sequence_indent: usize,
        options: FormatOptions,
    ) -> Option<(YamlNodeId, YamlEmitPlan)> {
        let value = value?;
        let value_node = self.ast.node(value);
        match &value_node.kind {
            YamlAstKind::Scalar(scalar)
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(
                        YamlRenderedKind::EmptyMarkdownScalar
                            | YamlRenderedKind::InlineMarkdownScalar
                            | YamlRenderedKind::Scalar,
                    )
                ) =>
            {
                let state = self.doc.state(value_node.state);
                let value_options = state.yaml_options(options);
                Some((
                    value,
                    self.yaml_scalar_emit_plan(
                        scalar,
                        value_node,
                        state,
                        value_options,
                        Some(sequence_indent + options.indent_width),
                    ),
                ))
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                let rendered_width =
                    planned_yaml_inline_width_or_source(self.source, &self.doc, &self.ast, value);
                let inline_width = sequence_indent + 2 + rendered_width;
                if yaml_flow_collection_should_expand(
                    self.source,
                    value_node,
                    inline_width,
                    options,
                ) {
                    if trailing_comment.is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            value,
                            sequence_indent + options.indent_width,
                            options,
                        )
                        .is_some()
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection),
                        ));
                    }
                    if yaml_flow_collection_has_multiline_intent(self.source, value_node, options) {
                        return Some((value, YamlEmitPlan::PreserveSource));
                    }
                }
                None
            }
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
                if yaml_block_collection_has_flow_collapse_hint(value_node) {
                    let available_width = options.line_width.saturating_sub(sequence_indent + 2);
                    return Some((
                        value,
                        self.yaml_flow_collapse_hint_emit_plan(value, available_width),
                    ));
                }
                if trailing_comment.is_none()
                    && self
                        .doc
                        .state(value_node.state)
                        .yaml_options(options)
                        .yaml_compact
                    && !yaml_node_has_properties(self.source, value_node)
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                {
                    let available_width = options.line_width.saturating_sub(sequence_indent + 2);
                    if compact_yaml_node_width(self.source, &self.doc, &self.ast, value)
                        .is_some_and(|width| width <= available_width)
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection),
                        ));
                    }
                }
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                ) {
                    return Some((value, YamlEmitPlan::None));
                }
                None
            }
            _ => None,
        }
    }

    fn yaml_mapping_child_forced_indent(
        &self,
        value: YamlNodeId,
        trailing_comment: Option<SourceSpan<'src>>,
        mapping_indent: usize,
        options: FormatOptions,
    ) -> Option<usize> {
        let value_node = self.ast.node(value);
        if trailing_comment.is_some()
            || matches!(
                value_node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
            )
        {
            return None;
        }
        matches!(
            value_node.kind,
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
        )
        .then_some(mapping_pair_child_indent(
            value_node,
            mapping_indent + options.indent_width,
        ))
    }

    fn yaml_sequence_child_forced_indent(
        &self,
        value: YamlNodeId,
        trailing_comment: Option<SourceSpan<'src>>,
        sequence_indent: usize,
        options: FormatOptions,
    ) -> Option<usize> {
        let value_node = self.ast.node(value);
        if trailing_comment.is_some()
            || matches!(
                value_node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
            )
        {
            return None;
        }
        matches!(
            value_node.kind,
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
        )
        .then_some(sequence_indent + options.indent_width)
    }

    fn plan_yaml_sequence_emit(&mut self, id: YamlNodeId) {
        self.plan_yaml_node_emit(id);
        self.plan_yaml_contextual_child_emits(id);
    }

    fn yaml_emit_plan_for(&self, id: YamlNodeId) -> YamlEmitPlan {
        let node = self.ast.node(id);
        if yaml_node_should_preserve(self.source, &self.doc, node) {
            return YamlEmitPlan::PreserveSource;
        }

        let state = self.doc.state(node.state);
        let options = state.yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Scalar(scalar) => {
                self.yaml_scalar_emit_plan(scalar, node, state, options, None)
            }
            YamlAstKind::Sequence(sequence) => {
                if yaml_block_collection_has_flow_collapse_hint(node) {
                    let available_width = options
                        .line_width
                        .saturating_sub(yaml_node_source_indent(self.source, node));
                    return self.yaml_flow_collapse_hint_emit_plan(id, available_width);
                }
                if let Some(compact_table) = state.table_compact {
                    if flow_table_sequence_renderable(
                        self.source,
                        &self.doc,
                        &self.ast,
                        sequence,
                        compact_table,
                    )
                    .is_some()
                    {
                        return YamlEmitPlan::rendered_shape(YamlRenderedKind::Table);
                    }
                    return YamlEmitPlan::PreserveSource;
                }
                if options.yaml_compact
                    && compact_yaml_node_width(self.source, &self.doc, &self.ast, id).is_some_and(
                        |width| {
                            width
                                <= options
                                    .line_width
                                    .saturating_sub(yaml_node_source_indent(self.source, node))
                        },
                    )
                {
                    return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
                }
                YamlEmitPlan::None
            }
            YamlAstKind::Mapping(_) => {
                if yaml_block_collection_has_flow_collapse_hint(node) {
                    let available_width = options
                        .line_width
                        .saturating_sub(yaml_node_source_indent(self.source, node));
                    return self.yaml_flow_collapse_hint_emit_plan(id, available_width);
                }
                if options.yaml_compact
                    && compact_yaml_node_width(self.source, &self.doc, &self.ast, id).is_some_and(
                        |width| {
                            width
                                <= options
                                    .line_width
                                    .saturating_sub(yaml_node_source_indent(self.source, node))
                        },
                    )
                {
                    return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
                }
                YamlEmitPlan::None
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                if state.table_compact.is_some() {
                    return YamlEmitPlan::PreserveSource;
                }
                let multiline_intent =
                    yaml_flow_collection_has_multiline_intent(self.source, node, options);
                let Some(output_width) =
                    render_yaml_inline_node_width(self.source, &self.doc, &self.ast, id)
                else {
                    return if multiline_intent {
                        YamlEmitPlan::PreserveSource
                    } else {
                        YamlEmitPlan::None
                    };
                };
                let inline_width = yaml_node_source_indent(self.source, node) + output_width;
                if yaml_flow_collection_should_expand(self.source, node, inline_width, options) {
                    if flow_trailing_comment(&node.kind).is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            id,
                            yaml_node_source_indent(self.source, node),
                            options,
                        )
                        .is_some()
                    {
                        return YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection);
                    }
                    if multiline_intent {
                        return YamlEmitPlan::PreserveSource;
                    }
                }
                YamlEmitPlan::rendered_shape(YamlRenderedKind::FlowCollection)
            }
            _ => YamlEmitPlan::None,
        }
    }

    fn yaml_flow_collapse_hint_emit_plan(
        &self,
        id: YamlNodeId,
        available_width: usize,
    ) -> YamlEmitPlan {
        if compact_yaml_node_width(self.source, &self.doc, &self.ast, id)
            .is_some_and(|width| width <= available_width)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
        }
        YamlEmitPlan::PreserveSource
    }

    fn yaml_scalar_emit_plan(
        &self,
        scalar: &YamlScalar<'_>,
        _node: &YamlAstNode<'_>,
        state: &crate::core::directives::DirectiveState,
        options: FormatOptions,
        _body_indent: Option<usize>,
    ) -> YamlEmitPlan {
        let tagged_markdown = scalar
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(self.source.slice(tag)));
        if scalar.value.is_empty() && (state.markdown_target || tagged_markdown) {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::EmptyMarkdownScalar);
        }
        if scalar.body.is_some()
            && (state.markdown_target || tagged_markdown)
            && markdown_block_scalar_body_is_empty(self.source, scalar)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::EmptyMarkdownScalar);
        }
        if scalar.body.is_none()
            && (state.markdown_target || tagged_markdown)
            && inline_markdown_scalar_is_renderable(self.source, scalar)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::InlineMarkdownScalar);
        }
        if scalar.body.is_some()
            && state.embedded_formatter.is_some()
            && options.skip_embedded_formatters
        {
            return YamlEmitPlan::PreserveSource;
        }
        if scalar.body.is_some() && state.embedded_formatter.is_some() {
            return YamlEmitPlan::ExternalBlockScalar;
        }
        if scalar.body.is_some() {
            if let Some(nested) = scalar.nested {
                return YamlEmitPlan::NestedMarkdownBlockScalar { nested };
            }
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        }
        if scalar.value.is_empty() && scalar_has_properties(scalar) {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        }
        if scalar.value.is_empty() {
            return YamlEmitPlan::None;
        }
        YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar)
    }

    fn parse_block(
        &mut self,
        indent: usize,
        leading: Vec<YamlTrivia<'src>>,
        container_is_target: bool,
    ) -> Result<Option<YamlNodeId>> {
        if self.line >= self.end {
            self.unread_trivia(leading);
            return Ok(None);
        }

        if let Some(content_start) = self.document_marker_inline_content_start(&leading) {
            return self
                .parse_document_marker_inline_content(leading, content_start)
                .map(Some);
        }

        let text = self.source.line_text(self.line);
        if tab_in_indentation(text).is_some() {
            return self.parse_tab_indented_opaque(leading).map(Some);
        }
        let actual_indent = indentation(text);
        if actual_indent < indent {
            self.unread_trivia(leading);
            return Ok(None);
        }

        if let Some(hint) =
            flow_collapse_hint_from_standalone_opener(self.source, self.line, self.end, indent)
        {
            return self
                .parse_flow_collapse_hint_collection(indent, leading, hint, container_is_target)
                .map(Some);
        }

        if !line_has_flow_collapse_hint_value(self.source, self.line, self.end)
            && let Some(block) = unsupported_flow_block_at(self.source, self.line, self.end)
        {
            return self.parse_opaque_flow(block, leading);
        }

        if sequence_line(text, actual_indent).is_some() {
            return self.parse_sequence(actual_indent, leading).map(Some);
        }

        if explicit_key_line(text, actual_indent).is_some() {
            return self
                .parse_explicit_mapping(actual_indent, leading, container_is_target)
                .map(Some);
        }

        if mapping_colon_at(text, actual_indent).is_some() {
            return self
                .parse_mapping(actual_indent, leading, container_is_target)
                .map(Some);
        }

        if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            actual_indent,
            self.source.lines[self.line].full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            return self.parse_flow_collection_block(block, leading).map(Some);
        }

        self.parse_plain_scalar_line(leading).map(Some)
    }

    fn document_marker_inline_content_start(&self, leading: &[YamlTrivia<'_>]) -> Option<usize> {
        let marker_trivia = leading
            .iter()
            .rev()
            .find(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)?;
        let line = self.source.lines.get(self.line)?;
        if marker_trivia.span.start() != line.full.start() {
            return None;
        }
        let marker = document_marker_line_info(self.source.line_text(self.line))?;
        let content_start = marker.inline_content_start?;
        (marker_trivia.span.end() == line.text.start() + content_start).then_some(content_start)
    }

    fn parse_document_marker_inline_content(
        &mut self,
        leading: Vec<YamlTrivia<'src>>,
        content_start: usize,
    ) -> Result<YamlNodeId> {
        if let Some(scan) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            content_start,
            self.source.lines[self.line].text.start() + content_start,
        ) {
            match scan {
                FlowCollectionScan::Complete(block) => {
                    return self.parse_flow_collection_block(block, leading);
                }
                FlowCollectionScan::Incomplete => {
                    let line = self.source.lines[self.line];
                    return self.parse_opaque_flow_node(
                        Span::new(line.text.start() + content_start, line.full.end()),
                        leading,
                    );
                }
            }
        }

        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let line_value = split_line_value_comment(text, content_start, line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;
        self.line += 1;
        let id = self.push_scalar_line_with_comment(
            Span::new(line.text.start() + content_start, line.full.end()),
            line_value.value,
            line_value.trailing_comment,
            PlainScalarContinuation::None,
        )?;
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    fn parse_mapping(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia<'src>>,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let state = if container_is_target {
            self.engine
                .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping)
        } else {
            self.engine.state_for_node(&mut self.doc, false)
        };
        let mut pairs = Vec::new();
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let mut start = self.source.lines[self.line].full.start();
        let mut end = start;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let actual_indent = indentation(text);
            let explicit = explicit_key_line(text, indent).is_some();
            let implicit = mapping_colon_at(text, indent).is_some();
            if actual_indent != indent || (!explicit && !implicit) {
                self.unread_trivia(leading);
                break;
            }

            if pairs.is_empty() && leading.is_empty() {
                start = line.full.start();
            }

            let (pair, pair_end) = if explicit {
                self.parse_explicit_mapping_pair_at(indent, std::mem::take(&mut leading))?
            } else {
                self.parse_mapping_pair_at(indent, std::mem::take(&mut leading))?
            };
            end = pair_end;
            pairs.push(pair);

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading) || self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(id)
    }

    fn parse_explicit_mapping(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia<'src>>,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let state = if container_is_target {
            self.engine
                .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping)
        } else {
            self.engine.state_for_node(&mut self.doc, false)
        };
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let start = self.source.lines[self.line].full.start();
        let mut end = start;
        let mut pairs = Vec::new();

        while self.line < self.end {
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let explicit = explicit_key_line(text, indent).is_some();
            let implicit = mapping_colon_at(text, indent).is_some();
            if indentation(text) != indent || (!explicit && !implicit) {
                self.unread_trivia(leading);
                break;
            }

            let (pair, pair_end) = if explicit {
                self.parse_explicit_mapping_pair_at(indent, std::mem::take(&mut leading))?
            } else {
                self.parse_mapping_pair_at(indent, std::mem::take(&mut leading))?
            };
            end = pair_end;
            pairs.push(pair);

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading) || self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(id)
    }

    fn parse_explicit_mapping_pair_at(
        &mut self,
        indent: usize,
        leading: Vec<YamlTrivia<'src>>,
    ) -> Result<(YamlMappingPair<'src>, usize)> {
        let key_line = self.source.lines[self.line];
        let key_text = self.source.line_text(self.line);
        let marker =
            explicit_indicator_at(key_text, indent, b'?').expect("explicit mapping key exists");
        let key_start = skip_ascii_whitespace(key_text, marker + 1);
        let (key, key_node) = self.parse_explicit_key_node(indent, key_start)?;
        let mut between_trivia = self.take_leading_trivia()?;
        if self.line >= self.end {
            self.unread_trivia(std::mem::take(&mut between_trivia));
            let pair_end = key_node
                .map(|id| self.ast.node(id).span.end())
                .unwrap_or(key_line.full.end())
                .max(key_line.full.end());
            let value = self.push_empty_scalar(Span::empty(pair_end), pair_end)?;
            return Ok((
                YamlMappingPair {
                    leading_trivia: leading.into(),
                    key: SourceSpan::new(key),
                    key_node,
                    colon: SourceSpan::empty(pair_end),
                    line: SourceSpan::new(key_line.full.into()),
                    source: SourceSpan::new(Span::new(key_line.full.start(), pair_end)),
                    explicit: true,
                    trailing_comment: None,
                    value: Some(value),
                },
                pair_end,
            ));
        }

        let value_line = self.source.lines[self.line];
        let value_text = self.source.line_text(self.line);
        let Some(colon) = explicit_value_line(value_text, indent) else {
            self.unread_trivia(std::mem::take(&mut between_trivia));
            let pair_end = key_node
                .map(|id| self.ast.node(id).span.end())
                .unwrap_or(key_line.full.end())
                .max(key_line.full.end());
            let value = self.push_empty_scalar(Span::empty(pair_end), pair_end)?;
            return Ok((
                YamlMappingPair {
                    leading_trivia: leading.into(),
                    key: SourceSpan::new(key),
                    key_node,
                    colon: SourceSpan::empty(pair_end),
                    line: SourceSpan::new(key_line.full.into()),
                    source: SourceSpan::new(Span::new(key_line.full.start(), pair_end)),
                    explicit: true,
                    trailing_comment: None,
                    value: Some(value),
                },
                pair_end,
            ));
        };
        let colon_span = Span::new(
            value_line.text.start() + colon,
            value_line.text.start() + colon + 1,
        );
        let value_start = skip_ascii_whitespace(value_text, colon + 1);
        let line_value = split_line_value_comment(value_text, value_start, value_line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;

        let mut pair_trailing_comment = None;
        let end;
        let value = if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            value_line.full.start(),
        ) {
            let value = self.parse_quoted_scalar_block(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            value_line.full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            let value = self.parse_flow_collection_block(block, Vec::new())?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(hint) =
            flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
        {
            let value =
                self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
            let value = self.parse_block_scalar(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if compact_sequence_marker_at(value_text, value_start).is_some() {
            let value = self.parse_compact_sequence(value_start)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(value) = self.parse_sequence_item_inline_mapping(indent, value_start)? {
            end = self.ast.node(value).span.end();
            Some(value)
        } else {
            let metadata = scalar_metadata(self.source, line_value.value);
            let value_is_empty = self.source.slice(line_value.value).trim().is_empty();
            let property_only_value = !value_is_empty
                && (metadata.tag.is_some() || metadata.anchor.is_some())
                && self.source.slice(metadata.content).trim().is_empty();
            self.line += 1;
            if value_is_empty || property_only_value {
                pair_trailing_comment = line_value.trailing_comment;
                let inline_directive =
                    self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                let nested_leading = self.take_leading_trivia()?;
                let nested_is_target = inline_directive
                    || leading
                        .iter()
                        .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                let collapse_hint = nested_leading.is_empty().then(|| {
                    flow_collapse_hint_from_standalone_opener(
                        self.source,
                        self.line,
                        self.end,
                        indent,
                    )
                });
                let indentless_sequence = self.line < self.end
                    && sequence_line(self.source.line_text(self.line), indent).is_some();
                if let Some(Some(hint)) = collapse_hint {
                    let nested = self.parse_flow_collapse_hint_collection(
                        indent,
                        nested_leading,
                        hint,
                        nested_is_target,
                    )?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.next_line_has_tab_indentation() {
                    let nested = self.parse_tab_indented_opaque(nested_leading)?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.line < self.end
                    && (indentation(self.source.line_text(self.line)) > indent
                        || indentless_sequence)
                {
                    let nested_indent = if indentless_sequence {
                        indent
                    } else {
                        indent + 1
                    };
                    let nested =
                        self.parse_block(nested_indent, nested_leading, nested_is_target)?;
                    if let Some(nested) = nested {
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                    } else {
                        end = value_line.full.end();
                    }
                    nested
                } else {
                    self.unread_trivia(nested_leading);
                    if property_only_value {
                        let scalar = self.push_scalar_line_with_comment(
                            value_line.full.into(),
                            line_value.value,
                            line_value.trailing_comment,
                            PlainScalarContinuation::None,
                        )?;
                        end = self.ast.node(scalar).span.end();
                        Some(scalar)
                    } else {
                        let scalar =
                            self.push_empty_scalar(value_line.full.into(), value_line.text.end())?;
                        end = value_line.full.end();
                        Some(scalar)
                    }
                }
            } else {
                let scalar = self.push_scalar_line_with_comment(
                    value_line.full.into(),
                    line_value.value,
                    line_value.trailing_comment,
                    PlainScalarContinuation::Inline {
                        parent_indent: indent,
                    },
                )?;
                end = self.ast.node(scalar).span.end();
                Some(scalar)
            }
        };

        Ok((
            YamlMappingPair {
                leading_trivia: leading.into(),
                key: SourceSpan::new(key),
                key_node,
                colon: SourceSpan::new(colon_span),
                line: SourceSpan::new(value_line.full.into()),
                source: SourceSpan::new(Span::new(key_line.full.start(), end)),
                explicit: true,
                trailing_comment: pair_trailing_comment.map(SourceSpan::new),
                value,
            },
            end,
        ))
    }

    fn parse_explicit_key_node(
        &mut self,
        indent: usize,
        key_start: usize,
    ) -> Result<(Span, Option<YamlNodeId>)> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let key_value = split_line_value_comment(text, key_start, line.text.start());
        self.reject_same_line_yaml_directive(key_value.trailing_comment)?;
        let key = trim_span_ascii(self.source, key_value.value);

        if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
            let key_node = self.parse_block_scalar(block)?;
            self.validate_non_value_directive_target(key_node)?;
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            key_start,
            line.text.start() + key_start,
        )
        .and_then(FlowCollectionScan::complete)
        {
            let key_node = self.parse_flow_collection_block(block, Vec::new())?;
            let key = match &self.ast.node(key_node).kind {
                YamlAstKind::FlowSequence(sequence) => sequence.value.span(),
                YamlAstKind::FlowMapping(mapping) => mapping.value.span(),
                _ => self.ast.node(key_node).span.span(),
            };
            return Ok((key, Some(key_node)));
        }

        if compact_sequence_marker_at(text, key_start).is_some() {
            let key_node = self.parse_compact_sequence(key_start)?;
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if mapping_colon_from(text, key_start).is_some() {
            let key_node = self
                .parse_sequence_item_inline_mapping(indent, key_start)?
                .expect("mapping colon produced a mapping key node");
            let key = self.ast.node(key_node).span.span();
            return Ok((key, Some(key_node)));
        }

        if !key.is_empty() {
            self.line += 1;
            return Ok((key, self.push_mapping_key_node(key)));
        }

        self.line += 1;
        let nested_leading = self.take_leading_trivia()?;
        if self.line < self.end && indentation(self.source.line_text(self.line)) > indent {
            let key_node = self.parse_block(indent + 1, nested_leading, false)?;
            if let Some(key_node) = key_node {
                let key = self.ast.node(key_node).span.span();
                return Ok((key, Some(key_node)));
            }
        } else {
            self.unread_trivia(nested_leading);
        }

        Ok((key, None))
    }

    fn parse_mapping_pair_at(
        &mut self,
        key_start_column: usize,
        leading: Vec<YamlTrivia<'src>>,
    ) -> Result<(YamlMappingPair<'src>, usize)> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let colon = mapping_colon_from(text, key_start_column).expect("mapping pair exists");
        let key_start = line.text.start() + key_start_column;
        let key_end =
            line.text.start() + key_start_column + text[key_start_column..colon].trim_end().len();
        let key = Span::new(key_start, key_end);
        let key_node = self.push_mapping_key_node(key);
        let colon_span = Span::new(line.text.start() + colon, line.text.start() + colon + 1);
        let value_start = colon
            + 1
            + text[colon + 1..]
                .bytes()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
        let line_value = split_line_value_comment(text, value_start, line.text.start());
        self.reject_populated_same_line_yaml_directive(line_value)?;

        let mut pair_trailing_comment = None;
        let end;
        let value = if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        ) {
            let value = self.parse_quoted_scalar_block(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = flow_collection_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        )
        .and_then(FlowCollectionScan::complete)
        {
            let value = self.parse_flow_collection_block(block, Vec::new())?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(hint) =
            flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
        {
            let value = self.parse_flow_collapse_hint_collection(
                key_start_column,
                Vec::new(),
                hint,
                false,
            )?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
            let value = self.parse_block_scalar(block)?;
            end = self.ast.node(value).span.end();
            Some(value)
        } else {
            let metadata = scalar_metadata(self.source, line_value.value);
            let value_is_empty = self.source.slice(line_value.value).trim().is_empty();
            let property_only_value = !value_is_empty
                && (metadata.tag.is_some() || metadata.anchor.is_some())
                && self.source.slice(metadata.content).trim().is_empty();
            self.line += 1;
            if value_is_empty || property_only_value {
                pair_trailing_comment = line_value.trailing_comment;
                let inline_directive =
                    self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                let nested_leading = self.take_leading_trivia()?;
                let nested_is_target = inline_directive
                    || leading
                        .iter()
                        .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                let collapse_hint = nested_leading.is_empty().then(|| {
                    flow_collapse_hint_from_standalone_opener(
                        self.source,
                        self.line,
                        self.end,
                        key_start_column,
                    )
                });
                let indentless_sequence = self.line < self.end
                    && sequence_line(self.source.line_text(self.line), key_start_column).is_some();
                if let Some(Some(hint)) = collapse_hint {
                    let nested = self.parse_flow_collapse_hint_collection(
                        key_start_column,
                        nested_leading,
                        hint,
                        nested_is_target,
                    )?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.next_line_has_tab_indentation() {
                    let nested = self.parse_tab_indented_opaque(nested_leading)?;
                    if property_only_value {
                        self.attach_collection_properties(nested, metadata.tag, metadata.anchor);
                    }
                    end = self.ast.node(nested).span.end();
                    Some(nested)
                } else if self.line < self.end
                    && (indentation(self.source.line_text(self.line)) > key_start_column
                        || indentless_sequence)
                {
                    let nested_indent = if indentless_sequence {
                        key_start_column
                    } else {
                        key_start_column + 1
                    };
                    let nested =
                        self.parse_block(nested_indent, nested_leading, nested_is_target)?;
                    if let Some(nested) = nested {
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                    } else {
                        end = line.full.end();
                    }
                    nested
                } else {
                    self.unread_trivia(nested_leading);
                    if property_only_value {
                        let scalar = self.push_scalar_line_with_comment(
                            line.full.into(),
                            line_value.value,
                            line_value.trailing_comment,
                            PlainScalarContinuation::None,
                        )?;
                        end = self.ast.node(scalar).span.end();
                        Some(scalar)
                    } else {
                        let scalar = self.push_empty_scalar(line.full.into(), line.text.end())?;
                        end = line.full.end();
                        Some(scalar)
                    }
                }
            } else {
                let scalar = self.push_scalar_line_with_comment(
                    line.full.into(),
                    line_value.value,
                    line_value.trailing_comment,
                    PlainScalarContinuation::Inline {
                        parent_indent: key_start_column,
                    },
                )?;
                end = self.ast.node(scalar).span.end();
                Some(scalar)
            }
        };

        Ok((
            YamlMappingPair {
                leading_trivia: leading.into(),
                key: SourceSpan::new(key),
                key_node,
                colon: SourceSpan::new(colon_span),
                line: SourceSpan::new(line.full.into()),
                source: SourceSpan::new(Span::new(line.full.start(), end)),
                explicit: false,
                trailing_comment: pair_trailing_comment.map(SourceSpan::new),
                value,
            },
            end,
        ))
    }

    fn attach_collection_properties(
        &mut self,
        node: YamlNodeId,
        tag: Option<Span>,
        anchor: Option<Span>,
    ) {
        let tag = tag.map(SourceSpan::new);
        let anchor = anchor.map(SourceSpan::new);
        match &mut self.ast.node_mut(node).kind {
            YamlAstKind::Mapping(mapping) => {
                mapping.tag = tag;
                mapping.anchor = anchor;
            }
            YamlAstKind::Sequence(sequence) => {
                sequence.tag = tag;
                sequence.anchor = anchor;
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.tag = tag;
                mapping.anchor = anchor;
            }
            YamlAstKind::FlowSequence(sequence) => {
                sequence.tag = tag;
                sequence.anchor = anchor;
            }
            _ => {}
        }
    }

    fn push_mapping_key_node(&mut self, key: Span) -> Option<YamlNodeId> {
        let key = trim_span_ascii(self.source, key);
        if key.is_empty() {
            return None;
        }

        let state = self.engine.state_for_node(&mut self.doc, false);
        let metadata = scalar_metadata(self.source, key);
        let template_spans_possible = self.template_spans_possible_for_state(state);
        if let Some(id) = parse_flow_collection(
            self.source,
            &mut self.ast,
            &mut self.flow_collection_nodes,
            state,
            &self.doc.state(state).template_delimiters,
            template_spans_possible,
            key,
            key,
            metadata.content,
            None,
        ) {
            self.plan_parsed_yaml_flow_nodes();
            return Some(id);
        }

        let source = self.source.slice(metadata.content).trim();
        if alias_scalar(source) {
            return Some(self.ast.push_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(metadata.content),
                    trailing_comment: None,
                }),
                key,
                Vec::new(),
                state,
            )));
        }

        Some(self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(key),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            key,
            Vec::new(),
            state,
        )))
    }

    fn parse_sequence(
        &mut self,
        indent: usize,
        first_leading: Vec<YamlTrivia<'src>>,
    ) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlSequence);
        let mut items = Vec::new();
        let node_leading = first_leading.clone();
        let mut leading = first_leading;
        let start = self.source.lines[self.line].full.start();
        let mut end = start;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                self.unread_trivia(leading);
                break;
            }
            let actual_indent = indentation(text);
            let Some(marker_local) = sequence_line(text, indent) else {
                self.unread_trivia(leading);
                break;
            };
            if actual_indent != indent {
                self.unread_trivia(leading);
                break;
            }

            let marker = Span::new(
                line.text.start() + marker_local,
                line.text.start() + marker_local + 1,
            );
            let value_start = marker_local
                + 1
                + text[marker_local + 1..]
                    .bytes()
                    .take_while(|byte| byte.is_ascii_whitespace())
                    .count();
            let line_value = split_line_value_comment(text, value_start, line.text.start());
            self.reject_populated_same_line_yaml_directive(line_value)?;

            let mut item_trailing_comment = None;
            let mut value_on_marker_line = false;
            let value = if let Some(block) = quoted_scalar_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            ) {
                let value = self.parse_quoted_scalar_block(block)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = flow_collection_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            )
            .and_then(FlowCollectionScan::complete)
            {
                let value = self.parse_flow_collection_block(block, Vec::new())?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(hint) =
                flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
            {
                let value =
                    self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
                let value = self.parse_block_scalar(block)?;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_explicit_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_inline_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if compact_sequence_marker_at(text, value_start).is_some() {
                let value = self.parse_compact_sequence(value_start)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else {
                let metadata = scalar_metadata(self.source, line_value.value);
                let value_is_empty = self.source.slice(line_value.value).trim().is_empty();
                let property_only_value = !value_is_empty
                    && (metadata.tag.is_some() || metadata.anchor.is_some())
                    && self.source.slice(metadata.content).trim().is_empty();
                self.line += 1;
                if value_is_empty || property_only_value {
                    item_trailing_comment = line_value.trailing_comment;
                    let inline_directive =
                        self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                    let nested_leading = self.take_leading_trivia()?;
                    let nested_is_target = inline_directive
                        || leading
                            .iter()
                            .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                    if self.next_line_has_tab_indentation() {
                        let nested = self.parse_tab_indented_opaque(nested_leading)?;
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                        Some(nested)
                    } else if self.line < self.end
                        && indentation(self.source.line_text(self.line)) > indent
                    {
                        let nested =
                            self.parse_block(indent + 1, nested_leading, nested_is_target)?;
                        if let Some(nested) = nested {
                            if property_only_value {
                                self.attach_collection_properties(
                                    nested,
                                    metadata.tag,
                                    metadata.anchor,
                                );
                            }
                            end = self.ast.node(nested).span.end();
                        } else {
                            end = line.full.end();
                        }
                        nested
                    } else {
                        self.unread_trivia(nested_leading);
                        if property_only_value {
                            let scalar = self.push_scalar_line_with_comment(
                                line.full.into(),
                                line_value.value,
                                line_value.trailing_comment,
                                PlainScalarContinuation::None,
                            )?;
                            end = self.ast.node(scalar).span.end();
                            Some(scalar)
                        } else {
                            let scalar =
                                self.push_empty_scalar(line.full.into(), line.text.end())?;
                            end = line.full.end();
                            Some(scalar)
                        }
                    }
                } else {
                    let scalar = self.push_scalar_line_with_comment(
                        line.full.into(),
                        line_value.value,
                        line_value.trailing_comment,
                        PlainScalarContinuation::Inline {
                            parent_indent: indent,
                        },
                    )?;
                    end = self.ast.node(scalar).span.end();
                    Some(scalar)
                }
            };

            items.push(YamlSequenceItem {
                leading_trivia: std::mem::take(&mut leading).into(),
                marker: SourceSpan::new(marker),
                line: SourceSpan::new(line.full.into()),
                value_on_marker_line,
                trailing_comment: item_trailing_comment.map(SourceSpan::new),
                value,
            });

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading)
                || leading
                    .iter()
                    .any(|trivia| trivia.kind == YamlTriviaKind::Directive)
            {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if tab_in_indentation(self.source.line_text(self.line)).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Sequence(YamlSequence {
                indent,
                items,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            node_leading,
            state,
        ));
        self.plan_yaml_sequence_emit(id);
        Ok(id)
    }

    fn parse_compact_sequence(&mut self, indent: usize) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlSequence);
        let start = self.source.lines[self.line].text.start() + indent;
        let mut end = start;
        let mut items = Vec::new();
        let mut leading = Vec::new();
        let mut first = true;

        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            let marker_local = if first {
                let Some(marker) = compact_sequence_marker_at(text, indent) else {
                    break;
                };
                marker
            } else {
                let actual_indent = indentation(text);
                let Some(marker) = sequence_line(text, indent) else {
                    self.unread_trivia(leading);
                    break;
                };
                if actual_indent != indent {
                    self.unread_trivia(leading);
                    break;
                }
                marker
            };
            first = false;

            let marker = Span::new(
                line.text.start() + marker_local,
                line.text.start() + marker_local + 1,
            );
            let value_start = marker_local
                + 1
                + text[marker_local + 1..]
                    .bytes()
                    .take_while(|byte| byte.is_ascii_whitespace())
                    .count();
            let line_value = split_line_value_comment(text, value_start, line.text.start());
            self.reject_populated_same_line_yaml_directive(line_value)?;

            let mut item_trailing_comment = None;
            let mut value_on_marker_line = false;
            let value = if let Some(block) = quoted_scalar_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            ) {
                let value = self.parse_quoted_scalar_block(block)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = flow_collection_block_from_value(
                self.source,
                self.line,
                self.end,
                value_start,
                line.full.start(),
            )
            .and_then(FlowCollectionScan::complete)
            {
                let value = self.parse_flow_collection_block(block, Vec::new())?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(hint) =
                flow_collapse_hint_from_value(self.source, self.line, self.end, value_start, true)
            {
                let value =
                    self.parse_flow_collapse_hint_collection(indent, Vec::new(), hint, false)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(block) = block_scalar_at(self.source, self.line, self.end) {
                let value = self.parse_block_scalar(block)?;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_explicit_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if let Some(value) =
                self.parse_sequence_item_inline_mapping(indent, value_start)?
            {
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else if compact_sequence_marker_at(text, value_start).is_some() {
                let value = self.parse_compact_sequence(value_start)?;
                value_on_marker_line = true;
                end = self.ast.node(value).span.end();
                Some(value)
            } else {
                let metadata = scalar_metadata(self.source, line_value.value);
                let value_is_empty = self.source.slice(line_value.value).trim().is_empty();
                let property_only_value = !value_is_empty
                    && (metadata.tag.is_some() || metadata.anchor.is_some())
                    && self.source.slice(metadata.content).trim().is_empty();
                self.line += 1;
                if value_is_empty || property_only_value {
                    item_trailing_comment = line_value.trailing_comment;
                    let inline_directive =
                        self.apply_same_line_yaml_directive(line_value.trailing_comment)?;
                    let nested_leading = self.take_leading_trivia()?;
                    let nested_is_target = inline_directive
                        || leading
                            .iter()
                            .any(|trivia| trivia.kind == YamlTriviaKind::Directive);
                    if self.next_line_has_tab_indentation() {
                        let nested = self.parse_tab_indented_opaque(nested_leading)?;
                        if property_only_value {
                            self.attach_collection_properties(
                                nested,
                                metadata.tag,
                                metadata.anchor,
                            );
                        }
                        end = self.ast.node(nested).span.end();
                        Some(nested)
                    } else if self.line < self.end
                        && indentation(self.source.line_text(self.line)) > indent
                    {
                        let nested =
                            self.parse_block(indent + 1, nested_leading, nested_is_target)?;
                        if let Some(nested) = nested {
                            if property_only_value {
                                self.attach_collection_properties(
                                    nested,
                                    metadata.tag,
                                    metadata.anchor,
                                );
                            }
                            end = self.ast.node(nested).span.end();
                        } else {
                            end = line.full.end();
                        }
                        nested
                    } else {
                        self.unread_trivia(nested_leading);
                        if property_only_value {
                            let scalar = self.push_scalar_line_with_comment(
                                line.full.into(),
                                line_value.value,
                                line_value.trailing_comment,
                                PlainScalarContinuation::None,
                            )?;
                            end = self.ast.node(scalar).span.end();
                            Some(scalar)
                        } else {
                            let scalar =
                                self.push_empty_scalar(line.full.into(), line.text.end())?;
                            end = line.full.end();
                            Some(scalar)
                        }
                    }
                } else {
                    let scalar = self.push_scalar_line_with_comment(
                        line.full.into(),
                        line_value.value,
                        line_value.trailing_comment,
                        PlainScalarContinuation::Inline {
                            parent_indent: indent,
                        },
                    )?;
                    end = self.ast.node(scalar).span.end();
                    Some(scalar)
                }
            };

            items.push(YamlSequenceItem {
                leading_trivia: std::mem::take(&mut leading).into(),
                marker: SourceSpan::new(marker),
                line: SourceSpan::new(line.full.into()),
                value_on_marker_line,
                trailing_comment: item_trailing_comment.map(SourceSpan::new),
                value,
            });

            leading = self.take_leading_trivia()?;
            if trivia_has_document_marker(&leading)
                || leading
                    .iter()
                    .any(|trivia| trivia.kind == YamlTriviaKind::Directive)
            {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            if self.line >= self.end {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let next_indent = indentation(self.source.line_text(self.line));
            if next_indent < indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Sequence(YamlSequence {
                indent,
                items,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_sequence_emit(id);
        Ok(id)
    }

    fn parse_sequence_item_inline_mapping(
        &mut self,
        sequence_indent: usize,
        key_start_column: usize,
    ) -> Result<Option<YamlNodeId>> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        if mapping_colon_from(text, key_start_column).is_none() {
            return Ok(None);
        }

        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping);
        let start = line.text.start() + key_start_column;
        let mut pairs = Vec::new();
        let (first_pair, mut end) = self.parse_mapping_pair_at(key_start_column, Vec::new())?;
        pairs.push(first_pair);

        let mut leading = self.take_leading_trivia()?;
        let mut continuation_indent = None;
        while self.line < self.end {
            if trivia_has_document_marker(&leading) {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let text = self.source.line_text(self.line);
            let actual_indent = indentation(text);
            if actual_indent <= sequence_indent || sequence_line(text, sequence_indent).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let pair_indent = *continuation_indent.get_or_insert(actual_indent);
            if actual_indent != pair_indent || mapping_colon_at(text, pair_indent).is_none() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let (pair, pair_end) =
                self.parse_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?;
            end = pair_end;
            pairs.push(pair);
            leading = self.take_leading_trivia()?;
        }
        if self.line >= self.end {
            self.unread_trivia(leading);
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent: key_start_column,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(Some(id))
    }

    fn parse_sequence_item_explicit_mapping(
        &mut self,
        sequence_indent: usize,
        key_marker_column: usize,
    ) -> Result<Option<YamlNodeId>> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        if explicit_indicator_at(text, key_marker_column, b'?').is_none() {
            return Ok(None);
        }

        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlMapping);
        let start = line.full.start();
        let mut pairs = Vec::new();
        let (first_pair, mut end) =
            self.parse_explicit_mapping_pair_at(key_marker_column, Vec::new())?;
        pairs.push(first_pair);

        let mut leading = self.take_leading_trivia()?;
        let mut continuation_indent = None;
        while self.line < self.end {
            if trivia_has_document_marker(&leading) {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }
            let text = self.source.line_text(self.line);
            let actual_indent = indentation(text);
            if actual_indent <= sequence_indent || sequence_line(text, sequence_indent).is_some() {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let pair_indent = *continuation_indent.get_or_insert(actual_indent);
            if actual_indent != pair_indent {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            }

            let (pair, pair_end) = if explicit_key_line(text, pair_indent).is_some() {
                self.parse_explicit_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?
            } else if mapping_colon_at(text, pair_indent).is_some() {
                self.parse_mapping_pair_at(pair_indent, std::mem::take(&mut leading))?
            } else {
                self.unread_trivia(std::mem::take(&mut leading));
                break;
            };
            end = pair_end;
            pairs.push(pair);
            leading = self.take_leading_trivia()?;
        }
        if self.line >= self.end {
            self.unread_trivia(leading);
        }

        let id = self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Mapping(YamlMapping {
                indent: key_marker_column,
                pairs,
                tag: None,
                anchor: None,
                flow_collapse_hint: None,
            }),
            Span::new(start, end),
            Vec::new(),
            state,
        ));
        self.plan_yaml_contextual_child_emits(id);
        Ok(Some(id))
    }

    fn parse_plain_scalar_line(&mut self, leading: Vec<YamlTrivia<'src>>) -> Result<YamlNodeId> {
        let line = self.source.lines[self.line];
        let text = self.source.line_text(self.line);
        let value_start = indentation(text);
        if let Some(block) = quoted_scalar_block_from_value(
            self.source,
            self.line,
            self.end,
            value_start,
            line.full.start(),
        ) {
            let id = self.parse_quoted_scalar_block(block)?;
            self.ast.node_mut(id).leading_trivia = leading.into();
            return Ok(id);
        }

        let line_value = split_line_value_comment(text, value_start, line.text.start());
        self.line += 1;
        let id = self.push_scalar_line_with_comment(
            line.full.into(),
            line_value.value,
            line_value.trailing_comment,
            PlainScalarContinuation::Block {
                indent: value_start,
            },
        )?;
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    fn parse_flow_collapse_hint_collection(
        &mut self,
        parent_indent: usize,
        leading: Vec<YamlTrivia<'src>>,
        hint: FlowCollapseHint,
        container_is_target: bool,
    ) -> Result<YamlNodeId> {
        let opener_line = self.source.lines[self.line];
        let id = match hint.rest {
            FlowCollapseRest::Empty => {
                self.line += 1;
                let nested_leading = self.take_leading_trivia()?;
                let text = self.source.line_text(self.line);
                let indent = indentation(text);
                self.parse_block(indent, nested_leading, container_is_target)?
                    .expect("prechecked flow collapse hint has a block collection target")
            }
            FlowCollapseRest::InlineMapping { key_start } => self
                .parse_sequence_item_inline_mapping(parent_indent, key_start)?
                .expect("prechecked flow collapse hint has an inline mapping target"),
        };
        self.mark_flow_collapse_hint(id, opener_line.full.start(), hint.opener);
        self.ast.node_mut(id).leading_trivia = leading.into();
        Ok(id)
    }

    fn mark_flow_collapse_hint(&mut self, id: YamlNodeId, source_start: usize, opener: Span) {
        let node = self.ast.node_mut(id);
        node.span.set_start(source_start);
        match &mut node.kind {
            YamlAstKind::Sequence(sequence) => {
                sequence.flow_collapse_hint = Some(SourceSpan::new(opener));
            }
            YamlAstKind::Mapping(mapping) => {
                mapping.flow_collapse_hint = Some(SourceSpan::new(opener));
            }
            _ => {}
        }
    }

    fn parse_opaque_flow(
        &mut self,
        block: Span,
        leading: Vec<YamlTrivia<'src>>,
    ) -> Result<Option<YamlNodeId>> {
        self.parse_opaque_flow_node(block, leading).map(Some)
    }

    fn parse_tab_indented_opaque(&mut self, leading: Vec<YamlTrivia<'src>>) -> Result<YamlNodeId> {
        let line = self.source.lines[self.line];
        let block = tab_indented_block_at(self.source, self.line, self.end);
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlUnsupported);
        let state_value = self.doc.state(state);
        if state_value.markdown_target
            || state_value.embedded_formatter.is_some()
            || state_value.table_compact.is_some()
        {
            return Err(yaml_error_at(
                self.source,
                line.text.start()
                    + tab_in_indentation(self.source.line_text(self.line)).unwrap_or(0),
                "tabs in YAML indentation are unsupported",
            ));
        }
        self.line = self.source.line_at_byte(block.end.saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Opaque(YamlOpaque {
                reason: YamlOpaqueReason::UnsupportedLine,
            }),
            block,
            leading,
            state,
        )))
    }

    fn next_line_has_tab_indentation(&self) -> bool {
        self.line < self.end && tab_in_indentation(self.source.line_text(self.line)).is_some()
    }

    fn parse_opaque_flow_node(
        &mut self,
        block: Span,
        leading: Vec<YamlTrivia<'src>>,
    ) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlUnsupported);
        let state_value = self.doc.state(state);
        if state_value.markdown_target
            || state_value.embedded_formatter.is_some()
            || state_value.table_compact.is_some()
        {
            let line = self.source.lines[self.line];
            return Err(yaml_error_at(
                self.source,
                line.text.start(),
                "fmt directive targets unsupported YAML flow syntax",
            ));
        }
        self.line = self.source.line_at_byte(block.end.saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Opaque(YamlOpaque {
                reason: YamlOpaqueReason::UnsupportedFlow,
            }),
            block,
            leading,
            state,
        )))
    }

    fn parse_quoted_scalar_block(&mut self, block: QuotedScalarBlock) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        self.validate_inline_scalar_directive_target(state, block.value)?;
        let metadata = scalar_metadata(self.source, block.value);
        let source = self.source.slice(metadata.content).trim();
        self.line = self.source.line_at_byte(block.full.end().saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(block.value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: block.trailing_comment.map(SourceSpan::new),
            }),
            block.full,
            Vec::new(),
            state,
        )))
    }

    fn parse_flow_collection_block(
        &mut self,
        block: FlowCollectionBlock,
        leading: Vec<YamlTrivia<'src>>,
    ) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;

        let target = flow_collection_directive_target(self.source, block.collection);
        let needs_rollback = self.engine.has_pending_target_directives();
        let engine_checkpoint = needs_rollback.then(|| self.engine.clone());
        let states_checkpoint = needs_rollback.then(|| self.doc.states.clone());
        let state = self.engine.state_for_yaml_node(&mut self.doc, target);
        let checkpoint = self.ast.nodes.len();
        let template_spans_possible = self.template_spans_possible_for_state(state);
        let Some(id) = parse_flow_collection(
            self.source,
            &mut self.ast,
            &mut self.flow_collection_nodes,
            state,
            &self.doc.state(state).template_delimiters,
            template_spans_possible,
            block.span,
            block.value,
            block.collection,
            block.trailing_comment,
        ) else {
            if let Some(engine) = engine_checkpoint {
                self.engine = engine;
            }
            if let Some(states) = states_checkpoint {
                self.doc.states = states;
            }
            return self.parse_opaque_flow_node(block.span, leading);
        };
        if let Err(err) = self.validate_flow_collection_directive_target(state, block.value) {
            self.ast.nodes.truncate(checkpoint);
            self.flow_collection_nodes.clear();
            return Err(err);
        }
        self.ast.node_mut(id).leading_trivia = leading.into();
        self.plan_parsed_yaml_flow_nodes();
        self.line = self.source.line_at_byte(block.span.end.saturating_sub(1)) + 1;
        Ok(id)
    }

    fn parse_block_scalar(&mut self, block: BlockScalar) -> Result<YamlNodeId> {
        self.reject_same_line_yaml_directive(block.trailing_comment)?;
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        let state_value = self.doc.state(state).clone();
        let metadata = scalar_metadata(self.source, block.value);
        let tagged_markdown = metadata
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(self.source.slice(tag)));
        let style = block_scalar_style(self.source.slice(block.header));
        if state_value.embedded_formatter.is_some() && style != YamlScalarStyle::LiteralBlock {
            return Err(self.embedded_formatter_target_error(state, block.value));
        }
        let nested = if state_value.markdown_target || tagged_markdown {
            Some(self.doc.push_nested(crate::core::markdown::parse_markdown(
                self.source,
                block.body,
                state_value.markdown_options(self.options),
                self.config,
            )?))
        } else {
            None
        };
        let nested = nested.map(compact_nested_document_index);
        self.line = self.source.line_at_byte(block.full.end().saturating_sub(1)) + 1;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style,
                semantic: YamlScalarSemantic::String,
                value: SourceSpan::new(block.value),
                header: Some(SourceSpan::new(block.header)),
                block_header: Some(block.header_info),
                body: Some(SourceSpan::new(block.body)),
                nested,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: block.trailing_comment.map(SourceSpan::new),
            }),
            block.full,
            Vec::new(),
            state,
        )))
    }

    fn push_empty_scalar(&mut self, span: Span, at: usize) -> Result<YamlNodeId> {
        let state = self
            .engine
            .state_for_yaml_node(&mut self.doc, DirectiveTargetKind::YamlScalar);
        self.validate_inline_scalar_directive_target(state, Span::empty(at))?;
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: YamlScalarStyle::Plain,
                semantic: YamlScalarSemantic::Null,
                value: SourceSpan::empty(at),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: None,
                anchor: None,
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            state,
        )))
    }

    fn push_scalar_line_with_comment(
        &mut self,
        mut span: Span,
        mut value: Span,
        trailing_comment: Option<Span>,
        continuation: PlainScalarContinuation,
    ) -> Result<YamlNodeId> {
        let mut metadata = scalar_metadata(self.source, value);
        if !self.source.slice(metadata.content).trim().is_empty() {
            self.reject_same_line_yaml_directive(trailing_comment)?;
        }
        let target = if let Some(target) =
            scalar_content_flow_collection_target(self.source, metadata.content)
        {
            target
        } else {
            DirectiveTargetKind::YamlScalar
        };
        let state = self.engine.state_for_yaml_node(&mut self.doc, target);
        let template_spans_possible = self.template_spans_possible_for_state(state);
        if let Some(id) = parse_flow_collection(
            self.source,
            &mut self.ast,
            &mut self.flow_collection_nodes,
            state,
            &self.doc.state(state).template_delimiters,
            template_spans_possible,
            span,
            value,
            metadata.content,
            trailing_comment,
        ) {
            self.validate_flow_collection_directive_target(state, value)?;
            self.plan_parsed_yaml_flow_nodes();
            return Ok(id);
        }
        self.validate_inline_scalar_directive_target(state, value)?;
        if metadata.content.is_empty() && (metadata.tag.is_some() || metadata.anchor.is_some()) {
            value = metadata.content;
        }
        let source = self.source.slice(metadata.content).trim();
        if alias_scalar(source) {
            self.validate_non_scalar_value_directive_target(state, metadata.content)?;
            return Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(metadata.content),
                    trailing_comment: trailing_comment.map(SourceSpan::new),
                }),
                span,
                Vec::new(),
                state,
            )));
        }
        if trailing_comment.is_none()
            && scalar_style(source) == YamlScalarStyle::Plain
            && !source.is_empty()
            && let Some(extension) = self.take_plain_scalar_continuation(continuation)
        {
            span.end = extension.span_end;
            value.end = extension.value_end;
            metadata.content.end = extension.value_end;
        }
        let source = self.source.slice(metadata.content).trim();
        Ok(self.push_planned_yaml_node(YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: scalar_style(source),
                semantic: scalar_semantic_with_tag(
                    source,
                    metadata.tag.map(|tag| self.source.slice(tag)),
                ),
                value: SourceSpan::new(value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: metadata.tag.map(SourceSpan::new),
                anchor: metadata.anchor.map(SourceSpan::new),
                trailing_comment: trailing_comment.map(SourceSpan::new),
            }),
            span,
            Vec::new(),
            state,
        )))
    }

    fn take_plain_scalar_continuation(
        &mut self,
        continuation: PlainScalarContinuation,
    ) -> Option<PlainScalarExtension> {
        if matches!(continuation, PlainScalarContinuation::None) {
            return None;
        }

        let initial_line = self.line;
        let mut scan = self.line;
        let mut final_line = self.line;
        let mut span_end = None;
        let mut value_end = None;

        while scan < self.end {
            let line = self.source.lines[scan];
            let text = self.source.line_text(scan);
            if text.trim().is_empty() {
                scan += 1;
                continue;
            }
            if !plain_scalar_continuation_line(text, continuation) {
                break;
            }

            final_line = scan + 1;
            span_end = Some(line.full.end());
            value_end = Some(line.text.end());
            scan += 1;
        }

        if final_line == initial_line {
            return None;
        }

        self.line = final_line;
        Some(PlainScalarExtension {
            span_end: span_end.expect("continuation has a final line"),
            value_end: value_end.expect("continuation has a final line"),
        })
    }

    fn validate_inline_scalar_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Ok(());
        }
        Ok(())
    }

    fn validate_flow_collection_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Err(yaml_error_at(
                self.source,
                span.start,
                "fmt: markdown targets a scalar value",
            ));
        }
        Ok(())
    }

    fn validate_non_value_directive_target(&self, node: YamlNodeId) -> Result<()> {
        let state = self.ast.node(node).state;
        self.validate_non_scalar_value_directive_target(state, self.ast.node(node).span.span())
    }

    fn validate_non_scalar_value_directive_target(&self, state: StateId, span: Span) -> Result<()> {
        let state_value = self.doc.state(state);
        if state_value.embedded_formatter.is_some() {
            return Err(self.embedded_formatter_target_error(state, span));
        }
        if state_value.markdown_target {
            return Err(yaml_error_at(
                self.source,
                span.start,
                "fmt: markdown targets a scalar value",
            ));
        }
        Ok(())
    }

    fn embedded_formatter_target_error(&self, state: StateId, span: Span) -> YamarkError {
        let name = self
            .doc
            .state(state)
            .embedded_formatter
            .as_deref()
            .unwrap_or("embedded formatter");
        yaml_error_at(
            self.source,
            span.start,
            format!("fmt: {name} targets a literal block scalar"),
        )
    }

    fn take_leading_trivia(&mut self) -> Result<Vec<YamlTrivia<'src>>> {
        let mut trivia = std::mem::take(&mut self.held_trivia);
        if self.doc.skip_file {
            return Ok(trivia);
        }
        while self.line < self.end {
            let line = self.source.lines[self.line];
            let text = self.source.line_text(self.line);
            if tab_in_indentation(text).is_some() {
                break;
            }
            let kind = if text.trim().is_empty() {
                Some(YamlTriviaKind::Blank)
            } else if let Some(marker) = document_marker_line_info(text) {
                if let Some(content_start) = marker.inline_content_start {
                    let marker_span =
                        Span::new(line.full.start(), line.text.start() + content_start);
                    if trivia.last().is_some_and(|trivia| {
                        trivia.kind == YamlTriviaKind::DocumentMarker
                            && trivia.span.span() == marker_span
                    }) {
                        break;
                    }
                    trivia.push(YamlTrivia {
                        kind: YamlTriviaKind::DocumentMarker,
                        span: SourceSpan::new(marker_span),
                    });
                    break;
                }
                Some(YamlTriviaKind::DocumentMarker)
            } else if standard_yaml_directive(text) {
                Some(YamlTriviaKind::Directive)
            } else if let Some(directive) = parse_yaml_hash_directive(text).map_err(|message| {
                yaml_error_at(
                    self.source,
                    line.text.start() + text.find('#').unwrap_or(0),
                    message,
                )
            })? {
                let directive = self.infer_yaml_template_directive_scope(directive, self.line)?;
                self.apply_yaml_directive(
                    directive,
                    line.text.start() + text.find('#').unwrap_or(0),
                )?;
                Some(YamlTriviaKind::Directive)
            } else if text.trim_start().starts_with('#') {
                Some(YamlTriviaKind::Comment)
            } else {
                None
            };
            let Some(kind) = kind else {
                break;
            };
            trivia.push(YamlTrivia {
                kind,
                span: SourceSpan::new(line.full.into()),
            });
            self.line += 1;
            if self.doc.skip_file {
                break;
            }
        }
        Ok(trivia)
    }

    fn infer_yaml_template_directive_scope(
        &self,
        directive: Directive,
        line: usize,
    ) -> Result<Directive> {
        let Directive::Template { scope, delimiter } = directive else {
            return Ok(directive);
        };
        if scope != crate::core::directives::Scope::Next
            || self.source.line_text(line).contains("scope=")
        {
            return Ok(Directive::Template { scope, delimiter });
        }
        if self.yaml_template_has_immediate_target(line) {
            return Ok(Directive::Template { scope, delimiter });
        }
        if self.yaml_directive_is_isolated(line) {
            return Ok(Directive::Template {
                scope: crate::core::directives::Scope::FromHere,
                delimiter,
            });
        }
        Err(yaml_error_at(
            self.source,
            self.source.lines[line].text.start()
                + self.source.line_text(line).find('#').unwrap_or(0),
            "fmt: template.delimiters needs explicit scope",
        ))
    }

    fn yaml_template_has_immediate_target(&self, line: usize) -> bool {
        let Some(next) = line.checked_add(1).filter(|next| *next < self.end) else {
            return false;
        };
        let text = self.source.line_text(next);
        !(text.trim().is_empty()
            || text.trim_start().starts_with('#')
            || document_marker(text)
            || standard_yaml_directive(text))
    }

    fn yaml_directive_is_isolated(&self, line: usize) -> bool {
        let before_blank = line == self.start || self.source.line_text(line - 1).trim().is_empty();
        let after_blank = line + 1 >= self.end || self.source.line_text(line + 1).trim().is_empty();
        before_blank && after_blank
    }

    fn apply_same_line_yaml_directive(&mut self, comment: Option<Span>) -> Result<bool> {
        let Some((comment, directive)) = self.parse_same_line_yaml_directive(comment)? else {
            return Ok(false);
        };
        self.apply_yaml_directive(directive, comment.start)?;
        Ok(true)
    }

    fn reject_same_line_yaml_directive(&self, comment: Option<Span>) -> Result<()> {
        let Some((comment, _)) = self.parse_same_line_yaml_directive(comment)? else {
            return Ok(());
        };
        Err(yaml_error_at(
            self.source,
            comment.start,
            "fmt directive is not supported in this position",
        ))
    }

    fn reject_populated_same_line_yaml_directive(&self, line_value: LineValue) -> Result<()> {
        let metadata = scalar_metadata(self.source, line_value.value);
        if !self.source.slice(metadata.content).trim().is_empty() {
            self.reject_same_line_yaml_directive(line_value.trailing_comment)?;
        }
        Ok(())
    }

    fn parse_same_line_yaml_directive(
        &self,
        comment: Option<Span>,
    ) -> Result<Option<(Span, Directive)>> {
        let Some(comment) = comment else {
            return Ok(None);
        };
        let text = self.source.slice(comment);
        let directive = parse_yaml_hash_directive(text)
            .map_err(|message| yaml_error_at(self.source, comment.start, message))?;
        Ok(directive.map(|directive| (comment, directive)))
    }

    fn apply_yaml_directive(&mut self, directive: Directive, byte: usize) -> Result<()> {
        if let Directive::Embedded { name } = &directive
            && !PluginRegistry::is_known_formatter(self.config, name)
        {
            return Err(yaml_error_at(
                self.source,
                byte,
                format!("unknown embedded formatter: {name}"),
            ));
        }
        let delta = file_scope_delta(&directive);
        self.engine
            .apply_yaml_directive(&mut self.doc, directive)
            .map_err(|message| yaml_error_at(self.source, byte, message))?;
        if let Some(delta) = delta {
            self.file_scope_delta.merge_from(delta);
        }
        Ok(())
    }

    fn patch_existing_ast_states(&mut self, delta: &DirectiveDelta) {
        let mut patched = Vec::with_capacity(self.ast.nodes.len());
        for node in &self.ast.nodes {
            let mut state = self.doc.states.get(node.state).clone();
            delta.apply_to(&mut state);
            patched.push(self.doc.states.intern(state));
        }
        for (node, state) in self.ast.nodes.iter_mut().zip(patched) {
            node.state = state;
        }
    }

    fn patch_nested_markdown_documents(&mut self, delta: &DirectiveDelta) -> Result<()> {
        let mut nested_markdown = Vec::new();
        for node in &self.ast.nodes {
            let YamlAstKind::Scalar(scalar) = &node.kind else {
                continue;
            };
            let Some(nested) = scalar.nested else {
                continue;
            };
            if scalar.body.is_none() {
                continue;
            }
            nested_markdown.push((nested as usize, node.state));
        }
        for (nested, state) in nested_markdown {
            let state = self.doc.state(state).clone();
            let nested_config = config_for_directive_state(self.config, &state);
            crate::core::markdown::apply_file_scope_delta_to_markdown_document(
                self.source,
                &mut self.doc.nested[nested],
                delta,
                state.markdown_options(self.options),
                &nested_config,
            )?;
        }
        Ok(())
    }

    fn unread_trivia(&mut self, trivia: Vec<YamlTrivia<'src>>) {
        if trivia.is_empty() {
            return;
        }
        if self.held_trivia.is_empty() {
            self.held_trivia = trivia;
        } else {
            let mut merged = trivia;
            merged.append(&mut self.held_trivia);
            self.held_trivia = merged;
        }
    }
}

fn trivia_has_document_marker(trivia: &[YamlTrivia<'_>]) -> bool {
    trivia
        .iter()
        .any(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
}

fn config_for_directive_state(config: &Config, state: &DirectiveState) -> Config {
    let mut config = config.clone();
    config.template_delimiters = state.template_delimiters.clone();
    config
}

#[allow(clippy::too_many_arguments)]
fn parse_flow_collection<'src>(
    source: &'src SourceBuffer,
    ast: &mut YamlDocumentAst<'src>,
    collection_nodes: &mut Vec<YamlNodeId>,
    state: StateId,
    template_delimiters: &[TemplateDelimiter],
    template_spans_possible: bool,
    line_span: Span,
    value: Span,
    collection: Span,
    trailing_comment: Option<Span>,
) -> Option<YamlNodeId> {
    let text = source.slice(collection);
    let (trimmed_start, trimmed_end) = trim_ascii_range(text, 0, text.len())?;
    let collection = Span::new(
        collection.start + trimmed_start,
        collection.start + trimmed_end,
    );
    let first = source.slice(collection).as_bytes().first().copied()?;
    if !matches!(first, b'[' | b'{') {
        return None;
    }
    let metadata = scalar_metadata(source, value);

    collection_nodes.clear();
    let checkpoint = ast.nodes.len();
    let parsed = {
        let mut parser = FlowParser {
            text: source.slice(collection),
            base: collection.start,
            pos: 0,
            ast,
            collection_nodes,
            state,
            template_delimiters,
            template_spans_possible,
            inner_trivia: Vec::new(),
        };
        let id = parser.parse_value();
        parser.skip_ws();
        id.filter(|_| parser.pos == parser.text.len())
            .map(|id| (id, !parser.inner_trivia.is_empty()))
    };
    let Some((id, has_inner_trivia)) = parsed else {
        ast.nodes.truncate(checkpoint);
        collection_nodes.clear();
        return None;
    };

    let node = ast.node_mut(id);
    node.span = SourceSpan::new(line_span);
    match &mut node.kind {
        YamlAstKind::FlowSequence(sequence) => {
            sequence.value = SourceSpan::new(value);
            sequence.tag = metadata.tag.map(SourceSpan::new);
            sequence.anchor = metadata.anchor.map(SourceSpan::new);
            sequence.trailing_comment = trailing_comment.map(SourceSpan::new);
            sequence.has_inner_trivia = has_inner_trivia;
        }
        YamlAstKind::FlowMapping(mapping) => {
            mapping.value = SourceSpan::new(value);
            mapping.tag = metadata.tag.map(SourceSpan::new);
            mapping.anchor = metadata.anchor.map(SourceSpan::new);
            mapping.trailing_comment = trailing_comment.map(SourceSpan::new);
            mapping.has_inner_trivia = has_inner_trivia;
        }
        _ => {
            ast.nodes.truncate(checkpoint);
            collection_nodes.clear();
            return None;
        }
    }
    Some(id)
}

fn flow_collection_parseable(source: &SourceBuffer, value: Span) -> bool {
    let mut ast = YamlDocumentAst::new(value);
    let mut collection_nodes = Vec::new();
    let metadata = scalar_metadata(source, value);
    parse_flow_collection(
        source,
        &mut ast,
        &mut collection_nodes,
        StateId(0),
        &[],
        false,
        value,
        value,
        metadata.content,
        None,
    )
    .is_some()
}

struct FlowParser<'src, 'ast, 'cfg> {
    text: &'src str,
    base: usize,
    pos: usize,
    ast: &'ast mut YamlDocumentAst<'src>,
    collection_nodes: &'ast mut Vec<YamlNodeId>,
    state: StateId,
    template_delimiters: &'cfg [TemplateDelimiter],
    template_spans_possible: bool,
    inner_trivia: Vec<YamlTrivia<'src>>,
}

#[derive(Debug, Clone, Copy)]
struct FlowNodeProperties {
    start: usize,
    tag: Option<Span>,
    anchor: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
enum FlowImplicitMappingStart {
    Reparse,
    PlainKey { end: usize },
}

impl FlowNodeProperties {
    fn empty_at(pos: usize) -> Self {
        Self {
            start: pos,
            tag: None,
            anchor: None,
        }
    }
}

impl<'src, 'ast, 'cfg> FlowParser<'src, 'ast, 'cfg> {
    fn parse_value(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        let properties = self.parse_node_properties();
        if self.value_is_empty_node() {
            return Some(self.push_empty_flow_scalar(properties));
        }
        let id = match self.peek_byte()? {
            b'[' => self.parse_sequence(),
            b'{' => self.parse_mapping(),
            _ => self.parse_scalar_until(b",]}", properties),
        }?;
        self.apply_node_properties(id, properties);
        Some(id)
    }

    fn parse_sequence_entry(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        if let Some(start) = self.consume_explicit_key_indicator() {
            self.parse_sequence_entry_mapping(start)
        } else if self.sequence_entry_starts_with_collection() {
            self.parse_sequence_entry_collection()
        } else if let Some(mapping_start) = self.sequence_entry_implicit_mapping_start() {
            match mapping_start {
                FlowImplicitMappingStart::Reparse => self.parse_implicit_mapping(),
                FlowImplicitMappingStart::PlainKey { end } => {
                    self.parse_plain_implicit_mapping(end)
                }
            }
        } else {
            self.parse_value()
        }
    }

    fn parse_sequence_entry_collection(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_value()?;
        self.skip_ws();
        if self.consume_byte(b':').is_none() {
            return Some(key);
        }
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_sequence(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.consume_byte(b'[')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.consume_byte(b']').is_some() {
            let id = self.push_flow_sequence(start, entries);
            self.mark_flow_inner_trivia(id, trivia_start);
            return Some(id);
        }

        loop {
            entries.push(self.parse_sequence_entry()?);
            self.skip_ws();
            if self.consume_byte(b',').is_some() {
                self.skip_ws();
                if self.consume_byte(b']').is_some() {
                    break;
                }
                continue;
            }
            self.consume_byte(b']')?;
            break;
        }

        let id = self.push_flow_sequence(start, entries);
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_mapping(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.consume_byte(b'{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.consume_byte(b'}').is_some() {
            let id = self.push_flow_mapping(start, pairs, true);
            self.mark_flow_inner_trivia(id, trivia_start);
            return Some(id);
        }

        loop {
            let pair_start = self.pos;
            let explicit = self.consume_explicit_key_indicator().is_some();
            let key = self.parse_flow_key()?;
            self.skip_ws();
            let value = if self.consume_byte(b':').is_some() {
                Some(self.parse_optional_flow_value()?)
            } else {
                Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
            };
            pairs.push(YamlFlowPair {
                key,
                value,
                explicit,
                source: SourceSpan::new(Span::new(self.base + pair_start, self.base + self.pos)),
            });
            self.skip_ws();
            if self.consume_byte(b',').is_some() {
                self.skip_ws();
                if self.consume_byte(b'}').is_some() {
                    break;
                }
                continue;
            }
            self.consume_byte(b'}')?;
            break;
        }

        let id = self.push_flow_mapping(start, pairs, true);
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_implicit_mapping(&mut self) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_flow_key()?;
        self.skip_ws();
        self.consume_byte(b':')?;
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_plain_implicit_mapping(&mut self, key_end: usize) -> Option<YamlNodeId> {
        let start = self.pos;
        let trivia_start = self.inner_trivia.len();
        self.skip_ws();
        let properties = self.parse_node_properties();
        let key_start = self.pos;
        self.pos = key_end;
        let key = self.push_flow_scalar(key_start, properties)?;
        self.apply_node_properties(key, properties);
        self.skip_ws();
        self.consume_byte(b':')?;
        let value = Some(self.parse_optional_flow_value()?);
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: false,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_sequence_entry_mapping(&mut self, start: usize) -> Option<YamlNodeId> {
        let trivia_start = self.inner_trivia.len();
        let key = self.parse_flow_key()?;
        self.skip_ws();
        let value = if self.consume_byte(b':').is_some() {
            Some(self.parse_optional_flow_value()?)
        } else {
            Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
        };
        let id = self.push_flow_mapping(
            start,
            vec![YamlFlowPair {
                key,
                value,
                explicit: true,
                source: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
            }],
            false,
        );
        self.mark_flow_inner_trivia(id, trivia_start);
        Some(id)
    }

    fn parse_flow_key(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        let properties = self.parse_node_properties();
        if self.key_is_empty_node() {
            return Some(self.push_empty_flow_scalar(properties));
        }
        let id = match self.peek_byte()? {
            b'[' => self.parse_sequence(),
            b'{' => self.parse_mapping(),
            b'\'' | b'"' => self.parse_scalar_until(b":", properties),
            _ => self.parse_plain_flow_key_scalar(properties),
        }?;
        self.apply_node_properties(id, properties);
        Some(id)
    }

    fn parse_plain_flow_key_scalar(
        &mut self,
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let start = self.pos;
        let (end, _) = self.plain_flow_key_end(start)?;
        self.pos = end;
        self.push_flow_scalar(start, properties)
    }

    fn parse_optional_flow_value(&mut self) -> Option<YamlNodeId> {
        self.skip_ws();
        if self.value_is_empty_node() {
            Some(self.push_empty_flow_scalar(FlowNodeProperties::empty_at(self.pos)))
        } else {
            self.parse_value()
        }
    }

    fn value_is_empty_node(&self) -> bool {
        matches!(self.peek_byte(), None | Some(b',' | b']' | b'}'))
    }

    fn key_is_empty_node(&self) -> bool {
        matches!(self.peek_byte(), None | Some(b':' | b',' | b']' | b'}'))
    }

    fn sequence_entry_implicit_mapping_start(&self) -> Option<FlowImplicitMappingStart> {
        let start = self.node_properties_end_from(self.pos);
        match self.text.as_bytes().get(start).copied() {
            Some(b':') => Some(FlowImplicitMappingStart::Reparse),
            Some(b'[' | b'{') => None,
            Some(quote @ (b'\'' | b'"')) => {
                let close = quoted_scalar_close(self.text, start, quote)?;
                (self.text.as_bytes().get(self.skip_ws_from(close + 1)) == Some(&b':'))
                    .then_some(FlowImplicitMappingStart::Reparse)
            }
            _ => {
                let (end, has_colon) = self.plain_flow_key_end(start)?;
                has_colon.then_some(FlowImplicitMappingStart::PlainKey { end })
            }
        }
    }

    fn sequence_entry_starts_with_collection(&self) -> bool {
        let start = self.node_properties_end_from(self.pos);
        matches!(self.text.as_bytes().get(start), Some(b'[' | b'{'))
    }

    fn explicit_key_indicator_at(&self, pos: usize) -> bool {
        if self.text.as_bytes().get(pos) != Some(&b'?') {
            return false;
        }
        self.text
            .as_bytes()
            .get(pos + 1)
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}'))
    }

    fn consume_explicit_key_indicator(&mut self) -> Option<usize> {
        let start = self.pos;
        if !self.explicit_key_indicator_at(start) {
            return None;
        }
        self.pos += 1;
        Some(start)
    }

    fn parse_scalar_until(
        &mut self,
        delimiters: &[u8],
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let start = self.pos;
        let mut in_single = false;
        let mut in_double = false;
        let mut escaped = false;

        while self.pos < self.text.len() {
            if in_double {
                let byte = self.peek_byte()?;
                self.advance_flow_char()?;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_double = false;
                }
                continue;
            }
            if in_single {
                let byte = self.peek_byte()?;
                self.advance_flow_char()?;
                if byte == b'\'' {
                    if self.peek_byte() == Some(b'\'') {
                        self.pos += 1;
                    } else {
                        in_single = false;
                    }
                }
                continue;
            }

            let byte = self.peek_byte()?;
            if byte == b'\'' {
                in_single = true;
                self.pos += 1;
                continue;
            }
            if byte == b'"' {
                in_double = true;
                self.pos += 1;
                continue;
            }
            if self.template_spans_possible
                && let Some(end) = self.template_span_end_at(self.pos)
            {
                self.pos = end;
                continue;
            }
            if byte == b'#' && comment_can_start(self.text, start, self.pos) {
                self.push_comment_trivia();
                break;
            }
            if delimiters.contains(&byte) {
                break;
            }
            self.advance_flow_char()?;
        }

        if in_single || in_double {
            return None;
        }

        self.push_flow_scalar(start, properties)
    }

    fn push_flow_scalar(
        &mut self,
        start: usize,
        properties: FlowNodeProperties,
    ) -> Option<YamlNodeId> {
        let (trimmed_start, trimmed_end) = trim_ascii_range(self.text, start, self.pos)?;
        let content = Span::new(self.base + trimmed_start, self.base + trimmed_end);
        let span = Span::new(self.base + properties.start, content.end);
        let value = self.source_slice(content);
        if value.as_bytes().first() == Some(&b'*') && alias_scalar(value) {
            return Some(self.ast.push_node(YamlAstNode::semantic(
                YamlAstKind::Alias(YamlAlias {
                    value: SourceSpan::new(content),
                    trailing_comment: None,
                }),
                span,
                Vec::new(),
                self.state,
            )));
        }
        let style = scalar_style(value);
        let tag = properties.tag.map(|tag| self.source_slice(tag));
        let semantic = scalar_semantic_with_tag_and_style(value, tag, style);
        let mut node = YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style,
                semantic,
                value: SourceSpan::new(content),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: properties.tag.map(SourceSpan::new),
                anchor: properties.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            self.state,
        );
        node.emit = YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        let id = self.ast.push_node(node);
        Some(id)
    }

    fn push_empty_flow_scalar(&mut self, properties: FlowNodeProperties) -> YamlNodeId {
        let value = Span::empty(self.base + self.pos);
        let span = Span::new(self.base + properties.start, value.end);
        let mut node = YamlAstNode::semantic(
            YamlAstKind::Scalar(YamlScalar {
                style: YamlScalarStyle::Plain,
                semantic: empty_scalar_semantic_with_tag(
                    properties.tag.map(|tag| self.source_slice(tag)),
                ),
                value: SourceSpan::new(value),
                header: None,
                block_header: None,
                body: None,
                nested: None,
                tag: properties.tag.map(SourceSpan::new),
                anchor: properties.anchor.map(SourceSpan::new),
                trailing_comment: None,
            }),
            span,
            Vec::new(),
            self.state,
        );
        node.emit = YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        self.ast.push_node(node)
    }

    fn push_flow_sequence(&mut self, start: usize, entries: Vec<YamlNodeId>) -> YamlNodeId {
        let value = Span::new(self.base + start, self.base + self.pos);
        let id = self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::FlowSequence(YamlFlowSequence {
                value: SourceSpan::new(value),
                entries: entries.into_boxed_slice(),
                tag: None,
                anchor: None,
                trailing_comment: None,
                has_inner_trivia: false,
                inner_trivia: Box::new([]),
            }),
            value,
            Vec::new(),
            self.state,
        ));
        self.collection_nodes.push(id);
        id
    }

    fn mark_flow_inner_trivia(&mut self, id: YamlNodeId, trivia_start: usize) {
        if self.inner_trivia.len() == trivia_start {
            return;
        }
        let inner_trivia = self.inner_trivia[trivia_start..]
            .to_vec()
            .into_boxed_slice();
        match &mut self.ast.node_mut(id).kind {
            YamlAstKind::FlowSequence(sequence) => {
                sequence.has_inner_trivia = true;
                sequence.inner_trivia = inner_trivia;
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.has_inner_trivia = true;
                mapping.inner_trivia = inner_trivia;
            }
            _ => {}
        }
    }

    fn push_flow_mapping(
        &mut self,
        start: usize,
        pairs: Vec<YamlFlowPair<'src>>,
        braced: bool,
    ) -> YamlNodeId {
        let value = Span::new(self.base + start, self.base + self.pos);
        let id = self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::FlowMapping(YamlFlowMapping {
                value: SourceSpan::new(value),
                pairs: pairs.into_boxed_slice(),
                braced,
                tag: None,
                anchor: None,
                trailing_comment: None,
                has_inner_trivia: false,
                inner_trivia: Box::new([]),
            }),
            value,
            Vec::new(),
            self.state,
        ));
        self.collection_nodes.push(id);
        id
    }

    fn parse_node_properties(&mut self) -> FlowNodeProperties {
        let start = self.pos;
        if !matches!(self.peek_byte(), Some(b'!' | b'&')) {
            return FlowNodeProperties {
                start,
                tag: None,
                anchor: None,
            };
        }
        let mut tag = None;
        let mut anchor = None;

        loop {
            self.skip_ws();
            let token_start = self.pos;
            let Some(marker @ (b'!' | b'&')) = self.peek_byte() else {
                break;
            };
            while self.text.as_bytes().get(self.pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                self.pos += 1;
            }
            if self.text.as_bytes().get(self.pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                self.pos = token_start;
                break;
            }
            let span = Span::new(self.base + token_start, self.base + self.pos);
            if marker == b'!' {
                tag = Some(span);
            } else {
                anchor = Some(span);
            }
        }

        FlowNodeProperties { start, tag, anchor }
    }

    fn apply_node_properties(&mut self, id: YamlNodeId, properties: FlowNodeProperties) {
        if properties.tag.is_none() && properties.anchor.is_none() {
            return;
        }

        let start = self.base + properties.start;
        let node = self.ast.node_mut(id);
        node.span.set_start(start);
        match &mut node.kind {
            YamlAstKind::Scalar(scalar) => {
                scalar.tag = properties.tag.map(SourceSpan::new);
                scalar.anchor = properties.anchor.map(SourceSpan::new);
            }
            YamlAstKind::FlowSequence(sequence) => {
                sequence.tag = properties.tag.map(SourceSpan::new);
                sequence.anchor = properties.anchor.map(SourceSpan::new);
                sequence.value.set_start(start);
            }
            YamlAstKind::FlowMapping(mapping) => {
                mapping.tag = properties.tag.map(SourceSpan::new);
                mapping.anchor = properties.anchor.map(SourceSpan::new);
                mapping.value.set_start(start);
            }
            _ => {}
        }
    }

    fn source_slice(&self, span: Span) -> &str {
        &self.text[span.start - self.base..span.end - self.base]
    }

    fn template_span_end_at(&self, pos: usize) -> Option<usize> {
        if !self.template_spans_possible {
            return None;
        }
        let byte = *self.text.as_bytes().get(pos)?;
        for delimiter in self.template_delimiters {
            let open = delimiter.open.as_bytes();
            if open.is_empty() || delimiter.close.is_empty() || open[0] != byte {
                continue;
            }
            if !self.text[pos..].starts_with(&delimiter.open) {
                continue;
            }
            let content_start = pos + delimiter.open.len();
            if let Some(close) = self.text[content_start..].find(&delimiter.close) {
                return Some(content_start + close + delimiter.close.len());
            }
        }
        None
    }

    fn skip_ws_from(&self, mut pos: usize) -> usize {
        while self
            .text
            .as_bytes()
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }
        pos
    }

    fn node_properties_end_from(&self, mut pos: usize) -> usize {
        loop {
            pos = self.skip_ws_from(pos);
            let token_start = pos;
            let Some(b'!' | b'&') = self.text.as_bytes().get(pos).copied() else {
                return pos;
            };
            while self.text.as_bytes().get(pos).is_some_and(|byte| {
                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b']' | b'}' | b':')
            }) {
                pos += 1;
            }
            let valid_property = self.text.as_bytes().get(pos).is_none_or(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}' | b':')
            });
            if !valid_property {
                return token_start;
            }
        }
    }

    fn plain_flow_key_end(&self, start: usize) -> Option<(usize, bool)> {
        let mut pos = start;
        let mut flow_depth = 0usize;
        let mut in_single = false;
        let mut in_double = false;
        let mut escaped = false;

        while pos < self.text.len() {
            let ch = self.text[pos..].chars().next()?;
            let ch_len = ch.len_utf8();
            if in_double {
                pos += ch_len;
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_double = false;
                }
                continue;
            }
            if in_single {
                pos += ch_len;
                if ch == '\'' {
                    if self.text[pos..].starts_with('\'') {
                        pos += '\''.len_utf8();
                    } else {
                        in_single = false;
                    }
                }
                continue;
            }

            if self.template_spans_possible
                && let Some(end) = self.template_span_end_at(pos)
            {
                pos = end;
                continue;
            }

            match ch {
                '\'' => in_single = true,
                '"' => in_double = true,
                '#' if flow_depth == 0 && comment_can_start(self.text, start, pos) => break,
                '[' | '{' => flow_depth += 1,
                ']' | '}' if flow_depth > 0 => flow_depth -= 1,
                ':' if flow_depth == 0 && flow_colon_is_value_indicator(self.text, pos, false) => {
                    return Some((pos, true));
                }
                ',' | ']' | '}' if flow_depth == 0 => return Some((pos, false)),
                _ => {}
            }
            pos += ch_len;
        }

        if in_single || in_double || flow_depth > 0 {
            None
        } else {
            Some((pos, false))
        }
    }

    fn skip_ws(&mut self) {
        loop {
            while let Some(byte) = self.text.as_bytes().get(self.pos).copied()
                && byte.is_ascii_whitespace()
            {
                if matches!(byte, b'\r' | b'\n') {
                    let newline_start = self.pos;
                    let newline_end = if byte == b'\r'
                        && self.text.as_bytes().get(self.pos + 1) == Some(&b'\n')
                    {
                        self.pos + 2
                    } else {
                        self.pos + 1
                    };
                    let line_start = self.line_start_before(newline_start);
                    if self.text[line_start..newline_start].trim().is_empty() {
                        self.inner_trivia.push(YamlTrivia {
                            kind: YamlTriviaKind::Blank,
                            span: SourceSpan::new(Span::new(
                                self.base + line_start,
                                self.base + newline_end,
                            )),
                        });
                    }
                    self.pos = newline_end;
                } else {
                    self.pos += 1;
                }
            }
            if self.peek_byte() == Some(b'#') && comment_can_start(self.text, 0, self.pos) {
                self.push_comment_trivia();
                continue;
            }
            break;
        }
    }

    fn line_start_before(&self, pos: usize) -> usize {
        self.text[..pos]
            .rfind(['\r', '\n'])
            .map(|index| index + 1)
            .unwrap_or(0)
    }

    fn push_comment_trivia(&mut self) {
        let start = self.pos;
        while self
            .text
            .as_bytes()
            .get(self.pos)
            .is_some_and(|byte| !matches!(byte, b'\r' | b'\n'))
        {
            self.pos += 1;
        }
        self.inner_trivia.push(YamlTrivia {
            kind: YamlTriviaKind::Comment,
            span: SourceSpan::new(Span::new(self.base + start, self.base + self.pos)),
        });
    }

    fn consume_byte(&mut self, byte: u8) -> Option<()> {
        if self.peek_byte() == Some(byte) {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn advance_flow_char(&mut self) -> Option<()> {
        let byte = self.peek_byte()?;
        self.pos += if byte.is_ascii() {
            1
        } else {
            self.text[self.pos..].chars().next()?.len_utf8()
        };
        Some(())
    }
}

fn trim_ascii_range(text: &str, mut start: usize, mut end: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start < end).then_some((start, end))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct YamlEmissionStats {
    pub emitted_nodes: usize,
}

enum ExternalBlockScalarAction {
    Preserve,
    Formatted(String),
}

trait IntoOptionalSpan {
    fn into_optional_span(self) -> Option<Span>;
}

impl IntoOptionalSpan for Option<Span> {
    fn into_optional_span(self) -> Option<Span> {
        self
    }
}

impl IntoOptionalSpan for Option<SourceSpan<'_>> {
    fn into_optional_span(self) -> Option<Span> {
        self.map(SourceSpan::span)
    }
}

pub fn emit_yaml_document(
    source: &SourceBuffer,
    document: &Document<'_>,
    options: FormatOptions,
    plugins: &PluginRegistry,
) -> Result<String> {
    emit_yaml_document_with_stats(source, document, options, plugins).map(|(output, _)| output)
}

pub fn emit_yaml_document_with_stats(
    source: &SourceBuffer,
    document: &Document<'_>,
    options: FormatOptions,
    plugins: &PluginRegistry,
) -> Result<(String, YamlEmissionStats)> {
    let Some(ast) = document.yaml.as_ref() else {
        return Ok((
            source.slice(document.range).to_owned(),
            YamlEmissionStats::default(),
        ));
    };
    let mut out = String::with_capacity(ast.range.len());
    let mut stats = YamlEmissionStats::default();
    let context = YamlEmitContext {
        source,
        document,
        ast,
        options,
        plugins,
    };
    for root in &ast.roots {
        if let Some(node) = root.node {
            emit_yaml_node(&mut out, context, node, None, &mut stats)?;
        }
    }
    emit_trivia(&mut out, source, &ast.trailing_trivia);
    restore_yaml_bom(source, document, &mut out);
    Ok((out, stats))
}

fn restore_yaml_bom(source: &SourceBuffer, document: &Document, out: &mut String) {
    let Some(bom) = source.bom else {
        return;
    };
    if document.range.start != bom.start || out.starts_with('\u{feff}') {
        return;
    }
    let placeholder = " ".repeat(bom.len());
    if out.starts_with(&placeholder) {
        out.replace_range(0..placeholder.len(), "\u{feff}");
    } else {
        out.insert(0, '\u{feff}');
    }
}

#[derive(Clone, Copy)]
struct YamlEmitContext<'a> {
    source: &'a SourceBuffer,
    document: &'a Document<'a>,
    ast: &'a YamlDocumentAst<'a>,
    options: FormatOptions,
    plugins: &'a PluginRegistry,
}

impl YamlEmitContext<'_> {
    fn with_options(self, options: FormatOptions) -> Self {
        Self { options, ..self }
    }
}

fn emit_yaml_node(
    out: &mut String,
    context: YamlEmitContext<'_>,
    id: YamlNodeId,
    forced_indent: Option<usize>,
    stats: &mut YamlEmissionStats,
) -> Result<()> {
    stats.emitted_nodes += 1;
    let YamlEmitContext {
        source,
        document,
        ast,
        options,
        plugins,
    } = context;
    let node = ast.node(id);
    emit_trivia(out, source, &node.leading_trivia);
    if matches!(node.emit, YamlEmitPlan::PreserveSource) {
        out.push_str(source.slice(node.span));
        return Ok(());
    }
    let state = document.state(node.state);
    let options = state.yaml_options(options);
    if let YamlEmitPlan::Rendered(
        YamlRenderedKind::EmptyMarkdownScalar
        | YamlRenderedKind::InlineMarkdownScalar
        | YamlRenderedKind::Scalar,
    ) = &node.emit
        && let YamlAstKind::Scalar(scalar) = &node.kind
    {
        emit_yaml_rendered_scalar_plan(out, source, scalar, node, state, options, None);
        return Ok(());
    }
    if matches!(
        node.emit,
        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
    ) && yaml_node_is_root(ast, id)
        && compact_root_collection_allowed(ast, id)
    {
        emit_compact_yaml_node(out, source, document, ast, id)
            .expect("planned compact YAML collection should render during emission");
        out.push_str(line_ending_for_span(source, node.span));
        return Ok(());
    }
    if let (YamlAstKind::Scalar(scalar), YamlEmitPlan::NestedMarkdownBlockScalar { nested }) =
        (&node.kind, &node.emit)
    {
        emit_yaml_nested_markdown_block_scalar(
            out,
            source,
            document,
            scalar,
            node,
            options,
            plugins,
            *nested as usize,
        )?;
        return Ok(());
    }
    let context = context.with_options(options);

    match &node.kind {
        YamlAstKind::Empty => {}
        YamlAstKind::Opaque(opaque) => {
            let _ = opaque.reason;
            out.push_str(source.slice(node.span));
        }
        YamlAstKind::Scalar(scalar) => {
            emit_yaml_scalar(out, source, document, scalar, node, options, plugins)?;
        }
        YamlAstKind::Alias(alias) => {
            emit_yaml_alias(out, source, alias, node);
        }
        YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
            let indent = forced_indent.unwrap_or_else(|| {
                let line = source.line_at_byte(node.span.start());
                indentation(source.line_text(line))
            });
            emit_yaml_flow_collection(out, source, document, ast, id, node, indent, options);
        }
        YamlAstKind::Mapping(mapping) => {
            let indent = forced_indent.unwrap_or(mapping.indent);
            for (index, pair) in mapping.pairs.iter().enumerate() {
                let emit_leading = !(index == 0 && pair.leading_trivia == node.leading_trivia);
                emit_yaml_mapping_pair(
                    out,
                    source,
                    document,
                    ast,
                    pair,
                    emit_leading,
                    YamlLinePrefix::Spaces(indent),
                    indent + options.indent_width,
                    indent + options.indent_width,
                    options,
                    plugins,
                    stats,
                )?;
            }
        }
        YamlAstKind::Sequence(sequence) => {
            let indent = forced_indent.unwrap_or(sequence.indent);
            if let Some(compact_table) = document.state(node.state).table_compact {
                if matches!(node.emit, YamlEmitPlan::Rendered(YamlRenderedKind::Table)) {
                    emit_flow_table_sequence_into(
                        out,
                        source,
                        document,
                        ast,
                        sequence,
                        indent,
                        compact_table,
                        &node.leading_trivia,
                    )
                    .expect("planned YAML table should render during emission");
                } else {
                    if let Some(first) = sequence.items.first()
                        && first.leading_trivia != node.leading_trivia
                    {
                        emit_trivia(out, source, &first.leading_trivia);
                    }
                    out.push_str(source.slice(node.span));
                }
                return Ok(());
            }
            for (index, item) in sequence.items.iter().enumerate() {
                if !(index == 0 && item.leading_trivia == node.leading_trivia) {
                    emit_trivia(out, source, &item.leading_trivia);
                }
                let Some(value) = item.value else {
                    out.push_str(source.slice(item.line));
                    continue;
                };
                let value_node = ast.node(value);
                if yaml_block_collection_has_flow_collapse_hint(value_node)
                    && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
                {
                    out.push_str(source.slice(value_node.span));
                    continue;
                }
                if matches!(
                    value_node.kind,
                    YamlAstKind::Scalar(_)
                        | YamlAstKind::Alias(_)
                        | YamlAstKind::FlowSequence(_)
                        | YamlAstKind::FlowMapping(_)
                ) && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
                {
                    out.push_str(source.slice(value_node.span));
                    continue;
                }
                if document.state(value_node.state).preserve {
                    if matches!(
                        value_node.kind,
                        YamlAstKind::Scalar(_)
                            | YamlAstKind::Alias(_)
                            | YamlAstKind::FlowSequence(_)
                            | YamlAstKind::FlowMapping(_)
                    ) {
                        out.push_str(source.slice(value_node.span));
                    } else {
                        emit_spaces(out, indent);
                        out.push('-');
                        emit_yaml_node_properties(out, source, value_node);
                        emit_inline_comment(out, source, item.trailing_comment);
                        out.push_str(line_ending_for_span(source, item.line));
                        emit_yaml_node(
                            out,
                            context,
                            value,
                            Some(indent + options.indent_width),
                            stats,
                        )?;
                    }
                    continue;
                }

                if let (YamlAstKind::Scalar(scalar), YamlEmitPlan::ExternalBlockScalar) =
                    (&value_node.kind, &value_node.emit)
                    && let Some(name) = document
                        .state(value_node.state)
                        .embedded_formatter
                        .as_deref()
                    && let Some(action) = external_block_scalar_action(
                        source, scalar, value_node, options, plugins, name,
                    )?
                {
                    match action {
                        ExternalBlockScalarAction::Preserve => {
                            out.push_str(source.slice(value_node.span));
                        }
                        ExternalBlockScalarAction::Formatted(formatted) => {
                            emit_spaces(out, indent);
                            out.push_str("- ");
                            emit_yaml_formatted_external_block_scalar_after_prefix(
                                out, source, scalar, value_node, &formatted, options,
                            );
                        }
                    }
                    continue;
                }

                if matches!(
                    value_node.kind,
                    YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
                ) && item.trailing_comment.is_none()
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                {
                    emit_spaces(out, indent);
                    out.push_str("- ");
                    emit_compact_yaml_node(out, source, document, ast, value)
                        .expect("planned compact YAML collection should render during emission");
                    out.push_str(line_ending_for_span(source, item.line));
                    continue;
                }

                if let YamlAstKind::Mapping(mapping) = &value_node.kind
                    && item.value_on_marker_line
                {
                    emit_yaml_inline_sequence_mapping(
                        out, source, document, ast, mapping, item, indent, options, plugins, stats,
                    )?;
                    continue;
                }

                emit_spaces(out, indent);
                out.push('-');
                match &value_node.kind {
                    YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
                        emit_yaml_node_properties(out, source, value_node);
                        emit_inline_comment(out, source, item.trailing_comment);
                        out.push_str(line_ending_for_span(source, item.line));
                        emit_yaml_node(
                            out,
                            context,
                            value,
                            Some(indent + options.indent_width),
                            stats,
                        )?;
                    }
                    YamlAstKind::Scalar(scalar) if scalar.header.is_some() => {
                        out.push(' ');
                        stats.emitted_nodes += 1;
                        emit_yaml_scalar_after_prefix(
                            out,
                            context,
                            scalar,
                            value_node,
                            Some(indent + options.indent_width),
                        )?;
                    }
                    YamlAstKind::Scalar(scalar) if scalar.value.is_empty() => {
                        if let YamlEmitPlan::Rendered(
                            YamlRenderedKind::EmptyMarkdownScalar
                            | YamlRenderedKind::InlineMarkdownScalar
                            | YamlRenderedKind::Scalar,
                        ) = &value_node.emit
                        {
                            out.push(' ');
                            let value_state = document.state(value_node.state);
                            emit_yaml_rendered_scalar_plan(
                                out,
                                source,
                                scalar,
                                value_node,
                                value_state,
                                options,
                                None,
                            );
                        } else {
                            emit_inline_comment(out, source, item.trailing_comment);
                            out.push_str(line_ending_for_span(source, item.line));
                        }
                    }
                    YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                        if matches!(
                            value_node.emit,
                            YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection)
                        ) {
                            let newline = line_ending_or_default(source, item.line, options);
                            if matches!(value_node.kind, YamlAstKind::FlowMapping(_)) {
                                emit_yaml_flow_collection_block_into(
                                    out,
                                    source,
                                    document,
                                    ast,
                                    value,
                                    indent + options.indent_width,
                                    newline,
                                    options,
                                    Some(" "),
                                )
                                .expect(
                                    "planned block YAML collection should render during emission",
                                );
                            } else {
                                out.push_str(newline);
                                emit_yaml_flow_collection_block_into(
                                    out,
                                    source,
                                    document,
                                    ast,
                                    value,
                                    indent + options.indent_width,
                                    newline,
                                    options,
                                    None,
                                )
                                .expect(
                                    "planned block YAML collection should render during emission",
                                );
                            }
                        } else {
                            out.push(' ');
                            emit_yaml_node(out, context, value, None, stats)?;
                        }
                    }
                    _ => {
                        out.push(' ');
                        emit_yaml_node(out, context, value, None, stats)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_inline_sequence_mapping(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    mapping: &YamlMapping<'_>,
    item: &YamlSequenceItem<'_>,
    indent: usize,
    options: FormatOptions,
    plugins: &PluginRegistry,
    stats: &mut YamlEmissionStats,
) -> Result<()> {
    let Some((first, rest)) = mapping.pairs.split_first() else {
        emit_spaces(out, indent);
        out.push('-');
        emit_inline_comment(out, source, item.trailing_comment);
        out.push_str(line_ending_for_span(source, item.line));
        return Ok(());
    };

    let first_prefix = format!("{}- ", " ".repeat(indent));
    let continuation_prefix = " ".repeat(indent + options.indent_width);
    if first.explicit {
        emit_trivia(out, source, &first.leading_trivia);
        emit_yaml_explicit_mapping_pair(
            out,
            source,
            document,
            ast,
            first,
            YamlLinePrefix::Text(&first_prefix),
            YamlLinePrefix::Text(&continuation_prefix),
            indent + options.indent_width,
            indent + options.indent_width * 2,
            options,
            plugins,
            stats,
        )?;
    } else {
        emit_yaml_mapping_pair(
            out,
            source,
            document,
            ast,
            first,
            true,
            YamlLinePrefix::Text(&first_prefix),
            indent + options.indent_width,
            indent + options.indent_width * 2,
            options,
            plugins,
            stats,
        )?;
    }
    for pair in rest {
        emit_yaml_mapping_pair(
            out,
            source,
            document,
            ast,
            pair,
            true,
            YamlLinePrefix::Text(&continuation_prefix),
            indent + options.indent_width,
            indent + options.indent_width * 2,
            options,
            plugins,
            stats,
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum YamlLinePrefix<'a> {
    Spaces(usize),
    Text(&'a str),
}

fn emit_yaml_line_prefix(out: &mut String, prefix: YamlLinePrefix<'_>) {
    match prefix {
        YamlLinePrefix::Spaces(count) => emit_spaces(out, count),
        YamlLinePrefix::Text(text) => out.push_str(text),
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_mapping_pair(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    pair: &YamlMappingPair<'_>,
    emit_leading: bool,
    prefix: YamlLinePrefix<'_>,
    child_indent: usize,
    scalar_body_indent: usize,
    options: FormatOptions,
    plugins: &PluginRegistry,
    stats: &mut YamlEmissionStats,
) -> Result<()> {
    if emit_leading {
        emit_trivia(out, source, &pair.leading_trivia);
    }
    if pair.explicit {
        emit_yaml_explicit_mapping_pair(
            out,
            source,
            document,
            ast,
            pair,
            prefix,
            prefix,
            child_indent,
            scalar_body_indent,
            options,
            plugins,
            stats,
        )?;
        return Ok(());
    }
    let Some(value) = pair.value else {
        out.push_str(source.slice(pair.line));
        return Ok(());
    };
    let value_node = ast.node(value);
    let context = YamlEmitContext {
        source,
        document,
        ast,
        options,
        plugins,
    };
    if yaml_block_collection_has_flow_collapse_hint(value_node)
        && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
    {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }
    if matches!(
        value_node.kind,
        YamlAstKind::Scalar(_)
            | YamlAstKind::Alias(_)
            | YamlAstKind::FlowSequence(_)
            | YamlAstKind::FlowMapping(_)
    ) && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
    {
        out.push_str(source.slice(value_node.span));
        return Ok(());
    }
    if let (YamlAstKind::Scalar(scalar), YamlEmitPlan::ExternalBlockScalar) =
        (&value_node.kind, &value_node.emit)
        && let Some(name) = document
            .state(value_node.state)
            .embedded_formatter
            .as_deref()
        && let Some(action) =
            external_block_scalar_action(source, scalar, value_node, options, plugins, name)?
    {
        match action {
            ExternalBlockScalarAction::Preserve => {
                out.push_str(source.slice(value_node.span));
            }
            ExternalBlockScalarAction::Formatted(formatted) => {
                emit_yaml_line_prefix(out, prefix);
                let key = source.slice(pair.key).trim();
                out.push_str(key);
                out.push(':');
                out.push(' ');
                emit_yaml_formatted_external_block_scalar_after_prefix(
                    out, source, scalar, value_node, &formatted, options,
                );
            }
        }
        return Ok(());
    }
    if document.state(value_node.state).preserve {
        if matches!(
            value_node.kind,
            YamlAstKind::Scalar(_)
                | YamlAstKind::Alias(_)
                | YamlAstKind::FlowSequence(_)
                | YamlAstKind::FlowMapping(_)
        ) {
            out.push_str(source.slice(value_node.span));
        } else {
            emit_yaml_line_prefix(out, prefix);
            out.push_str(source.slice(pair.key).trim());
            out.push(':');
            emit_yaml_node_properties(out, source, value_node);
            emit_inline_comment(out, source, pair.trailing_comment);
            out.push_str(line_ending_for_span(source, pair.line));
            emit_yaml_node(
                out,
                context,
                value,
                Some(mapping_pair_child_indent(value_node, child_indent)),
                stats,
            )?;
        }
        return Ok(());
    }

    emit_yaml_line_prefix(out, prefix);
    let key = source.slice(pair.key).trim();
    out.push_str(key);
    out.push(':');
    emit_yaml_mapping_value_after_colon(
        out,
        context,
        pair,
        value,
        child_indent,
        scalar_body_indent,
        stats,
    )
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_explicit_mapping_pair(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    pair: &YamlMappingPair<'_>,
    key_prefix: YamlLinePrefix<'_>,
    value_prefix: YamlLinePrefix<'_>,
    child_indent: usize,
    scalar_body_indent: usize,
    options: FormatOptions,
    plugins: &PluginRegistry,
    stats: &mut YamlEmissionStats,
) -> Result<()> {
    let Some(value) = pair.value else {
        out.push_str(source.slice(pair.source));
        return Ok(());
    };
    if pair.colon.is_empty() {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }

    let key_line_index = source.line_at_byte(pair.source.start());
    let value_line_index = source.line_at_byte(pair.colon.start());
    if value_line_index != key_line_index + 1
        || pair
            .key_node
            .is_some_and(|key| ast.node(key).span.end() > source.lines[key_line_index].full.end())
    {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }

    let value_node = ast.node(value);
    if yaml_block_collection_has_flow_collapse_hint(value_node)
        && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
    {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }
    if document.state(value_node.state).preserve
        || (matches!(
            value_node.kind,
            YamlAstKind::Scalar(_)
                | YamlAstKind::Alias(_)
                | YamlAstKind::FlowSequence(_)
                | YamlAstKind::FlowMapping(_)
        ) && matches!(value_node.emit, YamlEmitPlan::PreserveSource))
    {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }

    let external_action = if let (YamlAstKind::Scalar(scalar), YamlEmitPlan::ExternalBlockScalar) =
        (&value_node.kind, &value_node.emit)
    {
        if let Some(name) = document
            .state(value_node.state)
            .embedded_formatter
            .as_deref()
        {
            external_block_scalar_action(source, scalar, value_node, options, plugins, name)?
        } else {
            None
        }
    } else {
        None
    };
    if matches!(external_action, Some(ExternalBlockScalarAction::Preserve)) {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }

    let key_line = source.lines[key_line_index].full;
    let (key_body, key_newline) = strip_newline(source.slice(key_line));
    let key_body = explicit_mapping_key_body(source, pair, key_line_index).unwrap_or(key_body);
    emit_yaml_line_prefix(out, key_prefix);
    out.push_str(key_body.trim_end());
    if key_newline.is_empty() {
        out.push_str(line_ending_or_default(source, pair.line, options));
    } else {
        out.push_str(key_newline);
    }

    emit_yaml_line_prefix(out, value_prefix);
    out.push(':');

    let context = YamlEmitContext {
        source,
        document,
        ast,
        options,
        plugins,
    };
    if let Some(ExternalBlockScalarAction::Formatted(formatted)) = external_action {
        if let YamlAstKind::Scalar(scalar) = &value_node.kind {
            out.push(' ');
            emit_yaml_formatted_external_block_scalar_after_prefix(
                out, source, scalar, value_node, &formatted, options,
            );
        }
        return Ok(());
    }

    emit_yaml_mapping_value_after_colon(
        out,
        context,
        pair,
        value,
        child_indent,
        scalar_body_indent,
        stats,
    )
}

fn explicit_mapping_key_body<'a>(
    source: &'a SourceBuffer,
    pair: &YamlMappingPair<'_>,
    key_line_index: usize,
) -> Option<&'a str> {
    let text = source.line_text(key_line_index);
    let key_start = pair
        .key
        .start()
        .checked_sub(source.lines[key_line_index].text.start())?;
    let marker = text[..key_start].rfind('?')?;
    explicit_indicator_at(text, marker, b'?')?;
    text[marker + 1..key_start]
        .chars()
        .all(char::is_whitespace)
        .then_some(&text[marker..])
}

fn emit_yaml_mapping_value_after_colon(
    out: &mut String,
    context: YamlEmitContext<'_>,
    pair: &YamlMappingPair<'_>,
    value: YamlNodeId,
    child_indent: usize,
    scalar_body_indent: usize,
    stats: &mut YamlEmissionStats,
) -> Result<()> {
    let YamlEmitContext {
        source,
        document,
        ast,
        options,
        plugins: _,
    } = context;
    let value_node = ast.node(value);
    match &value_node.kind {
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
            if pair.trailing_comment.is_none()
                && matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                )
            {
                out.push(' ');
                emit_compact_yaml_node(out, source, document, ast, value)
                    .expect("planned compact YAML collection should render during emission");
                out.push_str(line_ending_for_span(source, pair.line));
            } else {
                emit_yaml_node_properties(out, source, value_node);
                emit_inline_comment(out, source, pair.trailing_comment);
                out.push_str(line_ending_for_span(source, pair.line));
                emit_yaml_node(
                    out,
                    context,
                    value,
                    Some(mapping_pair_child_indent(value_node, child_indent)),
                    stats,
                )?;
            }
        }
        YamlAstKind::Scalar(scalar) if scalar.header.is_some() => {
            out.push(' ');
            stats.emitted_nodes += 1;
            emit_yaml_scalar_after_prefix(
                out,
                context,
                scalar,
                value_node,
                Some(mapping_pair_child_indent(value_node, scalar_body_indent)),
            )?;
        }
        YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
            if matches!(
                value_node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection)
            ) {
                let newline = line_ending_for_span(source, pair.line);
                out.push_str(newline);
                emit_yaml_flow_collection_block_into(
                    out,
                    source,
                    document,
                    ast,
                    value,
                    mapping_pair_child_indent(value_node, child_indent),
                    newline,
                    options,
                    None,
                )
                .expect("planned block YAML collection should render during emission");
            } else {
                out.push(' ');
                emit_yaml_node(out, context, value, None, stats)?;
            }
        }
        YamlAstKind::Scalar(scalar) if scalar.value.is_empty() => {
            if let YamlEmitPlan::Rendered(
                YamlRenderedKind::EmptyMarkdownScalar
                | YamlRenderedKind::InlineMarkdownScalar
                | YamlRenderedKind::Scalar,
            ) = &value_node.emit
            {
                out.push(' ');
                let value_state = document.state(value_node.state);
                emit_yaml_rendered_scalar_plan(
                    out,
                    source,
                    scalar,
                    value_node,
                    value_state,
                    options,
                    None,
                );
            } else {
                emit_inline_comment(out, source, pair.trailing_comment);
                out.push_str(line_ending_for_span(source, pair.line));
            }
        }
        YamlAstKind::Scalar(scalar) => {
            out.push(' ');
            stats.emitted_nodes += 1;
            emit_yaml_scalar_after_prefix(
                out,
                context,
                scalar,
                value_node,
                Some(mapping_pair_child_indent(value_node, scalar_body_indent)),
            )?;
        }
        _ => {
            out.push(' ');
            emit_yaml_node(out, context, value, None, stats)?;
        }
    }
    Ok(())
}

fn emit_yaml_scalar(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    plugins: &PluginRegistry,
) -> Result<()> {
    let _ = (scalar.style, scalar.semantic, scalar.tag, scalar.anchor);
    match &node.emit {
        YamlEmitPlan::None => {
            out.push_str(source.slice(node.span));
            Ok(())
        }
        YamlEmitPlan::PreserveSource => {
            out.push_str(source.slice(node.span));
            Ok(())
        }
        YamlEmitPlan::Rendered(_) => {
            let state = document.state(node.state);
            emit_yaml_rendered_scalar_plan(out, source, scalar, node, state, options, None);
            Ok(())
        }
        YamlEmitPlan::NestedMarkdownBlockScalar { nested } => {
            emit_yaml_nested_markdown_block_scalar(
                out,
                source,
                document,
                scalar,
                node,
                options,
                plugins,
                *nested as usize,
            )
        }
        YamlEmitPlan::ExternalBlockScalar => {
            if let Some(name) = document.state(node.state).embedded_formatter.as_deref()
                && let Some(action) =
                    external_block_scalar_action(source, scalar, node, options, plugins, name)?
            {
                match action {
                    ExternalBlockScalarAction::Preserve => {
                        out.push_str(source.slice(node.span));
                    }
                    ExternalBlockScalarAction::Formatted(formatted) => {
                        emit_yaml_formatted_external_block_scalar(
                            out, source, scalar, node, &formatted, options,
                        );
                    }
                }
            }
            Ok(())
        }
    }
}

fn emit_yaml_rendered_scalar_plan(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    state: &crate::core::directives::DirectiveState,
    options: FormatOptions,
    body_indent: Option<usize>,
) {
    match &node.emit {
        YamlEmitPlan::Rendered(YamlRenderedKind::EmptyMarkdownScalar) => {
            out.push_str(&render_empty_markdown_scalar(
                source,
                scalar,
                scalar.trailing_comment,
                line_ending_for_span(source, node.span),
            ));
        }
        YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar) => {
            if let Some(output) =
                render_inline_markdown_scalar(source, scalar, node, state, options, body_indent)
            {
                out.push_str(&output);
            }
        }
        YamlEmitPlan::Rendered(YamlRenderedKind::Scalar) => {
            emit_yaml_scalar_plan_output_into(out, source, scalar, node, options, body_indent);
        }
        _ => unreachable!("non-scalar YAML render plan used for scalar output"),
    }
}

fn emit_yaml_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    body_indent: Option<usize>,
) {
    if let Some(body) = scalar.body {
        output.push_str(&render_yaml_block_scalar_value_header(source, scalar));
        if let Some(rewrapped) = render_rewrapped_folded_block_scalar(source, scalar, node, options)
        {
            output.push_str(&rewrapped);
        } else {
            output.push_str(source.slice(body));
        }
        return;
    }
    if scalar.value.is_empty() && scalar_has_properties(scalar) {
        let (body, newline) = strip_newline(source.slice(node.span));
        output.push_str(body.trim_end());
        output.push_str(newline);
        return;
    }
    if emit_non_string_plain_scalar_plan_output_into(output, source, scalar, node).is_some() {
        return;
    }
    if let Some(rendered) =
        render_quoted_newline_literal_scalar(source, scalar, node, options, body_indent)
    {
        output.push_str(&rendered);
        return;
    }
    if let Some(rendered) = render_folded_prose_scalar(source, scalar, node, options, body_indent) {
        output.push_str(&rendered);
        return;
    }
    if emit_plain_scalar_plan_output_into(output, source, scalar, node).is_some() {
        return;
    }
    if scalar.body.is_none()
        && let Some(normalized) = normalize_core_scalar(source, scalar, false)
    {
        output.push_str(&normalized);
        emit_inline_comment(output, source, scalar.trailing_comment);
        output.push_str(line_ending_for_span(source, node.span));
        return;
    }
    if let Some(simplified) = simplify_quoted_string_scalar(source, scalar, false) {
        output.push_str(&simplified);
        emit_inline_comment(output, source, scalar.trailing_comment);
        output.push_str(line_ending_for_span(source, node.span));
        return;
    }
    if let Some(rendered) = render_unsafe_plain_string_scalar(source, scalar, node) {
        output.push_str(&rendered);
        return;
    }
    output.push_str(source.slice(scalar.value).trim());
    emit_inline_comment(output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
}

fn emit_plain_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
) -> Option<()> {
    if scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.style != YamlScalarStyle::Plain
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    let raw = source.slice(scalar.value).trim();
    match scalar.semantic {
        YamlScalarSemantic::Boolean => {
            let normalized = match raw {
                "true" | "True" | "TRUE" => "true",
                "false" | "False" | "FALSE" => "false",
                _ => return None,
            };
            output.push_str(normalized);
        }
        YamlScalarSemantic::Null => output.push_str("null"),
        _ => {
            if let Some(rendered) = render_unsafe_plain_string_scalar(source, scalar, node) {
                output.push_str(&rendered);
                return Some(());
            }
            output.push_str(raw);
        }
    }
    emit_inline_comment(output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
    Some(())
}

fn emit_non_string_plain_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
) -> Option<()> {
    if scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.style != YamlScalarStyle::Plain
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
        || scalar.semantic == YamlScalarSemantic::String
    {
        return None;
    }
    let raw = source.slice(scalar.value).trim();
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => output.push_str("true"),
            "false" | "False" | "FALSE" => output.push_str("false"),
            _ => return None,
        },
        YamlScalarSemantic::Null => output.push_str("null"),
        YamlScalarSemantic::Integer | YamlScalarSemantic::Float | YamlScalarSemantic::Unknown => {
            output.push_str(raw);
        }
        YamlScalarSemantic::String => unreachable!("string scalars return before match"),
    }
    emit_inline_comment(output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
    Some(())
}

fn render_unsafe_plain_string_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
) -> Option<String> {
    if scalar.style != YamlScalarStyle::Plain
        || scalar.semantic != YamlScalarSemantic::String
        || scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.tag.is_some()
    {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let content = source.slice(metadata.content).trim();
    if content.contains(['\n', '\r']) || block_plain_string_safe(content) {
        return None;
    }
    let quoted = quote_yaml_single_for_flow(content)?;
    let mut output = render_explicit_core_scalar(source, scalar, &quoted);
    emit_inline_comment(&mut output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
    Some(output)
}

fn simplify_quoted_string_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<String> {
    if !matches!(
        scalar.style,
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted
    ) || scalar.semantic != YamlScalarSemantic::String
        || scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    let raw = source.slice(scalar.value).trim();
    if raw.contains(['\n', '\r']) {
        return None;
    }
    if let Some(decoded) = simple_quoted_scalar_inner(raw) {
        return plain_string_safe(decoded, flow_context).then(|| decoded.to_owned());
    }
    let decoded = decode_quoted_scalar(raw)?;
    plain_string_safe(&decoded, flow_context).then_some(decoded)
}

fn render_quoted_newline_literal_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    body_indent: Option<usize>,
) -> Option<String> {
    if !matches!(
        scalar.style,
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted
    ) || scalar.tag.is_some_and(|tag| source.slice(tag) != "!!str")
        || scalar.anchor.is_some()
        || scalar.trailing_comment.is_some()
        || scalar.header.is_some()
    {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let raw = source.slice(metadata.content).trim();
    if raw.contains(['\n', '\r']) {
        return None;
    }
    let decoded = decode_quoted_scalar(raw)?;
    if !decoded.contains('\n') || decoded.contains('\r') {
        return None;
    }
    if decoded
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
    {
        return None;
    }

    let newline = line_ending_or_default(source, node.span, options);
    let final_newline = !line_ending_for_span(source, node.span).is_empty();
    let indent = body_indent.unwrap_or_else(|| {
        let line_index = source.line_at_byte(node.span.start());
        indentation(source.line_text(line_index)) + options.indent_width
    });
    let indent = " ".repeat(indent);

    let mut out = String::new();
    let trailing_newlines = decoded
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\n')
        .count();
    let marker = match trailing_newlines {
        0 => "|-",
        1 => "|",
        _ => "|+",
    };
    out.push_str(&render_explicit_core_scalar(source, scalar, marker));
    out.push_str(newline);

    if trailing_newlines == 0 {
        let lines = decoded.split('\n').collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            out.push_str(&indent);
            out.push_str(line);
            if index + 1 < lines.len() || final_newline {
                out.push_str(newline);
            }
        }
    } else {
        let body = &decoded[..decoded.len() - trailing_newlines];
        for line in body.split('\n') {
            out.push_str(&indent);
            out.push_str(line);
            out.push_str(newline);
        }
        for _ in 1..trailing_newlines {
            out.push_str(newline);
        }
    }
    Some(out)
}

fn decode_quoted_scalar(raw: &str) -> Option<String> {
    if raw.starts_with('"') {
        decode_double_quoted_scalar(raw)
    } else if raw.starts_with('\'') {
        decode_single_quoted_scalar(raw)
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
        let escaped = chars.next()?;
        match escaped {
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

fn render_folded_prose_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    body_indent: Option<usize>,
) -> Option<String> {
    if scalar.tag.is_some_and(|tag| source.slice(tag) != "!!str")
        || scalar.anchor.is_some()
        || scalar.trailing_comment.is_some()
        || scalar.header.is_some()
    {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let raw = source.slice(metadata.content).trim();
    let prose = match scalar.style {
        YamlScalarStyle::Plain => {
            if scalar.semantic != YamlScalarSemantic::String {
                return None;
            }
            if raw.contains(['\n', '\r', '\t']) {
                return None;
            }
            if raw.chars().count() <= options.prose_width || !raw.contains(char::is_whitespace) {
                return None;
            }
            let normalized = normalize_yaml_prose(raw);
            if normalized != raw {
                return None;
            }
            normalized
        }
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            if let Some(decoded) = simple_quoted_scalar_inner(raw) {
                if decoded.chars().count() <= options.prose_width {
                    return None;
                }
                if decoded != decoded.trim()
                    || decoded.contains(['\n', '\r', '\t'])
                    || decoded
                        .chars()
                        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
                {
                    return None;
                }
                let normalized = normalize_yaml_prose(decoded);
                if normalized != decoded {
                    return None;
                }
                normalized
            } else {
                let decoded = decode_quoted_scalar(raw)?;
                if decoded.chars().count() <= options.prose_width {
                    return None;
                }
                if decoded != decoded.trim()
                    || decoded.contains(['\n', '\r', '\t'])
                    || decoded
                        .chars()
                        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
                {
                    return None;
                }
                let normalized = normalize_yaml_prose(&decoded);
                if normalized != decoded {
                    return None;
                }
                decoded
            }
        }
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => return None,
    };
    if prose.is_empty() || prose.chars().count() <= options.prose_width {
        return None;
    }
    let newline = line_ending_or_default(source, node.span, options);
    let final_newline = !line_ending_for_span(source, node.span).is_empty();
    let indent = body_indent.unwrap_or_else(|| {
        let line_index = source.line_at_byte(node.span.start());
        indentation(source.line_text(line_index)) + options.indent_width
    });
    let indent = " ".repeat(indent);
    let lines = wrap_yaml_prose(&prose, options.prose_width.max(1));
    let mut out = String::new();
    out.push_str(&render_explicit_core_scalar(source, scalar, ">-"));
    out.push_str(newline);
    for (index, line) in lines.iter().enumerate() {
        out.push_str(&indent);
        out.push_str(line);
        if index + 1 < lines.len() || final_newline {
            out.push_str(newline);
        }
    }
    Some(out)
}

fn render_rewrapped_folded_block_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
) -> Option<String> {
    if scalar.style != YamlScalarStyle::FoldedBlock
        || scalar.nested.is_some()
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
        || scalar.trailing_comment.is_some()
    {
        return None;
    }
    let header = source.slice(scalar.header?);
    if !simple_folded_block_header(header) {
        return None;
    }
    let body = scalar.body?;
    let body_text = source.slice(body);
    if body_text.contains('\t') {
        return None;
    }
    let prose = folded_block_body_prose(body_text)?;
    if !options.markdown_canonical && prose.text.chars().count() <= options.prose_width {
        return None;
    }
    let newline = line_ending_or_default(source, node.span, options);
    let lines = wrap_yaml_prose(&prose.text, options.prose_width.max(1));
    let indent = " ".repeat(prose.indent);
    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        out.push_str(&indent);
        out.push_str(line);
        if index + 1 < lines.len() || prose.final_newline {
            out.push_str(newline);
        }
    }
    for _ in 0..prose.trailing_blank_lines {
        out.push_str(newline);
    }
    Some(out)
}

fn simple_folded_block_header(header: &str) -> bool {
    let (body, _) = strip_newline(header);
    let comment_start = find_trailing_comment(body, 0).unwrap_or(body.len());
    let Some(marker) = body[..comment_start].rfind('>') else {
        return false;
    };
    body[marker + 1..comment_start]
        .chars()
        .all(|ch| matches!(ch, '-' | ' ' | '\t'))
}

struct FoldedBlockProse {
    text: String,
    indent: usize,
    final_newline: bool,
    trailing_blank_lines: usize,
}

fn folded_block_body_prose(body: &str) -> Option<FoldedBlockProse> {
    let mut indent = None::<usize>;
    let mut parts = Vec::new();
    let mut pending_blank_lines = 0usize;
    let mut final_newline = false;
    for line in body.split_inclusive('\n') {
        let (line_body, newline) = strip_newline(line);
        final_newline = !newline.is_empty();
        if line_body.trim().is_empty() {
            if parts.is_empty() {
                return None;
            }
            pending_blank_lines += 1;
            continue;
        }
        if pending_blank_lines > 0 {
            return None;
        }
        let line_indent = line_body.bytes().take_while(|byte| *byte == b' ').count();
        let indent = *indent.get_or_insert(line_indent);
        if line_indent < indent {
            return None;
        }
        parts.push(line_body[indent..].trim());
    }
    let indent = indent?;
    let prose = normalize_yaml_prose(&parts.join(" "));
    (!prose.is_empty()).then_some(FoldedBlockProse {
        text: prose,
        indent,
        final_newline,
        trailing_blank_lines: pending_blank_lines,
    })
}

fn normalize_yaml_prose(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut pending_space = false;
    for ch in source.chars() {
        if ch.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(ch);
            pending_space = false;
        }
    }
    out
}

fn wrap_yaml_prose(source: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in source.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_owned();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn line_ending_or_default(
    source: &SourceBuffer,
    span: impl Into<Span>,
    options: FormatOptions,
) -> &'static str {
    let newline = line_ending_for_span(source, span);
    if newline.is_empty() {
        options.default_line_ending
    } else {
        newline
    }
}

fn render_inline_markdown_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    state: &crate::core::directives::DirectiveState,
    options: FormatOptions,
    body_indent: Option<usize>,
) -> Option<String> {
    let metadata = scalar_metadata(source, scalar.value);
    let content = inline_markdown_scalar_content(source, scalar, metadata.content)?;
    let prefix = source
        .slice(Span::new(scalar.value.start(), metadata.content.start))
        .trim();
    let newline = line_ending_or_default(source, node.span, options);

    let mut out = String::new();
    if content.is_empty() {
        if !prefix.is_empty() {
            out.push_str(prefix);
            out.push(' ');
        }
        out.push_str("\"\"");
        emit_inline_comment(&mut out, source, scalar.trailing_comment);
        out.push_str(newline);
        return Some(out);
    }

    if !prefix.is_empty() {
        out.push_str(prefix);
        out.push(' ');
    }
    out.push('|');
    emit_inline_comment(&mut out, source, scalar.trailing_comment);
    out.push_str(newline);

    let mut formatted =
        crate::core::wrap::format_markdown_fragment(&content, state.markdown_options(options));
    if !formatted.ends_with('\n') && !formatted.ends_with('\r') {
        formatted.push_str(newline);
    }
    let indent = body_indent.unwrap_or_else(|| {
        let line_index = source.line_at_byte(node.span.start());
        indentation(source.line_text(line_index)) + options.indent_width
    });
    emit_indented_block(&mut out, &formatted, indent, newline);
    Some(out)
}

fn inline_markdown_scalar_is_renderable(source: &SourceBuffer, scalar: &YamlScalar<'_>) -> bool {
    let metadata = scalar_metadata(source, scalar.value);
    inline_markdown_scalar_content(source, scalar, metadata.content).is_some()
}

fn inline_markdown_scalar_content<'a>(
    source: &'a SourceBuffer,
    scalar: &YamlScalar<'_>,
    content: Span,
) -> Option<Cow<'a, str>> {
    let raw = source.slice(content).trim();
    match scalar.style {
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            if let Some(inner) = simple_quoted_scalar_inner(raw) {
                Some(Cow::Borrowed(inner))
            } else {
                decode_quoted_scalar(raw).map(Cow::Owned)
            }
        }
        YamlScalarStyle::Plain => Some(Cow::Borrowed(raw)),
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => None,
    }
}

fn emit_indented_block(out: &mut String, body: &str, indent: usize, default_newline: &str) {
    let indent = " ".repeat(indent);
    for line in body.split_inclusive('\n') {
        let (line_body, newline) = strip_newline(line);
        out.push_str(&indent);
        out.push_str(line_body);
        out.push_str(if newline.is_empty() {
            default_newline
        } else {
            newline
        });
    }
}

fn reindent_yaml_block_scalar_body(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    body: &str,
    options: FormatOptions,
) -> String {
    let Some(source_body) = scalar.body else {
        return body.to_owned();
    };
    let strip_indent = explicit_block_scalar_body_indent(source, scalar, node)
        .or_else(|| block_scalar_body_indent(source.slice(source_body)))
        .unwrap_or_else(|| {
            let line_index = source.line_at_byte(node.span.start());
            indentation(source.line_text(line_index)) + options.indent_width
        });
    let emit_indent = strip_indent;
    reindent_block_lines(body, strip_indent, emit_indent)
}

fn explicit_block_scalar_body_indent(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
) -> Option<usize> {
    let indent = scalar.block_header?.indent? as usize;
    let line_index = source.line_at_byte(node.span.start());
    Some(indentation(source.line_text(line_index)) + indent)
}

fn block_scalar_body_indent(body: &str) -> Option<usize> {
    body.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.bytes().take_while(|byte| *byte == b' ').count())
        .min()
}

fn reindent_block_lines(body: &str, strip_indent: usize, emit_indent: usize) -> String {
    let prefix = " ".repeat(emit_indent);
    let strip = " ".repeat(strip_indent);
    let mut out = String::with_capacity(body.len() + prefix.len());
    for line in body.split_inclusive('\n') {
        let (line_body, newline) = strip_newline(line);
        if line_body.trim().is_empty() {
            out.push_str(line_body);
            out.push_str(newline);
            continue;
        }
        let content = line_body.strip_prefix(&strip).unwrap_or(line_body);
        out.push_str(&prefix);
        out.push_str(content);
        out.push_str(newline);
    }
    if !body.is_empty() && !body.ends_with('\n') && !body.ends_with('\r') {
        let line = body.rsplit(['\n', '\r']).next().unwrap_or(body);
        if line.len() == body.len() {
            let content = line.strip_prefix(&strip).unwrap_or(line);
            out.clear();
            out.push_str(&prefix);
            out.push_str(content);
        }
    }
    out
}

fn emit_yaml_alias(
    out: &mut String,
    source: &SourceBuffer,
    alias: &YamlAlias,
    node: &YamlAstNode<'_>,
) {
    out.push_str(source.slice(alias.value).trim());
    emit_inline_comment(out, source, alias.trailing_comment);
    out.push_str(line_ending_for_span(source, node.span));
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_flow_collection(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    node: &YamlAstNode<'_>,
    indent: usize,
    options: FormatOptions,
) {
    match &node.emit {
        YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection) => {
            emit_yaml_flow_collection_block_into(
                out,
                source,
                document,
                ast,
                id,
                indent,
                line_ending_or_default(source, node.span, options),
                options,
                None,
            )
            .expect("planned block YAML collection should render during emission");
            return;
        }
        YamlEmitPlan::Rendered(YamlRenderedKind::FlowCollection) => {
            emit_yaml_inline_node_into(out, source, document, ast, id)
                .expect("planned inline YAML collection should render during emission");
        }
        _ => match &node.kind {
            YamlAstKind::FlowSequence(sequence) => {
                out.push_str(source.slice(sequence.value).trim());
            }
            YamlAstKind::FlowMapping(mapping) => {
                out.push_str(source.slice(mapping.value).trim());
            }
            _ => out.push_str(source.slice(node.span).trim()),
        },
    }
    emit_inline_comment(out, source, flow_trailing_comment(&node.kind));
    out.push_str(line_ending_for_span(source, node.span));
}

fn planned_yaml_inline_width_or_source(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> usize {
    let node = ast.node(id);
    if matches!(
        node.emit,
        YamlEmitPlan::Rendered(YamlRenderedKind::FlowCollection)
    ) && let Some(width) = render_yaml_inline_node_width(source, document, ast, id)
    {
        return width;
    }
    match &node.kind {
        YamlAstKind::FlowSequence(sequence) => source.slice(sequence.value).trim().chars().count(),
        YamlAstKind::FlowMapping(mapping) => source.slice(mapping.value).trim().chars().count(),
        _ => source.slice(node.span).trim().chars().count(),
    }
}

fn emit_yaml_inline_node_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<()> {
    emit_yaml_inline_node_into_with_context(out, source, document, ast, id, false)
}

fn emit_yaml_inline_node_into_for_flow(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<()> {
    emit_yaml_inline_node_into_with_context(out, source, document, ast, id, true)
}

fn emit_yaml_inline_node_into_with_context(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    flow_context: bool,
) -> Option<()> {
    let start_len = out.len();
    if emit_yaml_inline_node_into_inner(out, source, document, ast, id, flow_context).is_some() {
        Some(())
    } else {
        out.truncate(start_len);
        None
    }
}

fn emit_yaml_inline_node_into_inner(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    flow_context: bool,
) -> Option<()> {
    let node = ast.node(id);
    if matches!(
        &node.kind,
        YamlAstKind::Scalar(scalar) if scalar.value.is_empty() && scalar_has_properties(scalar)
    ) {
        return None;
    }
    if yaml_node_should_preserve(source, document, node) {
        match &node.kind {
            YamlAstKind::Scalar(scalar) => out.push_str(source.slice(scalar.value).trim()),
            YamlAstKind::Alias(alias) => out.push_str(source.slice(alias.value).trim()),
            YamlAstKind::FlowSequence(sequence) => {
                out.push_str(source.slice(sequence.value).trim());
            }
            YamlAstKind::FlowMapping(mapping) => out.push_str(source.slice(mapping.value).trim()),
            _ => out.push_str(source.slice(node.span).trim()),
        }
        return Some(());
    }

    match &node.kind {
        YamlAstKind::Scalar(scalar) => {
            if scalar.header.is_some()
                || scalar.value.is_empty()
                || scalar.trailing_comment.is_some()
            {
                return None;
            }
            if flow_context
                && scalar.style == YamlScalarStyle::Plain
                && source.slice(scalar.value).contains(['\n', '\r'])
            {
                return None;
            }
            emit_yaml_scalar_inline_into(out, source, scalar, flow_context)?;
        }
        YamlAstKind::Alias(alias) => {
            if alias.trailing_comment.is_some() {
                return None;
            }
            out.push_str(source.slice(alias.value).trim());
        }
        YamlAstKind::FlowSequence(sequence) => {
            if sequence.has_inner_trivia {
                return None;
            }
            if flow_context && sequence.anchor.is_some() {
                return None;
            }
            emit_collection_property_prefix_into(
                out,
                source,
                YamlCollectionKind::Sequence,
                sequence.tag,
                sequence.anchor,
            )?;
            out.push('[');
            for (index, entry) in sequence.entries.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                emit_yaml_inline_node_into_for_flow(out, source, document, ast, *entry)?;
            }
            out.push(']');
        }
        YamlAstKind::FlowMapping(mapping) => {
            if !mapping.braced {
                out.push_str(source.slice(mapping.value).trim());
                return Some(());
            }
            if mapping.pairs.iter().any(|pair| pair.explicit) {
                return None;
            }
            if mapping.has_inner_trivia {
                return None;
            }
            if flow_context && mapping.anchor.is_some() {
                return None;
            }
            emit_collection_property_prefix_into(
                out,
                source,
                YamlCollectionKind::Mapping,
                mapping.tag,
                mapping.anchor,
            )?;
            out.push('{');
            for (index, pair) in mapping.pairs.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                emit_yaml_inline_node_into_for_flow(out, source, document, ast, pair.key)?;
                out.push(':');
                if let Some(value) = pair.value {
                    out.push(' ');
                    emit_yaml_inline_node_into_for_flow(out, source, document, ast, value)?;
                }
            }
            out.push('}');
        }
        _ => return None,
    }
    Some(())
}

fn render_yaml_inline_node_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<usize> {
    render_yaml_inline_node_width_with_context(source, document, ast, id, false)
}

fn render_yaml_inline_node_width_for_flow(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<usize> {
    render_yaml_inline_node_width_with_context(source, document, ast, id, true)
}

fn render_yaml_inline_node_width_with_context(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    flow_context: bool,
) -> Option<usize> {
    let node = ast.node(id);
    let cached_width = if flow_context {
        node.flow_inline_width()
    } else {
        node.inline_width()
    };
    if let Some(width) = cached_width {
        return width;
    }
    let width = render_yaml_inline_node_width_with_context_uncached(
        source,
        document,
        ast,
        id,
        flow_context,
    );
    if flow_context {
        node.set_flow_inline_width(width);
    } else {
        node.set_inline_width(width);
    }
    width
}

fn render_yaml_inline_node_width_with_context_uncached(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    flow_context: bool,
) -> Option<usize> {
    let node = ast.node(id);
    if matches!(
        &node.kind,
        YamlAstKind::Scalar(scalar) if scalar.value.is_empty() && scalar_has_properties(scalar)
    ) {
        return None;
    }
    if yaml_node_should_preserve(source, document, node) {
        return Some(match &node.kind {
            YamlAstKind::Scalar(scalar) => source.slice(scalar.value).trim().chars().count(),
            YamlAstKind::Alias(alias) => source.slice(alias.value).trim().chars().count(),
            YamlAstKind::FlowSequence(sequence) => {
                source.slice(sequence.value).trim().chars().count()
            }
            YamlAstKind::FlowMapping(mapping) => source.slice(mapping.value).trim().chars().count(),
            _ => source.slice(node.span).trim().chars().count(),
        });
    }

    match &node.kind {
        YamlAstKind::Scalar(scalar) => {
            if scalar.header.is_some()
                || scalar.value.is_empty()
                || scalar.trailing_comment.is_some()
            {
                return None;
            }
            if flow_context
                && scalar.style == YamlScalarStyle::Plain
                && source.slice(scalar.value).contains(['\n', '\r'])
            {
                return None;
            }
            render_yaml_scalar_inline_width(source, scalar, flow_context)
        }
        YamlAstKind::Alias(alias) => {
            if alias.trailing_comment.is_some() {
                return None;
            }
            Some(source.slice(alias.value).trim().chars().count())
        }
        YamlAstKind::FlowSequence(sequence) => {
            if sequence.has_inner_trivia {
                return None;
            }
            let property_prefix_width = collection_property_prefix_width(
                source,
                YamlCollectionKind::Sequence,
                sequence.tag,
                sequence.anchor,
            )?;
            if flow_context && property_prefix_width > 0 {
                return None;
            }
            let mut width = property_prefix_width + 2;
            for (index, entry) in sequence.entries.iter().enumerate() {
                if index > 0 {
                    width += 2;
                }
                width += render_yaml_inline_node_width_for_flow(source, document, ast, *entry)?;
            }
            Some(width)
        }
        YamlAstKind::FlowMapping(mapping) => {
            if !mapping.braced {
                return Some(source.slice(mapping.value).trim().chars().count());
            }
            if mapping.pairs.iter().any(|pair| pair.explicit) {
                return None;
            }
            if mapping.has_inner_trivia {
                return None;
            }
            let property_prefix_width = collection_property_prefix_width(
                source,
                YamlCollectionKind::Mapping,
                mapping.tag,
                mapping.anchor,
            )?;
            if flow_context && property_prefix_width > 0 {
                return None;
            }
            let mut width = property_prefix_width + 2;
            for (index, pair) in mapping.pairs.iter().enumerate() {
                if index > 0 {
                    width += 2;
                }
                width += render_yaml_inline_node_width_for_flow(source, document, ast, pair.key)?;
                width += if let Some(value) = pair.value {
                    2 + render_yaml_inline_node_width_for_flow(source, document, ast, value)?
                } else {
                    1
                };
            }
            Some(width)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn yaml_flow_collection_block_renderable(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    indent: usize,
    options: FormatOptions,
) -> Option<()> {
    let node = ast.node(id);
    if yaml_node_should_preserve(source, document, node) {
        return None;
    }
    if yaml_flow_collection_source_contains_tab(source, node) {
        return None;
    }
    match &node.kind {
        YamlAstKind::FlowSequence(sequence) => {
            if sequence.has_inner_trivia
                || sequence.entries.is_empty()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Sequence,
                    sequence.tag,
                )
                || sequence.anchor.is_some()
                || sequence.trailing_comment.is_some()
            {
                return None;
            }
            for entry in &sequence.entries {
                let value_width =
                    render_yaml_inline_node_width_for_flow(source, document, ast, *entry)?;
                if yaml_node_is_flow_collection(ast.node(*entry))
                    && indent + 2 + value_width > options.line_width
                {
                    yaml_flow_collection_block_renderable(
                        source,
                        document,
                        ast,
                        *entry,
                        indent + options.indent_width,
                        options,
                    )?;
                }
            }
        }
        YamlAstKind::FlowMapping(mapping) => {
            if !mapping.braced
                || mapping.has_inner_trivia
                || mapping.pairs.is_empty()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Mapping,
                    mapping.tag,
                )
                || mapping.anchor.is_some()
                || mapping.trailing_comment.is_some()
                || mapping.pairs.iter().any(|pair| pair.explicit)
            {
                return None;
            }
            for pair in &mapping.pairs {
                let key_width =
                    render_yaml_inline_node_width_for_flow(source, document, ast, pair.key)?;
                if let Some(value_id) = pair.value {
                    let value_width =
                        render_yaml_inline_node_width_for_flow(source, document, ast, value_id)?;
                    if yaml_node_is_flow_collection(ast.node(value_id))
                        && indent + key_width + 2 + value_width > options.line_width
                    {
                        yaml_flow_collection_block_renderable(
                            source,
                            document,
                            ast,
                            value_id,
                            indent + options.indent_width,
                            options,
                        )?;
                    }
                }
            }
        }
        _ => return None,
    }
    Some(())
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_flow_collection_block_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    indent: usize,
    newline: &str,
    options: FormatOptions,
    first_line_prefix: Option<&str>,
) -> Option<()> {
    let start_len = out.len();
    let mut first_line_prefix = first_line_prefix;
    if emit_yaml_flow_collection_block_lines(
        out,
        source,
        document,
        ast,
        id,
        indent,
        newline,
        options,
        &mut first_line_prefix,
    )
    .is_none()
    {
        out.truncate(start_len);
        return None;
    }
    (out.len() > start_len).then_some(())
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_flow_collection_block_lines(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
    indent: usize,
    newline: &str,
    options: FormatOptions,
    first_line_prefix: &mut Option<&str>,
) -> Option<()> {
    let node = ast.node(id);
    if yaml_node_should_preserve(source, document, node) {
        return None;
    }
    if yaml_flow_collection_source_contains_tab(source, node) {
        return None;
    }
    match &node.kind {
        YamlAstKind::FlowSequence(sequence) => {
            if sequence.has_inner_trivia
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Sequence,
                    sequence.tag,
                )
                || sequence.anchor.is_some()
                || sequence.trailing_comment.is_some()
            {
                return None;
            }
            for entry in &sequence.entries {
                let value_width =
                    render_yaml_inline_node_width_for_flow(source, document, ast, *entry)?;
                emit_yaml_flow_block_line_prefix(out, indent, first_line_prefix);
                if yaml_node_is_flow_collection(ast.node(*entry))
                    && indent + 2 + value_width > options.line_width
                {
                    out.push('-');
                    out.push_str(newline);
                    emit_yaml_flow_collection_block_lines(
                        out,
                        source,
                        document,
                        ast,
                        *entry,
                        indent + options.indent_width,
                        newline,
                        options,
                        first_line_prefix,
                    )?;
                } else {
                    out.push_str("- ");
                    emit_yaml_inline_node_into_for_flow(out, source, document, ast, *entry)?;
                    out.push_str(newline);
                }
            }
        }
        YamlAstKind::FlowMapping(mapping) => {
            if !mapping.braced
                || mapping.has_inner_trivia
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Mapping,
                    mapping.tag,
                )
                || mapping.anchor.is_some()
                || mapping.trailing_comment.is_some()
                || mapping.pairs.iter().any(|pair| pair.explicit)
            {
                return None;
            }
            for pair in &mapping.pairs {
                let key_width =
                    render_yaml_inline_node_width_for_flow(source, document, ast, pair.key)?;
                let value_action = if let Some(value_id) = pair.value {
                    let value_width =
                        render_yaml_inline_node_width_for_flow(source, document, ast, value_id)?;
                    Some((
                        value_id,
                        yaml_node_is_flow_collection(ast.node(value_id))
                            && indent + key_width + 2 + value_width > options.line_width,
                    ))
                } else {
                    None
                };
                emit_yaml_flow_block_line_prefix(out, indent, first_line_prefix);
                emit_yaml_inline_node_into_for_flow(out, source, document, ast, pair.key)?;
                out.push(':');
                if let Some((value_id, should_expand)) = value_action {
                    if should_expand {
                        out.push_str(newline);
                        emit_yaml_flow_collection_block_lines(
                            out,
                            source,
                            document,
                            ast,
                            value_id,
                            indent + options.indent_width,
                            newline,
                            options,
                            first_line_prefix,
                        )?;
                    } else {
                        out.push(' ');
                        emit_yaml_inline_node_into_for_flow(out, source, document, ast, value_id)?;
                        out.push_str(newline);
                    }
                } else {
                    out.push_str(newline);
                }
            }
        }
        _ => return None,
    }
    Some(())
}

fn emit_yaml_flow_block_line_prefix(
    out: &mut String,
    indent: usize,
    first_line_prefix: &mut Option<&str>,
) {
    if let Some(prefix) = first_line_prefix.take() {
        out.push_str(prefix);
    } else {
        emit_spaces(out, indent);
    }
}

fn emit_spaces(out: &mut String, count: usize) {
    const SPACES: &str = "                                ";
    let mut remaining = count;
    while remaining >= SPACES.len() {
        out.push_str(SPACES);
        remaining -= SPACES.len();
    }
    out.push_str(&SPACES[..remaining]);
}

fn yaml_node_is_flow_collection(node: &YamlAstNode<'_>) -> bool {
    matches!(
        node.kind,
        YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_)
    )
}

fn yaml_flow_collection_should_expand(
    source: &SourceBuffer,
    node: &YamlAstNode<'_>,
    inline_width: usize,
    options: FormatOptions,
) -> bool {
    inline_width > options.line_width
        || yaml_flow_collection_has_multiline_intent(source, node, options)
}

fn yaml_flow_collection_has_multiline_intent(
    source: &SourceBuffer,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
) -> bool {
    !options.yaml_compact && yaml_flow_collection_has_source_newline(source, node)
}

fn yaml_flow_collection_has_source_newline(source: &SourceBuffer, node: &YamlAstNode<'_>) -> bool {
    let Some(span) = yaml_flow_collection_source_span(node) else {
        return false;
    };
    !span.is_empty() && source.line_at_byte(span.start()) != source.line_at_byte(span.end())
}

fn yaml_flow_collection_source_contains_tab(source: &SourceBuffer, node: &YamlAstNode<'_>) -> bool {
    let Some(span) = yaml_flow_collection_source_span(node) else {
        return false;
    };
    source.slice(span).as_bytes().contains(&b'\t')
}

fn yaml_flow_collection_source_span<'src>(node: &YamlAstNode<'src>) -> Option<SourceSpan<'src>> {
    match &node.kind {
        YamlAstKind::FlowSequence(sequence) => Some(sequence.value),
        YamlAstKind::FlowMapping(mapping) => Some(mapping.value),
        _ => None,
    }
}

fn emit_yaml_scalar_inline_into(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<()> {
    let raw = source.slice(scalar.value).trim();
    if emit_plain_scalar_inline_into(out, raw, scalar, flow_context).is_some() {
        return Some(());
    }
    if let Some(normalized) = normalize_core_scalar(source, scalar, flow_context) {
        out.push_str(&normalized);
        return Some(());
    }
    if scalar.tag.is_some() || scalar.anchor.is_some() {
        return None;
    }
    if emit_simple_quoted_string_scalar_into(out, raw, scalar, flow_context).is_some() {
        return Some(());
    }
    if let Some(simplified) = simplify_quoted_string_scalar(source, scalar, flow_context) {
        out.push_str(&simplified);
        return Some(());
    }
    if !flow_context || scalar.style != YamlScalarStyle::Plain || flow_plain_scalar_safe(raw) {
        out.push_str(raw);
        return Some(());
    }
    let quoted = quote_yaml_single_for_flow(raw)?;
    out.push_str(&quoted);
    Some(())
}

fn emit_plain_scalar_inline_into(
    out: &mut String,
    raw: &str,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<()> {
    if scalar.header.is_some()
        || scalar.trailing_comment.is_some()
        || scalar.style != YamlScalarStyle::Plain
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    match scalar.semantic {
        YamlScalarSemantic::Boolean => {
            let normalized = match raw {
                "true" | "True" | "TRUE" => "true",
                "false" | "False" | "FALSE" => "false",
                _ => return None,
            };
            out.push_str(normalized);
            Some(())
        }
        YamlScalarSemantic::Null => {
            out.push_str("null");
            Some(())
        }
        _ if !flow_context || flow_plain_scalar_safe(raw) => {
            out.push_str(raw);
            Some(())
        }
        _ => {
            let quoted = quote_yaml_single_for_flow(raw)?;
            out.push_str(&quoted);
            Some(())
        }
    }
}

fn emit_simple_quoted_string_scalar_into(
    out: &mut String,
    raw: &str,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<()> {
    if !matches!(
        scalar.style,
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted
    ) || scalar.semantic != YamlScalarSemantic::String
        || scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    let decoded = simple_quoted_scalar_inner(raw)?;
    if !plain_string_safe(decoded, flow_context) {
        return None;
    }
    out.push_str(decoded);
    Some(())
}

fn render_yaml_scalar_inline_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<usize> {
    let raw = source.slice(scalar.value).trim();
    if let Some(width) = normalize_core_scalar_width(source, scalar, flow_context) {
        return Some(width);
    }
    if scalar.tag.is_some() || scalar.anchor.is_some() {
        return None;
    }
    if let Some(width) = simplify_quoted_string_scalar_width(source, scalar, flow_context) {
        return Some(width);
    }
    if !flow_context || scalar.style != YamlScalarStyle::Plain || flow_plain_scalar_safe(raw) {
        return Some(raw.chars().count());
    }
    Some(quote_yaml_single_for_flow_width(raw))
}

fn normalize_core_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<String> {
    if scalar.header.is_some() || scalar.trailing_comment.is_some() {
        return None;
    }
    if scalar.style == YamlScalarStyle::Plain
        && scalar.semantic == YamlScalarSemantic::String
        && scalar.tag.is_none()
        && scalar.anchor.is_none()
    {
        return None;
    }
    if scalar.style != YamlScalarStyle::Plain && scalar.tag.is_none() {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let content = source.slice(metadata.content).trim();
    let tag = scalar.tag.map(|tag| source.slice(tag));
    let decoded = scalar_value_for_core_tag(content)?;
    match tag {
        Some("!!bool") => normalize_yaml_bool(&decoded)
            .map(|value| render_explicit_core_scalar(source, scalar, &value)),
        Some("!!null") if scalar_semantic(&decoded) == YamlScalarSemantic::Null => {
            Some(render_explicit_core_scalar(source, scalar, "null"))
        }
        Some("!!int") if scalar_semantic(&decoded) == YamlScalarSemantic::Integer => {
            Some(render_explicit_core_scalar(source, scalar, &decoded))
        }
        Some("!!float") if scalar_semantic(&decoded) == YamlScalarSemantic::Float => {
            Some(render_explicit_core_scalar(source, scalar, &decoded))
        }
        Some("!!str") => {
            let value = if plain_string_safe(&decoded, flow_context) {
                decoded
            } else {
                quote_yaml_single_for_flow(&decoded)?
            };
            Some(render_explicit_core_scalar(source, scalar, &value))
        }
        Some(_) => None,
        None if scalar.anchor.is_some() => None,
        None => {
            if scalar.style == YamlScalarStyle::Plain {
                normalize_implicit_core_scalar(content, scalar)
            } else {
                None
            }
        }
    }
}

fn normalize_core_scalar_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<usize> {
    if scalar.header.is_some() || scalar.trailing_comment.is_some() {
        return None;
    }
    if scalar.style == YamlScalarStyle::Plain
        && scalar.semantic == YamlScalarSemantic::String
        && scalar.tag.is_none()
        && scalar.anchor.is_none()
    {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let content = source.slice(metadata.content).trim();
    let tag = scalar.tag.map(|tag| source.slice(tag));
    if scalar.style != YamlScalarStyle::Plain {
        if let Some(decoded) = simple_quoted_scalar_inner(content) {
            return normalize_explicit_core_scalar_width(
                source,
                scalar,
                metadata,
                tag,
                decoded,
                flow_context,
            );
        }
        tag?;
        if tag == Some("!!str")
            && let Some(value_width) = decoded_quoted_explicit_string_width(content, flow_context)
        {
            return Some(explicit_core_scalar_width(
                source,
                scalar,
                metadata,
                value_width,
            ));
        }
        if let Some(value_width) = decoded_quoted_explicit_core_width(content, tag) {
            return Some(explicit_core_scalar_width(
                source,
                scalar,
                metadata,
                value_width,
            ));
        }
        return normalize_core_scalar(source, scalar, flow_context)
            .map(|value| value.chars().count());
    }
    match tag {
        Some("!!bool") => normalized_yaml_bool_width(content)
            .map(|width| explicit_core_scalar_width(source, scalar, metadata, width)),
        Some("!!null") if scalar_semantic(content) == YamlScalarSemantic::Null => Some(
            explicit_core_scalar_width(source, scalar, metadata, "null".len()),
        ),
        Some("!!int") if scalar_semantic(content) == YamlScalarSemantic::Integer => Some(
            explicit_core_scalar_width(source, scalar, metadata, content.chars().count()),
        ),
        Some("!!float") if scalar_semantic(content) == YamlScalarSemantic::Float => Some(
            explicit_core_scalar_width(source, scalar, metadata, content.chars().count()),
        ),
        Some("!!str") => {
            let value_width = if plain_string_safe(content, flow_context) {
                content.chars().count()
            } else {
                quote_yaml_single_for_flow_width(content)
            };
            Some(explicit_core_scalar_width(
                source,
                scalar,
                metadata,
                value_width,
            ))
        }
        Some(_) => None,
        None if scalar.anchor.is_some() => None,
        None => normalize_implicit_core_scalar_width(content, scalar),
    }
}

fn normalize_explicit_core_scalar_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    metadata: ScalarMetadata,
    tag: Option<&str>,
    decoded: &str,
    flow_context: bool,
) -> Option<usize> {
    let value_width = match tag {
        Some("!!bool") => normalized_yaml_bool_width(decoded)?,
        Some("!!null") if scalar_semantic(decoded) == YamlScalarSemantic::Null => "null".len(),
        Some("!!int") if scalar_semantic(decoded) == YamlScalarSemantic::Integer => {
            decoded.chars().count()
        }
        Some("!!float") if scalar_semantic(decoded) == YamlScalarSemantic::Float => {
            decoded.chars().count()
        }
        Some("!!str") if plain_string_safe(decoded, flow_context) => decoded.chars().count(),
        Some("!!str") => quote_yaml_single_for_flow_width(decoded),
        Some(_) | None => return None,
    };
    Some(explicit_core_scalar_width(
        source,
        scalar,
        metadata,
        value_width,
    ))
}

fn simple_quoted_scalar_inner(raw: &str) -> Option<&str> {
    if let Some(inner) = raw
        .strip_prefix('\'')
        .and_then(|raw| raw.strip_suffix('\''))
    {
        return (!inner.contains('\'')).then_some(inner);
    }
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains(['\\', '\n', '\r'])).then_some(inner)
}

fn normalize_implicit_core_scalar(raw: &str, scalar: &YamlScalar<'_>) -> Option<String> {
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => Some("true".to_owned()),
            "false" | "False" | "FALSE" => Some("false".to_owned()),
            _ => None,
        },
        YamlScalarSemantic::Null => Some("null".to_owned()),
        _ => None,
    }
}

fn normalize_implicit_core_scalar_width(raw: &str, scalar: &YamlScalar<'_>) -> Option<usize> {
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => Some("true".len()),
            "false" | "False" | "FALSE" => Some("false".len()),
            _ => None,
        },
        YamlScalarSemantic::Null => Some("null".len()),
        _ => None,
    }
}

fn normalize_yaml_bool(value: &str) -> Option<String> {
    match value {
        "true" | "True" | "TRUE" => Some("true".to_owned()),
        "false" | "False" | "FALSE" => Some("false".to_owned()),
        _ => None,
    }
}

fn normalized_yaml_bool_width(value: &str) -> Option<usize> {
    match value {
        "true" | "True" | "TRUE" => Some("true".len()),
        "false" | "False" | "FALSE" => Some("false".len()),
        _ => None,
    }
}

fn explicit_core_scalar_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    metadata: ScalarMetadata,
    value_width: usize,
) -> usize {
    let prefix_width = scalar_property_prefix_width(source, scalar, metadata);
    if prefix_width == 0 {
        value_width
    } else {
        prefix_width + 1 + value_width
    }
}

fn render_explicit_core_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    value: &str,
) -> String {
    let metadata = scalar_metadata(source, scalar.value);
    let prefix = scalar_property_prefix(source, scalar, metadata);
    if prefix.is_empty() {
        value.to_owned()
    } else {
        format!("{prefix} {value}")
    }
}

fn scalar_property_prefix_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    metadata: ScalarMetadata,
) -> usize {
    let inline_prefix = source
        .slice(Span::new(scalar.value.start(), metadata.content.start))
        .trim();
    if !inline_prefix.is_empty() {
        return inline_prefix.chars().count();
    }
    match (scalar.tag, scalar.anchor) {
        (Some(tag), Some(anchor)) if tag.start() <= anchor.start() => {
            source.slice(tag).chars().count() + 1 + source.slice(anchor).chars().count()
        }
        (Some(tag), Some(anchor)) => {
            source.slice(anchor).chars().count() + 1 + source.slice(tag).chars().count()
        }
        (Some(tag), None) => source.slice(tag).chars().count(),
        (None, Some(anchor)) => source.slice(anchor).chars().count(),
        (None, None) => 0,
    }
}

fn simplify_quoted_string_scalar_width(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    flow_context: bool,
) -> Option<usize> {
    if !matches!(
        scalar.style,
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted
    ) || scalar.semantic != YamlScalarSemantic::String
        || scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    let raw = source.slice(scalar.value).trim();
    if raw.contains(['\n', '\r']) {
        return None;
    }
    if let Some(decoded) = simple_quoted_scalar_inner(raw) {
        return plain_string_safe(decoded, flow_context).then(|| decoded.chars().count());
    }
    match decoded_quoted_plain_width(raw, flow_context) {
        Some(DecodedPlainWidth::Simplified(width)) => return Some(width),
        Some(DecodedPlainWidth::PreserveSource) => return None,
        None => {}
    }
    simplify_quoted_string_scalar(source, scalar, flow_context).map(|value| value.chars().count())
}

fn scalar_property_prefix(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    metadata: ScalarMetadata,
) -> String {
    let inline_prefix = source
        .slice(Span::new(scalar.value.start(), metadata.content.start))
        .trim();
    if !inline_prefix.is_empty() {
        return inline_prefix.to_owned();
    }
    let mut properties = [scalar.tag, scalar.anchor]
        .into_iter()
        .flatten()
        .map(|span| (span.start(), source.slice(span)))
        .collect::<Vec<_>>();
    properties.sort_by_key(|(start, _)| *start);
    properties
        .into_iter()
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn plain_string_safe(value: &str, flow_context: bool) -> bool {
    scalar_semantic(value) == YamlScalarSemantic::String
        && if flow_context {
            flow_plain_scalar_safe(value)
        } else {
            block_plain_string_safe(value)
        }
}

fn block_plain_string_safe(value: &str) -> bool {
    if value.is_empty() || plain_scalar_unsafe_start_value(value) {
        return false;
    }
    let mut chars = value.chars();
    let mut previous = chars.next().expect("non-empty strings have a first char");
    if previous.is_whitespace() || previous.is_control() {
        return false;
    }
    for ch in chars {
        if ch.is_control() || matches!((previous, ch), (':', ' ') | (' ', '#')) {
            return false;
        }
        previous = ch;
    }
    !previous.is_whitespace()
}

fn flow_plain_scalar_safe(value: &str) -> bool {
    if value.is_empty() || plain_scalar_unsafe_start_value(value) {
        return false;
    }
    let mut previous = None;
    for ch in value.chars() {
        if ch.is_control()
            || matches!(ch, ',' | '[' | ']' | '{' | '}')
            || matches!((previous, ch), (Some(':'), ' ') | (Some(' '), '#'))
        {
            return false;
        }
        previous = Some(ch);
    }
    true
}

enum DecodedPlainWidth {
    Simplified(usize),
    PreserveSource,
}

fn decoded_quoted_plain_width(raw: &str, flow_context: bool) -> Option<DecodedPlainWidth> {
    let state = decoded_quoted_string_width_state(raw, flow_context)?;
    if let Some(width) = state.plain_safe_width() {
        return Some(DecodedPlainWidth::Simplified(width));
    }
    state
        .known_not_plain_string_safe()
        .then_some(DecodedPlainWidth::PreserveSource)
}

fn decoded_quoted_explicit_string_width(raw: &str, flow_context: bool) -> Option<usize> {
    decoded_quoted_string_width_state(raw, flow_context)?.explicit_string_width()
}

fn decoded_quoted_explicit_core_width(raw: &str, tag: Option<&str>) -> Option<usize> {
    let mut state = DecodedCoreScalarState::new();
    if raw.starts_with('"') {
        decode_double_quoted_scalar_chars(raw, |ch| state.push(ch))?;
    } else if raw.starts_with('\'') {
        decode_single_quoted_scalar_chars(raw, |ch| state.push(ch))?;
    } else {
        return None;
    }
    match tag {
        Some("!!bool") => state.normalized_bool_width(),
        Some("!!null") => state.normalized_null_width(),
        Some("!!int") => state.normalized_int_width(),
        Some("!!float") => state.normalized_float_width(),
        _ => None,
    }
}

fn decoded_quoted_string_width_state(
    raw: &str,
    flow_context: bool,
) -> Option<DecodedStringWidthState> {
    let mut state = DecodedStringWidthState::new(flow_context);
    if raw.starts_with('"') {
        decode_double_quoted_scalar_chars(raw, |ch| state.push(ch))?;
    } else if raw.starts_with('\'') {
        decode_single_quoted_scalar_chars(raw, |ch| state.push(ch))?;
    } else {
        return None;
    }
    Some(state)
}

struct DecodedCoreScalarState {
    bytes: [u8; 64],
    len: usize,
    valid: bool,
}

impl DecodedCoreScalarState {
    fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
            valid: true,
        }
    }

    fn push(&mut self, ch: char) {
        if self.len == self.bytes.len() || !ch.is_ascii() {
            self.valid = false;
            return;
        }
        self.bytes[self.len] = ch as u8;
        self.len += 1;
    }

    fn value(&self) -> Option<&[u8]> {
        self.valid.then_some(&self.bytes[..self.len])
    }

    fn normalized_bool_width(&self) -> Option<usize> {
        match self.value()? {
            b"true" | b"True" | b"TRUE" => Some("true".len()),
            b"false" | b"False" | b"FALSE" => Some("false".len()),
            _ => None,
        }
    }

    fn normalized_null_width(&self) -> Option<usize> {
        match self.value()? {
            b"~" | b"null" | b"Null" | b"NULL" => Some("null".len()),
            _ => None,
        }
    }

    fn normalized_int_width(&self) -> Option<usize> {
        let value = self.value()?;
        let digits = strip_sign_bytes(value);
        yaml_decimal_digits_bytes(digits).then_some(value.len())
    }

    fn normalized_float_width(&self) -> Option<usize> {
        let value = self.value()?;
        yaml_float_semantic_bytes(value).then_some(value.len())
    }
}

fn non_string_core_semantic_bytes(value: &[u8]) -> Option<YamlScalarSemantic> {
    if yaml_null_bytes(value) {
        Some(YamlScalarSemantic::Null)
    } else if yaml_bool_bytes(value) {
        Some(YamlScalarSemantic::Boolean)
    } else if yaml_integer_bytes(value) {
        Some(YamlScalarSemantic::Integer)
    } else if yaml_float_semantic_bytes(value) {
        Some(YamlScalarSemantic::Float)
    } else {
        None
    }
}

fn yaml_integer_bytes(value: &[u8]) -> bool {
    let unsigned = strip_sign_bytes(value);
    yaml_decimal_digits_bytes(unsigned) || yaml_prefixed_integer_bytes(unsigned)
}

fn yaml_prefixed_integer_bytes(value: &[u8]) -> bool {
    let Some((prefix, digits)) = value.split_first_chunk::<2>() else {
        return false;
    };
    match prefix {
        b"0b" | b"0B" => yaml_radix_digits_bytes(digits, |byte| matches!(byte, b'0' | b'1')),
        b"0o" | b"0O" => yaml_radix_digits_bytes(digits, |byte| matches!(byte, b'0'..=b'7')),
        b"0x" | b"0X" => yaml_radix_digits_bytes(digits, |byte| byte.is_ascii_hexdigit()),
        _ => false,
    }
}

fn yaml_radix_digits_bytes(value: &[u8], valid: impl Fn(u8) -> bool) -> bool {
    let mut saw_digit = false;
    let mut previous_underscore = false;
    for byte in value {
        if valid(*byte) {
            saw_digit = true;
            previous_underscore = false;
        } else if *byte == b'_' && saw_digit && !previous_underscore {
            previous_underscore = true;
        } else {
            return false;
        }
    }
    saw_digit && !previous_underscore
}

fn yaml_null_bytes(value: &[u8]) -> bool {
    value.is_empty() || matches!(value, b"~" | b"null" | b"Null" | b"NULL")
}

fn yaml_bool_bytes(value: &[u8]) -> bool {
    matches!(
        value,
        b"true" | b"True" | b"TRUE" | b"false" | b"False" | b"FALSE"
    )
}

fn strip_sign_bytes(value: &[u8]) -> &[u8] {
    match value.first() {
        Some(b'+' | b'-') => &value[1..],
        Some(_) | None => value,
    }
}

fn yaml_decimal_digits_bytes(value: &[u8]) -> bool {
    let mut saw_digit = false;
    let mut previous_underscore = false;
    for byte in value {
        if byte.is_ascii_digit() {
            saw_digit = true;
            previous_underscore = false;
        } else if *byte == b'_' && saw_digit && !previous_underscore {
            previous_underscore = true;
        } else {
            return false;
        }
    }
    saw_digit && !previous_underscore
}

fn yaml_float_special_bytes(value: &[u8]) -> bool {
    let unsigned = strip_sign_bytes(value);
    matches!(unsigned, b".inf" | b".Inf" | b".INF") || matches!(value, b".nan" | b".NaN" | b".NAN")
}

fn yaml_float_semantic_bytes(value: &[u8]) -> bool {
    if yaml_float_special_bytes(value) {
        return true;
    }
    let unsigned = strip_sign_bytes(value);
    if unsigned.is_empty() || yaml_decimal_digits_bytes(unsigned) {
        return false;
    }
    yaml_float_spelling_bytes(value)
}

fn yaml_float_spelling_bytes(value: &[u8]) -> bool {
    let unsigned = strip_sign_bytes(value);
    if unsigned.is_empty() {
        return false;
    }
    if yaml_decimal_digits_bytes(unsigned) {
        return true;
    }
    let Some(number_end) = unsigned.iter().position(|byte| matches!(byte, b'e' | b'E')) else {
        return yaml_decimal_fraction_bytes(unsigned);
    };
    let (number, exponent) = unsigned.split_at(number_end);
    let exponent = &exponent[1..];
    yaml_float_number_part_bytes(number) && yaml_decimal_digits_bytes(strip_sign_bytes(exponent))
}

fn yaml_float_number_part_bytes(value: &[u8]) -> bool {
    yaml_decimal_digits_bytes(value) || yaml_decimal_fraction_bytes(value)
}

fn yaml_decimal_fraction_bytes(value: &[u8]) -> bool {
    let Some(dot) = value.iter().position(|byte| *byte == b'.') else {
        return false;
    };
    let (before, after) = value.split_at(dot);
    let after = &after[1..];
    (before.is_empty() || yaml_decimal_digits_bytes(before))
        && (after.is_empty() || yaml_decimal_digits_bytes(after))
        && !(before.is_empty() && after.is_empty())
}

struct DecodedStringWidthState {
    flow_context: bool,
    width: usize,
    bytes: usize,
    first: Option<char>,
    last: Option<char>,
    previous: Option<char>,
    has_space: bool,
    has_string_semantic_marker: bool,
    plain_unsafe: bool,
    core_bytes: [u8; 16],
    core_len: usize,
    core_valid: bool,
    single_quote_count: usize,
    has_control: bool,
    double_bytes: usize,
    double_chars: usize,
}

impl DecodedStringWidthState {
    fn new(flow_context: bool) -> Self {
        Self {
            flow_context,
            width: 0,
            bytes: 0,
            first: None,
            last: None,
            previous: None,
            has_space: false,
            has_string_semantic_marker: false,
            plain_unsafe: false,
            core_bytes: [0; 16],
            core_len: 0,
            core_valid: true,
            single_quote_count: 0,
            has_control: false,
            double_bytes: 2,
            double_chars: 2,
        }
    }

    fn push(&mut self, ch: char) {
        if self.first.is_none() {
            self.first = Some(ch);
            if plain_scalar_unsafe_first_char(ch) {
                self.plain_unsafe = true;
            }
        }
        if self.previous == Some('-') && ch.is_whitespace() {
            self.plain_unsafe = true;
        }
        if ch == ' ' {
            self.has_space = true;
        }
        if !yaml_core_non_string_semantic_char(ch) {
            self.has_string_semantic_marker = true;
            self.core_valid = false;
        }
        if self.core_valid {
            if self.core_len == self.core_bytes.len() || !ch.is_ascii() {
                self.core_valid = false;
            } else {
                self.core_bytes[self.core_len] = ch as u8;
                self.core_len += 1;
            }
        }
        if ch == '\'' {
            self.single_quote_count += 1;
        }
        if ch.is_control()
            || matches!(ch, '\n' | '\r' | '\t')
            || self.flow_context && matches!(ch, ',' | '[' | ']' | '{' | '}')
            || matches!((self.previous, ch), (Some(':'), ' ') | (Some(' '), '#'))
        {
            self.plain_unsafe = true;
        }
        if ch.is_control() {
            self.has_control = true;
        }
        let (double_bytes, double_chars) = double_quote_char_metrics(ch);
        self.double_bytes += double_bytes;
        self.double_chars += double_chars;
        self.previous = Some(ch);
        self.last = Some(ch);
        self.width += 1;
        self.bytes += ch.len_utf8();
    }

    fn plain_safe_width(&self) -> Option<usize> {
        if !self.plain_unsafe
            && self.width > 0
            && (self.has_space || self.has_string_semantic_marker)
            && !self.first?.is_whitespace()
            && !self.last?.is_whitespace()
        {
            Some(self.width)
        } else {
            None
        }
    }

    fn explicit_string_width(&self) -> Option<usize> {
        if let Some(width) = self.plain_safe_width() {
            return Some(width);
        }
        if self.known_not_plain_string_safe() {
            return Some(self.quoted_width());
        }
        None
    }

    fn known_not_plain_string_safe(&self) -> bool {
        self.width == 0
            || self.plain_unsafe
            || matches!(self.first, Some(ch) if ch.is_whitespace())
            || matches!(self.last, Some(ch) if ch.is_whitespace())
            || self.non_string_core_semantic().is_some()
    }

    fn non_string_core_semantic(&self) -> Option<YamlScalarSemantic> {
        non_string_core_semantic_bytes(self.core_value()?)
    }

    fn core_value(&self) -> Option<&[u8]> {
        self.core_valid.then_some(&self.core_bytes[..self.core_len])
    }

    fn quoted_width(&self) -> usize {
        if self.has_control {
            return self.double_chars;
        }
        let single_bytes = self.bytes + 2 + self.single_quote_count;
        let single_chars = self.width + 2 + self.single_quote_count;
        if self.double_bytes < single_bytes {
            self.double_chars
        } else {
            single_chars
        }
    }
}

// Characters outside this set cannot appear in the current null/bool/int/float
// recognizers, so a plain-safe decoded scalar containing one is a string.
fn yaml_core_non_string_semantic_char(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(
            ch,
            '+' | '-'
                | '_'
                | '.'
                | '~'
                | 'a'
                | 'A'
                | 'e'
                | 'E'
                | 'f'
                | 'F'
                | 'i'
                | 'I'
                | 'l'
                | 'L'
                | 'n'
                | 'N'
                | 'r'
                | 'R'
                | 's'
                | 'S'
                | 't'
                | 'T'
                | 'u'
                | 'U'
        )
}

fn plain_scalar_unsafe_start_value(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes.first().copied() {
        Some(b'-') => bytes.get(1).is_none_or(u8::is_ascii_whitespace),
        Some(byte) => plain_scalar_unsafe_start_byte(byte),
        None => true,
    }
}

fn plain_scalar_unsafe_first_char(ch: char) -> bool {
    ch.is_ascii() && plain_scalar_unsafe_start_byte(ch as u8)
}

fn plain_scalar_unsafe_start_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'?' | b':'
            | b','
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'#'
            | b'&'
            | b'*'
            | b'!'
            | b'|'
            | b'>'
            | b'@'
            | b'`'
            | b'"'
            | b'\''
            | b'%'
    )
}

fn decode_single_quoted_scalar_chars(raw: &str, mut push: impl FnMut(char)) -> Option<()> {
    let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.next() == Some('\'') {
                push('\'');
            } else {
                return None;
            }
        } else {
            push(ch);
        }
    }
    Some(())
}

fn decode_double_quoted_scalar_chars(raw: &str, mut push: impl FnMut(char)) -> Option<()> {
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            if matches!(ch, '\r' | '\n') {
                return None;
            }
            push(ch);
            continue;
        }
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
            _ => return None,
        };
        push(decoded);
    }
    Some(())
}

fn double_quote_char_metrics(ch: char) -> (usize, usize) {
    match ch {
        '\0' | '\u{0007}' | '\u{0008}' | '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r'
        | '\u{001b}' | '"' | '\\' => (2, 2),
        ch if ch.is_control() => {
            let bytes = if ch as u32 <= 0xff {
                4
            } else {
                2 + hex_digits(ch as u32)
            };
            (bytes, bytes)
        }
        ch => (ch.len_utf8(), 1),
    }
}

fn hex_digits(mut value: u32) -> usize {
    let mut digits = 1;
    while value >= 16 {
        value /= 16;
        digits += 1;
    }
    digits
}

fn quote_yaml_single_for_flow(value: &str) -> Option<String> {
    if value.chars().any(char::is_control) {
        return Some(quote_yaml_double_for_flow(value));
    }
    let single = format!("'{}'", value.replace('\'', "''"));
    let double = quote_yaml_double_for_flow(value);
    if double.len() < single.len() {
        Some(double)
    } else {
        Some(single)
    }
}

fn quote_yaml_single_for_flow_width(value: &str) -> usize {
    let single_bytes = value.len() + 2 + value.matches('\'').count();
    let single_chars = value.chars().count() + 2 + value.matches('\'').count();
    if value.chars().any(char::is_control) {
        let (_, double_chars) = quote_yaml_double_for_flow_metrics(value);
        return double_chars;
    }
    let (double_bytes, double_chars) = quote_yaml_double_for_flow_metrics(value);
    if double_bytes < single_bytes {
        double_chars
    } else {
        single_chars
    }
}

fn quote_yaml_double_for_flow(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\0' => out.push_str("\\0"),
            '\u{0007}' => out.push_str("\\a"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000b}' => out.push_str("\\v"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\u{001b}' => out.push_str("\\e"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ch if ch.is_control() => out.push_str(&format!("\\x{:02X}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn quote_yaml_double_for_flow_metrics(value: &str) -> (usize, usize) {
    let mut bytes = 2usize;
    let mut chars = 2usize;
    for ch in value.chars() {
        match ch {
            '\0' | '\u{0007}' | '\u{0008}' | '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r'
            | '\u{001b}' | '"' | '\\' => {
                bytes += 2;
                chars += 2;
            }
            ch if ch.is_control() => {
                let escaped = format!("\\x{:02X}", ch as u32);
                bytes += escaped.len();
                chars += escaped.chars().count();
            }
            ch => {
                bytes += ch.len_utf8();
                chars += 1;
            }
        }
    }
    (bytes, chars)
}

fn flow_trailing_comment<'src>(kind: &YamlAstKind<'src>) -> Option<SourceSpan<'src>> {
    match kind {
        YamlAstKind::FlowSequence(sequence) => sequence.trailing_comment,
        YamlAstKind::FlowMapping(mapping) => mapping.trailing_comment,
        _ => None,
    }
}

fn emit_yaml_scalar_after_prefix(
    out: &mut String,
    context: YamlEmitContext<'_>,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    body_indent: Option<usize>,
) -> Result<()> {
    let YamlEmitContext {
        source,
        document,
        options,
        plugins,
        ..
    } = context;
    match &node.emit {
        YamlEmitPlan::Rendered(_) => {
            let state = document.state(node.state);
            emit_yaml_rendered_scalar_plan(out, source, scalar, node, state, options, body_indent);
            Ok(())
        }
        _ => emit_yaml_scalar(out, source, document, scalar, node, options, plugins),
    }
}

fn render_yaml_block_scalar_value_header(source: &SourceBuffer, scalar: &YamlScalar<'_>) -> String {
    let header = scalar.header.expect("block scalars have headers");
    let mut output = if scalar.nested.is_some() && scalar.style == YamlScalarStyle::FoldedBlock {
        format_markdown_yaml_block_value(source.slice(scalar.value))
    } else {
        source.slice(scalar.value).trim().to_owned()
    };
    emit_inline_comment(&mut output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, header));
    output
}

#[allow(clippy::too_many_arguments)]
fn emit_yaml_nested_markdown_block_scalar(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    plugins: &PluginRegistry,
    nested: usize,
) -> Result<()> {
    let header_span = scalar.header.expect("block scalars have headers");
    out.push_str(&render_yaml_block_scalar_value_header(source, scalar));
    let state = document.state(node.state);
    let mut nested_output = crate::core::emit::emit_document(
        source,
        &document.nested[nested],
        state.markdown_options(options),
        plugins,
    )?;
    if !nested_output.is_empty() && !nested_output.ends_with('\n') && !nested_output.ends_with('\r')
    {
        nested_output.push_str(line_ending_for_span(source, header_span));
    }
    out.push_str(&reindent_yaml_block_scalar_body(
        source,
        scalar,
        node,
        &nested_output,
        options,
    ));
    Ok(())
}

fn external_block_scalar_action(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
    plugins: &PluginRegistry,
    name: &str,
) -> Result<Option<ExternalBlockScalarAction>> {
    let Some(body) = scalar.body else {
        return Ok(None);
    };
    let line = source.line_column_at_byte(body.start()).0;
    let formatter_source = yaml_block_scalar_formatter_input(source, scalar, node, options);
    let (preamble, formatter_input) = crate::core::emit::split_renderer_preamble(&formatter_source);
    Ok(Some(match plugins.run(name, formatter_input, line)? {
        Some(mut formatted) => {
            append_trailing_line_ending_if_missing(
                &mut formatted,
                external_block_scalar_line_ending(source, scalar, options),
            );
            if !preamble.is_empty() {
                formatted.insert_str(0, preamble);
            }
            ExternalBlockScalarAction::Formatted(formatted)
        }
        None => ExternalBlockScalarAction::Preserve,
    }))
}

fn external_block_scalar_line_ending(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    options: FormatOptions,
) -> &'static str {
    let header = scalar.header.expect("external block scalars have headers");
    let line_ending = line_ending_for_span(source, header);
    if line_ending.is_empty() {
        options.default_line_ending
    } else {
        line_ending
    }
}

fn append_trailing_line_ending_if_missing(text: &mut String, line_ending: &str) {
    if !text.is_empty() && !text.ends_with('\n') && !text.ends_with('\r') {
        text.push_str(line_ending);
    }
}

fn yaml_block_scalar_formatter_input(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    options: FormatOptions,
) -> String {
    let Some(source_body) = scalar.body else {
        return String::new();
    };
    let strip_indent = explicit_block_scalar_body_indent(source, scalar, node)
        .or_else(|| block_scalar_body_indent(source.slice(source_body)))
        .unwrap_or_else(|| {
            let line_index = source.line_at_byte(node.span.start());
            indentation(source.line_text(line_index)) + options.indent_width
        });
    reindent_block_lines(source.slice(source_body), strip_indent, 0)
}

fn emit_yaml_formatted_external_block_scalar(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    formatted: &str,
    options: FormatOptions,
) {
    let header = scalar.header.expect("block scalars have headers");
    out.push_str(&format_external_yaml_block_header(
        source.slice(header),
        formatted,
    ));
    out.push_str(&reindent_yaml_block_scalar_body(
        source, scalar, node, formatted, options,
    ));
}

fn emit_yaml_formatted_external_block_scalar_after_prefix(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
    formatted: &str,
    options: FormatOptions,
) {
    let value_header = render_yaml_block_scalar_value_header(source, scalar);
    out.push_str(&format_external_yaml_block_header(&value_header, formatted));
    out.push_str(&reindent_yaml_block_scalar_body(
        source, scalar, node, formatted, options,
    ));
}

fn format_external_yaml_block_header(header: &str, formatted: &str) -> String {
    let header = format_simple_yaml_line(header);
    rewrite_yaml_block_chomp(&header, chomp_for_block_scalar_output(formatted))
}

fn chomp_for_block_scalar_output(formatted: &str) -> YamlBlockChomp {
    if formatted.is_empty() || (!formatted.ends_with('\n') && !formatted.ends_with('\r')) {
        return YamlBlockChomp::Strip;
    }
    if formatted_ends_with_multiple_line_endings(formatted) {
        YamlBlockChomp::Keep
    } else {
        YamlBlockChomp::Clip
    }
}

fn formatted_ends_with_multiple_line_endings(formatted: &str) -> bool {
    let Some(without_last) = formatted
        .strip_suffix("\r\n")
        .or_else(|| formatted.strip_suffix('\n'))
        .or_else(|| formatted.strip_suffix('\r'))
    else {
        return false;
    };
    without_last.ends_with('\n') || without_last.ends_with('\r')
}

fn rewrite_yaml_block_chomp(header: &str, chomp: YamlBlockChomp) -> String {
    let (body, newline) = strip_newline(header);
    let comment_start = find_trailing_comment(body, 0).unwrap_or(body.len());
    let Some(marker) = body[..comment_start].rfind(['|', '>']) else {
        return header.to_owned();
    };
    let mut out = String::with_capacity(header.len() + 1);
    out.push_str(&body[..marker + 1]);
    let indicators = &body[marker + 1..comment_start];
    for ch in indicators.chars() {
        if !matches!(ch, '-' | '+') {
            out.push(ch);
        }
    }
    match chomp {
        YamlBlockChomp::Clip => {}
        YamlBlockChomp::Strip => out.push('-'),
        YamlBlockChomp::Keep => out.push('+'),
    }
    out.push_str(&body[comment_start..]);
    out.push_str(newline);
    out
}

fn yaml_node_should_preserve(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode<'_>,
) -> bool {
    if let Some(must_preserve_source) = node.must_preserve_source {
        return must_preserve_source;
    }
    yaml_node_should_preserve_uncached(source, document, node)
}

fn yaml_node_should_preserve_uncached(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode<'_>,
) -> bool {
    yaml_node_should_preserve_uncached_with_template_possible(source, document, node, true)
}

fn yaml_node_should_preserve_uncached_with_template_possible(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode<'_>,
    template_spans_possible: bool,
) -> bool {
    let state = document.state(node.state);
    state.preserve
        || (template_spans_possible && yaml_node_preserves_own_template_span(&node.kind) && {
            let text = source.slice(node.span);
            source_may_contain_template_span(text, &state.template_delimiters)
                && preserves_yaml_template_span(text, &state.template_delimiters)
        })
        || matches!(
            &node.kind,
            YamlAstKind::Scalar(scalar)
                if scalar.value.is_empty()
                    && scalar_has_properties(scalar)
                    && !yaml_scalar_is_markdown_target(source, document, scalar, node)
        )
}

fn yaml_node_preserves_own_template_span(kind: &YamlAstKind<'_>) -> bool {
    matches!(
        kind,
        YamlAstKind::Scalar(_)
            | YamlAstKind::FlowSequence(_)
            | YamlAstKind::FlowMapping(_)
            | YamlAstKind::Opaque(_)
    )
}

fn source_contains_any_template_opener(
    source: &str,
    delimiters: &[crate::core::directives::TemplateDelimiter],
) -> bool {
    delimiters.iter().any(|delimiter| {
        !delimiter.open.is_empty()
            && !delimiter.close.is_empty()
            && source.contains(&delimiter.open)
    })
}

fn source_may_contain_template_span(
    source: &str,
    delimiters: &[crate::core::directives::TemplateDelimiter],
) -> bool {
    let source = source.as_bytes();
    delimiters.iter().enumerate().any(|(index, delimiter)| {
        let Some(first) = delimiter.open.as_bytes().first() else {
            return false;
        };
        if delimiters[..index]
            .iter()
            .any(|prior| prior.open.as_bytes().first() == Some(first))
        {
            return false;
        }
        source.contains(first)
    })
}

fn yaml_node_has_properties(source: &SourceBuffer, node: &YamlAstNode<'_>) -> bool {
    match &node.kind {
        YamlAstKind::Mapping(mapping) => {
            mapping.anchor.is_some()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Mapping,
                    mapping.tag,
                )
        }
        YamlAstKind::Sequence(sequence) => {
            sequence.anchor.is_some()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Sequence,
                    sequence.tag,
                )
        }
        YamlAstKind::FlowMapping(mapping) => {
            mapping.anchor.is_some()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Mapping,
                    mapping.tag,
                )
        }
        YamlAstKind::FlowSequence(sequence) => {
            sequence.anchor.is_some()
                || collection_has_non_removable_tag(
                    source,
                    YamlCollectionKind::Sequence,
                    sequence.tag,
                )
        }
        _ => yaml_node_properties(&node.kind)
            .is_some_and(|(tag, anchor)| tag.is_some() || anchor.is_some()),
    }
}

fn scalar_has_properties(scalar: &YamlScalar<'_>) -> bool {
    scalar.tag.is_some() || scalar.anchor.is_some()
}

fn yaml_scalar_is_markdown_target(
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar<'_>,
    node: &YamlAstNode<'_>,
) -> bool {
    document.state(node.state).markdown_target
        || scalar
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(source.slice(tag)))
}

fn yaml_tag_is_markdown(tag: &str) -> bool {
    matches!(tag, "!markdown" | "!md")
}

fn markdown_block_scalar_body_is_empty(source: &SourceBuffer, scalar: &YamlScalar<'_>) -> bool {
    scalar
        .body
        .is_some_and(|body| source.slice(body).is_empty())
}

fn render_empty_markdown_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar<'_>,
    trailing_comment: impl IntoOptionalSpan,
    newline: &str,
) -> String {
    let metadata = scalar_metadata(source, scalar.value);
    let prefix = scalar_property_prefix(source, scalar, metadata);
    let mut out = String::new();
    if !prefix.is_empty() {
        out.push_str(&prefix);
        out.push(' ');
    }
    out.push_str("\"\"");
    emit_inline_comment(&mut out, source, trailing_comment);
    out.push_str(newline);
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YamlCollectionKind {
    Sequence,
    Mapping,
}

fn collection_has_non_removable_tag(
    source: &SourceBuffer,
    kind: YamlCollectionKind,
    tag: impl IntoOptionalSpan,
) -> bool {
    tag.into_optional_span()
        .is_some_and(|tag| !collection_tag_is_core(source.slice(tag), kind))
}

fn collection_tag_is_core(tag: &str, kind: YamlCollectionKind) -> bool {
    matches!(
        (kind, tag),
        (YamlCollectionKind::Sequence, "!!seq") | (YamlCollectionKind::Mapping, "!!map")
    )
}

fn collection_tag_is_removable(
    source: &SourceBuffer,
    kind: &YamlAstKind<'_>,
    tag: impl Into<Span>,
) -> bool {
    let tag = tag.into();
    let kind = match kind {
        YamlAstKind::Sequence(_) | YamlAstKind::FlowSequence(_) => YamlCollectionKind::Sequence,
        YamlAstKind::Mapping(_) | YamlAstKind::FlowMapping(_) => YamlCollectionKind::Mapping,
        _ => return false,
    };
    collection_tag_is_core(source.slice(tag), kind)
}

fn emit_collection_property_prefix_into(
    out: &mut String,
    source: &SourceBuffer,
    kind: YamlCollectionKind,
    tag: Option<SourceSpan<'_>>,
    anchor: Option<SourceSpan<'_>>,
) -> Option<()> {
    if collection_has_non_removable_tag(source, kind, tag) {
        return None;
    }
    if let Some(anchor) = anchor {
        out.push_str(source.slice(anchor));
        out.push(' ');
    }
    Some(())
}

fn collection_property_prefix_width(
    source: &SourceBuffer,
    kind: YamlCollectionKind,
    tag: Option<SourceSpan<'_>>,
    anchor: Option<SourceSpan<'_>>,
) -> Option<usize> {
    if collection_has_non_removable_tag(source, kind, tag) {
        return None;
    }
    Some(
        anchor
            .map(|anchor| source.slice(anchor).chars().count() + 1)
            .unwrap_or(0),
    )
}

fn mapping_pair_child_indent(_node: &YamlAstNode<'_>, default: usize) -> usize {
    default
}

fn yaml_node_source_indent(source: &SourceBuffer, node: &YamlAstNode<'_>) -> usize {
    if let Some(indent) = node.source_indent() {
        return indent;
    }
    let line = source.line_at_byte(node.span.start());
    let indent = indentation(source.line_text(line));
    node.set_source_indent(indent);
    indent
}

fn yaml_node_properties<'src>(
    kind: &YamlAstKind<'src>,
) -> Option<(Option<SourceSpan<'src>>, Option<SourceSpan<'src>>)> {
    match kind {
        YamlAstKind::Mapping(mapping) => Some((mapping.tag, mapping.anchor)),
        YamlAstKind::Sequence(sequence) => Some((sequence.tag, sequence.anchor)),
        YamlAstKind::FlowMapping(mapping) => Some((mapping.tag, mapping.anchor)),
        YamlAstKind::FlowSequence(sequence) => Some((sequence.tag, sequence.anchor)),
        _ => None,
    }
}

fn emit_yaml_node_properties(out: &mut String, source: &SourceBuffer, node: &YamlAstNode<'_>) {
    if let Some((tag, anchor)) = yaml_node_properties(&node.kind) {
        if let Some(tag) = tag
            && collection_tag_is_removable(source, &node.kind, tag)
        {
            if let Some(anchor) = anchor {
                out.push(' ');
                out.push_str(source.slice(anchor));
            }
            return;
        }
        let mut properties = [tag, anchor]
            .into_iter()
            .flatten()
            .map(|span| (span.start(), source.slice(span)))
            .collect::<Vec<_>>();
        properties.sort_by_key(|(start, _)| *start);
        for (_, property) in properties {
            out.push(' ');
            out.push_str(property);
        }
    }
}

fn emit_inline_comment(out: &mut String, source: &SourceBuffer, comment: impl IntoOptionalSpan) {
    if let Some(comment) = comment.into_optional_span() {
        out.push(' ');
        out.push_str(source.slice(comment).trim_end());
    }
}

fn yaml_node_is_root(ast: &YamlDocumentAst<'_>, id: YamlNodeId) -> bool {
    ast.roots.iter().any(|root| root.node == Some(id))
}

fn compact_root_collection_allowed(ast: &YamlDocumentAst<'_>, id: YamlNodeId) -> bool {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(mapping) => mapping.pairs.iter().all(|pair| {
            pair.value
                .is_some_and(|value| !yaml_node_is_block_collection(ast, value))
        }),
        YamlAstKind::Sequence(sequence) => sequence.items.iter().all(|item| {
            item.value
                .is_some_and(|value| !yaml_node_is_block_collection(ast, value))
        }),
        _ => true,
    }
}

fn yaml_node_is_block_collection(ast: &YamlDocumentAst<'_>, id: YamlNodeId) -> bool {
    matches!(
        ast.node(id).kind,
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
    )
}

fn yaml_block_collection_has_flow_collapse_hint(node: &YamlAstNode<'_>) -> bool {
    match &node.kind {
        YamlAstKind::Mapping(mapping) => mapping.flow_collapse_hint.is_some(),
        YamlAstKind::Sequence(sequence) => sequence.flow_collapse_hint.is_some(),
        _ => false,
    }
}

fn compact_yaml_value_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<usize> {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
            compact_yaml_node_width(source, document, ast, id)
        }
        _ => render_yaml_inline_node_width_for_flow(source, document, ast, id),
    }
}

fn compact_yaml_node_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<usize> {
    let node = ast.node(id);
    if yaml_node_should_preserve(source, document, node)
        || yaml_node_has_properties(source, node)
        || !node.leading_trivia.is_empty()
    {
        return None;
    }
    match &node.kind {
        YamlAstKind::Mapping(mapping) => {
            if mapping.pairs.is_empty() {
                return None;
            }
            let mut width = 2;
            for (index, pair) in mapping.pairs.iter().enumerate() {
                if !pair.leading_trivia.is_empty() || pair.trailing_comment.is_some() {
                    return None;
                }
                let value = pair.value?;
                let value_node = ast.node(value);
                if yaml_node_should_preserve(source, document, value_node) {
                    return None;
                }
                if index > 0 {
                    width += 2;
                }
                width += render_mapping_key_width_for_flow(source, document, ast, pair)?;
                width += 2 + compact_yaml_value_width(source, document, ast, value)?;
            }
            Some(width)
        }
        YamlAstKind::Sequence(sequence) => {
            if sequence.items.is_empty() {
                return None;
            }
            let mut width = 2;
            for (index, item) in sequence.items.iter().enumerate() {
                if !item.leading_trivia.is_empty() || item.trailing_comment.is_some() {
                    return None;
                }
                let value = item.value?;
                let value_node = ast.node(value);
                if yaml_node_should_preserve(source, document, value_node) {
                    return None;
                }
                if index > 0 {
                    width += 2;
                }
                width += compact_yaml_value_width(source, document, ast, value)?;
            }
            Some(width)
        }
        _ => None,
    }
}

fn emit_compact_yaml_node(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<()> {
    let start_len = out.len();
    if emit_compact_yaml_node_inner(out, source, document, ast, id).is_some() {
        Some(())
    } else {
        out.truncate(start_len);
        None
    }
}

fn emit_compact_yaml_node_inner(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<()> {
    let node = ast.node(id);
    if yaml_node_should_preserve(source, document, node)
        || yaml_node_has_properties(source, node)
        || !node.leading_trivia.is_empty()
    {
        return None;
    }
    match &node.kind {
        YamlAstKind::Mapping(mapping) => {
            if mapping.pairs.is_empty() {
                return None;
            }
            out.push('{');
            for (index, pair) in mapping.pairs.iter().enumerate() {
                if !pair.leading_trivia.is_empty() || pair.trailing_comment.is_some() {
                    return None;
                }
                let value = pair.value?;
                let value_node = ast.node(value);
                if yaml_node_should_preserve(source, document, value_node) {
                    return None;
                }
                if index > 0 {
                    out.push_str(", ");
                }
                emit_mapping_key_for_flow(out, source, document, ast, pair)?;
                out.push_str(": ");
                emit_compact_yaml_value(out, source, document, ast, value)?;
            }
            out.push('}');
        }
        YamlAstKind::Sequence(sequence) => {
            if sequence.items.is_empty() {
                return None;
            }
            out.push('[');
            for (index, item) in sequence.items.iter().enumerate() {
                if !item.leading_trivia.is_empty() || item.trailing_comment.is_some() {
                    return None;
                }
                let value = item.value?;
                let value_node = ast.node(value);
                if yaml_node_should_preserve(source, document, value_node) {
                    return None;
                }
                if index > 0 {
                    out.push_str(", ");
                }
                emit_compact_yaml_value(out, source, document, ast, value)?;
            }
            out.push(']');
        }
        _ => return None,
    }
    Some(())
}

fn emit_compact_yaml_value(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    id: YamlNodeId,
) -> Option<()> {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
            emit_compact_yaml_node(out, source, document, ast, id)
        }
        _ => emit_yaml_inline_node_into_for_flow(out, source, document, ast, id),
    }
}

fn flow_table_sequence_renderable(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    sequence: &YamlSequence<'_>,
    compact_table: bool,
) -> Option<()> {
    let mut has_row = false;
    for item in &sequence.items {
        let value = item.value?;
        let value_node = ast.node(value);
        if yaml_node_should_preserve(source, document, value_node) {
            return None;
        }
        if flow_table_row_field_count(source, document, ast, value_node, compact_table)? == 0 {
            return None;
        }
        has_row = true;
    }
    has_row.then_some(())
}

#[allow(clippy::too_many_arguments)]
fn emit_flow_table_sequence_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    sequence: &YamlSequence<'_>,
    indent: usize,
    compact_table: bool,
    node_leading: &[YamlTrivia<'_>],
) -> Option<()> {
    let start_len = out.len();
    if emit_flow_table_sequence_inner(
        out,
        source,
        document,
        ast,
        sequence,
        indent,
        compact_table,
        node_leading,
    )
    .is_some()
    {
        Some(())
    } else {
        out.truncate(start_len);
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_flow_table_sequence_inner(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    sequence: &YamlSequence<'_>,
    indent: usize,
    compact_table: bool,
    node_leading: &[YamlTrivia<'_>],
) -> Option<()> {
    let mut rows = Vec::new();
    let mut fields = Vec::new();

    for (index, item) in sequence.items.iter().enumerate() {
        if index == 0 {
            if item.leading_trivia.as_ref() != node_leading {
                emit_trivia(out, source, &item.leading_trivia);
            }
        } else if !item.leading_trivia.is_empty() {
            emit_flow_table_rows(out, source, document, ast, &rows, &fields, indent)?;
            rows.clear();
            fields.clear();
            emit_trivia(out, source, &item.leading_trivia);
        }

        let value = item.value?;
        let value_node = ast.node(value);
        if yaml_node_should_preserve(source, document, value_node) {
            return None;
        }
        let field_start = fields.len();
        let field_count = flow_table_row_fields(
            source,
            document,
            ast,
            value_node,
            compact_table,
            &mut fields,
        )?;
        let trailing_comment = match &value_node.kind {
            YamlAstKind::FlowMapping(mapping) => item.trailing_comment.or(mapping.trailing_comment),
            YamlAstKind::Mapping(_) => item.trailing_comment,
            _ => unreachable!("table row fields require a mapping row"),
        };
        if field_count == 0 {
            fields.truncate(field_start);
            return None;
        }
        rows.push(FlowTableRow {
            line: item.line.span(),
            field_start,
            field_count,
            trailing_comment: trailing_comment.map(SourceSpan::span),
        });
    }

    if rows.is_empty() {
        return None;
    }
    emit_flow_table_rows(out, source, document, ast, &rows, &fields, indent)
}

fn flow_table_row_fields(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    value_node: &YamlAstNode<'_>,
    compact_table: bool,
    fields: &mut Vec<FlowTableField>,
) -> Option<usize> {
    let start = fields.len();
    let result = match &value_node.kind {
        YamlAstKind::FlowMapping(mapping) => {
            flow_mapping_table_fields(source, document, ast, mapping, fields)
        }
        YamlAstKind::Mapping(mapping) if compact_table => {
            block_mapping_table_fields(source, document, ast, mapping, fields)
        }
        _ => None,
    };
    if result.is_none() {
        fields.truncate(start);
        return None;
    }
    Some(fields.len() - start)
}

fn flow_table_row_field_count(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    value_node: &YamlAstNode<'_>,
    compact_table: bool,
) -> Option<usize> {
    match &value_node.kind {
        YamlAstKind::FlowMapping(mapping) => {
            flow_mapping_table_field_count(source, document, ast, mapping)
        }
        YamlAstKind::Mapping(mapping) if compact_table => {
            block_mapping_table_field_count(source, document, ast, mapping)
        }
        _ => None,
    }
}

fn flow_mapping_table_field_count(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    mapping: &YamlFlowMapping<'_>,
) -> Option<usize> {
    if !flow_mapping_table_supported(mapping) {
        return None;
    }
    for pair in &mapping.pairs {
        let value = pair.value?;
        flow_table_field_width_for_parts(
            source,
            document,
            ast,
            FlowTableCell::Node(pair.key),
            value,
        )?;
    }
    Some(mapping.pairs.len())
}

fn flow_mapping_table_fields(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    mapping: &YamlFlowMapping<'_>,
    fields: &mut Vec<FlowTableField>,
) -> Option<()> {
    if !flow_mapping_table_supported(mapping) {
        return None;
    }
    for pair in &mapping.pairs {
        let value = pair.value?;
        let key = FlowTableCell::Node(pair.key);
        let width = flow_table_field_width_for_parts(source, document, ast, key, value)?;
        let field = FlowTableField { key, value, width };
        fields.push(field);
    }
    Some(())
}

fn flow_mapping_table_supported(mapping: &YamlFlowMapping<'_>) -> bool {
    mapping.braced
        && !mapping.has_inner_trivia
        && mapping.tag.is_none()
        && mapping.anchor.is_none()
        && mapping.pairs.iter().all(|pair| !pair.explicit)
}

fn block_mapping_table_field_count(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    mapping: &YamlMapping<'_>,
) -> Option<usize> {
    if !block_mapping_table_supported(mapping) {
        return None;
    }
    for pair in &mapping.pairs {
        if !pair.leading_trivia.is_empty() || pair.trailing_comment.is_some() {
            return None;
        }
        let value = pair.value?;
        let value_node = ast.node(value);
        if yaml_node_should_preserve(source, document, value_node) {
            return None;
        }
        flow_table_field_width_for_parts(
            source,
            document,
            ast,
            FlowTableCell::MappingKey {
                key: pair.key.span(),
                key_node: pair.key_node,
            },
            value,
        )?;
    }
    Some(mapping.pairs.len())
}

fn block_mapping_table_fields(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    mapping: &YamlMapping<'_>,
    fields: &mut Vec<FlowTableField>,
) -> Option<()> {
    if !block_mapping_table_supported(mapping) {
        return None;
    }
    for pair in &mapping.pairs {
        if !pair.leading_trivia.is_empty() || pair.trailing_comment.is_some() {
            return None;
        }
        let value = pair.value?;
        let value_node = ast.node(value);
        if yaml_node_should_preserve(source, document, value_node) {
            return None;
        }
        let key = FlowTableCell::MappingKey {
            key: pair.key.span(),
            key_node: pair.key_node,
        };
        let width = flow_table_field_width_for_parts(source, document, ast, key, value)?;
        let field = FlowTableField { key, value, width };
        fields.push(field);
    }
    Some(())
}

fn block_mapping_table_supported(mapping: &YamlMapping<'_>) -> bool {
    mapping.tag.is_none()
        && mapping.anchor.is_none()
        && mapping.pairs.iter().all(|pair| !pair.explicit)
}

fn emit_mapping_key_for_flow(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    pair: &YamlMappingPair<'_>,
) -> Option<()> {
    emit_mapping_key_for_flow_parts(out, source, document, ast, pair.key.span(), pair.key_node)
}

fn emit_mapping_key_for_flow_parts(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    key: Span,
    key_node: Option<YamlNodeId>,
) -> Option<()> {
    if let Some(key) = key_node {
        return emit_yaml_inline_node_into_for_flow(out, source, document, ast, key);
    }
    let key = source.slice(key).trim();
    if flow_plain_scalar_safe(key) {
        out.push_str(key);
    } else {
        out.push_str(&quote_yaml_single_for_flow(key)?);
    }
    Some(())
}

fn mapping_key_width_for_flow(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    key: Span,
    key_node: Option<YamlNodeId>,
) -> Option<usize> {
    if let Some(key) = key_node {
        return render_yaml_inline_node_width_for_flow(source, document, ast, key);
    }
    let key = source.slice(key).trim();
    if flow_plain_scalar_safe(key) {
        Some(key.chars().count())
    } else {
        Some(quote_yaml_single_for_flow_width(key))
    }
}

fn render_mapping_key_width_for_flow(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    pair: &YamlMappingPair<'_>,
) -> Option<usize> {
    mapping_key_width_for_flow(source, document, ast, pair.key.span(), pair.key_node)
}

#[derive(Clone, Copy)]
enum FlowTableCell {
    Node(YamlNodeId),
    MappingKey {
        key: Span,
        key_node: Option<YamlNodeId>,
    },
}

#[derive(Clone, Copy)]
struct FlowTableField {
    key: FlowTableCell,
    value: YamlNodeId,
    width: usize,
}

struct FlowTableRow {
    line: Span,
    field_start: usize,
    field_count: usize,
    trailing_comment: Option<Span>,
}

struct FlowTableEmitContext<'a> {
    source: &'a SourceBuffer,
    document: &'a Document<'a>,
    ast: &'a YamlDocumentAst<'a>,
    fields: &'a [FlowTableField],
    field_widths: &'a [usize],
    indent: usize,
    align_closing_brace: bool,
}

fn emit_flow_table_rows(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    rows: &[FlowTableRow],
    fields: &[FlowTableField],
    indent: usize,
) -> Option<()> {
    let align_closing_brace = rows.iter().any(|row| row.trailing_comment.is_some());
    let field_widths = flow_table_field_widths(rows, fields, align_closing_brace);
    let body_widths = rows
        .iter()
        .map(|row| {
            flow_table_row_body_width(row, fields, &field_widths, indent, align_closing_brace)
        })
        .collect::<Vec<_>>();
    let comment_column = rows
        .iter()
        .zip(&body_widths)
        .filter_map(|(row, body_width)| row.trailing_comment.map(|_| *body_width))
        .max();
    let context = FlowTableEmitContext {
        source,
        document,
        ast,
        fields,
        field_widths: &field_widths,
        indent,
        align_closing_brace,
    };
    for (row, body_width) in rows.iter().zip(body_widths) {
        emit_flow_table_row_body(out, &context, row)?;
        if let Some(comment) = row.trailing_comment {
            let column = comment_column.unwrap_or(body_width);
            out.push_str(&" ".repeat(column.saturating_sub(body_width) + 1));
            out.push_str(source.slice(comment).trim_end());
        }
        out.push_str(line_ending_for_span(source, row.line));
    }
    Some(())
}

fn emit_flow_table_row_body(
    out: &mut String,
    context: &FlowTableEmitContext<'_>,
    row: &FlowTableRow,
) -> Option<()> {
    out.push_str(&" ".repeat(context.indent));
    out.push_str("- {");
    let row_fields = flow_table_row_field_slice(row, context.fields);
    for (index, field) in row_fields.iter().enumerate() {
        emit_flow_table_cell(
            out,
            context.source,
            context.document,
            context.ast,
            field.key,
        )?;
        out.push_str(": ");
        emit_yaml_inline_node_into_for_flow(
            out,
            context.source,
            context.document,
            context.ast,
            field.value,
        )?;
        let rendered = field.width;
        if index + 1 < row_fields.len() {
            out.push(',');
            let width = context.field_widths.get(index).copied().unwrap_or(rendered);
            out.push_str(&" ".repeat(width.saturating_sub(rendered) + 1));
        } else if context.align_closing_brace {
            let width = context.field_widths.get(index).copied().unwrap_or(rendered);
            out.push_str(&" ".repeat(width.saturating_sub(rendered)));
        }
    }
    out.push('}');
    Some(())
}

fn flow_table_row_body_width(
    row: &FlowTableRow,
    fields: &[FlowTableField],
    field_widths: &[usize],
    indent: usize,
    align_closing_brace: bool,
) -> usize {
    let mut width = indent + 3;
    let row_fields = flow_table_row_field_slice(row, fields);
    for (index, field) in row_fields.iter().enumerate() {
        let rendered = field.width;
        width += rendered;
        if index + 1 < row_fields.len() {
            width += 1;
            let target = field_widths.get(index).copied().unwrap_or(rendered);
            width += target.saturating_sub(rendered) + 1;
        } else if align_closing_brace {
            let target = field_widths.get(index).copied().unwrap_or(rendered);
            width += target.saturating_sub(rendered);
        }
    }
    width + 1
}

fn flow_table_field_widths(
    rows: &[FlowTableRow],
    fields: &[FlowTableField],
    include_last_field: bool,
) -> Vec<usize> {
    let field_count = rows
        .iter()
        .map(|row| {
            if include_last_field {
                row.field_count
            } else {
                row.field_count.saturating_sub(1)
            }
        })
        .max()
        .unwrap_or(0);
    let mut widths = vec![0; field_count];
    for row in rows {
        for (index, field) in flow_table_row_field_slice(row, fields)
            .iter()
            .enumerate()
            .take(field_count)
        {
            if let Some(slot) = widths.get_mut(index) {
                *slot = (*slot).max(field.width);
            }
        }
    }
    widths
}

fn flow_table_row_field_slice<'a>(
    row: &FlowTableRow,
    fields: &'a [FlowTableField],
) -> &'a [FlowTableField] {
    let end = row.field_start + row.field_count;
    &fields[row.field_start..end]
}

fn flow_table_field_width_for_parts(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    key: FlowTableCell,
    value: YamlNodeId,
) -> Option<usize> {
    let key_width = flow_table_cell_width(source, document, ast, key)?;
    let value_width = render_yaml_inline_node_width_for_flow(source, document, ast, value)?;
    Some(key_width + 2 + value_width)
}

fn flow_table_cell_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    cell: FlowTableCell,
) -> Option<usize> {
    match cell {
        FlowTableCell::Node(id) => {
            render_yaml_inline_node_width_for_flow(source, document, ast, id)
        }
        FlowTableCell::MappingKey { key, key_node } => {
            mapping_key_width_for_flow(source, document, ast, key, key_node)
        }
    }
}

fn emit_flow_table_cell(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst<'_>,
    cell: FlowTableCell,
) -> Option<()> {
    match cell {
        FlowTableCell::Node(id) => {
            emit_yaml_inline_node_into_for_flow(out, source, document, ast, id)
        }
        FlowTableCell::MappingKey { key, key_node } => {
            emit_mapping_key_for_flow_parts(out, source, document, ast, key, key_node)
        }
    }
}

fn emit_trivia(out: &mut String, source: &SourceBuffer, trivia: &[YamlTrivia<'_>]) {
    for item in trivia {
        let _ = item.kind;
        out.push_str(source.slice(item.span));
    }
}

#[derive(Debug, Clone, Copy)]
struct LineValue {
    value: Span,
    trailing_comment: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
struct ScalarMetadata {
    tag: Option<Span>,
    anchor: Option<Span>,
    content: Span,
}

fn scalar_content_flow_collection_target(
    source: &SourceBuffer,
    content: Span,
) -> Option<DirectiveTargetKind> {
    match source.slice(content).trim_start().as_bytes().first() {
        Some(b'[') => Some(DirectiveTargetKind::YamlFlowSequence),
        Some(b'{') => Some(DirectiveTargetKind::YamlFlowMapping),
        _ => None,
    }
}

fn flow_collection_directive_target(
    source: &SourceBuffer,
    collection: Span,
) -> DirectiveTargetKind {
    scalar_content_flow_collection_target(source, collection)
        .expect("flow collection starts with a flow collection indicator")
}

#[derive(Debug, Clone, Copy)]
enum PlainScalarContinuation {
    None,
    Inline { parent_indent: usize },
    Block { indent: usize },
}

#[derive(Debug, Clone, Copy)]
struct PlainScalarExtension {
    span_end: usize,
    value_end: usize,
}

fn plain_scalar_continuation_line(text: &str, continuation: PlainScalarContinuation) -> bool {
    if text.trim_start().starts_with('#') || document_marker(text) || standard_yaml_directive(text)
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

fn split_line_value_comment(text: &str, value_start: usize, line_start: usize) -> LineValue {
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

fn find_trailing_comment(text: &str, value_start: usize) -> Option<usize> {
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

fn comment_can_start(text: &str, value_start: usize, index: usize) -> bool {
    index == value_start
        || text[..index]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
}

fn flow_colon_is_value_indicator(text: &str, colon: usize, adjacent_allowed: bool) -> bool {
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

fn scalar_metadata(source: &SourceBuffer, value: impl Into<Span>) -> ScalarMetadata {
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

fn trim_span_ascii(source: &SourceBuffer, span: Span) -> Span {
    let (start, end) = trim_ascii_bounds(source.slice(span));
    Span::new(span.start + start, span.start + end)
}

fn indentation(text: &str) -> usize {
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    bom + text[bom..].bytes().take_while(|byte| *byte == b' ').count()
}

fn tab_in_indentation(text: &str) -> Option<usize> {
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

fn tab_indented_block_at(source: &SourceBuffer, line: usize, end: usize) -> Span {
    let start = source.lines[line].full.start();
    let first_text = source.line_text(line);
    let base_indent = tab_in_indentation(first_text).unwrap_or_else(|| indentation(first_text));
    let mut scan = line + 1;
    while scan < end {
        let text = source.line_text(scan);
        if text.trim().is_empty()
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

fn mapping_colon_at(text: &str, indent: usize) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    let trimmed = text[indent..].trim_end();
    if trimmed.starts_with('-') {
        return None;
    }
    mapping_colon_from(text, indent)
}

fn mapping_colon_from(text: &str, key_start_column: usize) -> Option<usize> {
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

fn explicit_key_line(text: &str, indent: usize) -> Option<usize> {
    explicit_indicator_line(text, indent, b'?')
}

fn explicit_value_line(text: &str, indent: usize) -> Option<usize> {
    explicit_indicator_line(text, indent, b':')
}

fn explicit_indicator_line(text: &str, indent: usize, indicator: u8) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    explicit_indicator_at(text, indent, indicator)
}

fn explicit_indicator_at(text: &str, marker: usize, indicator: u8) -> Option<usize> {
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

fn sequence_line(text: &str, indent: usize) -> Option<usize> {
    if indentation(text) != indent {
        return None;
    }
    compact_sequence_marker_at(text, indent)
}

fn compact_sequence_marker_at(text: &str, marker: usize) -> Option<usize> {
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

fn scalar_style(value: &str) -> YamlScalarStyle {
    if value.starts_with('\'') {
        YamlScalarStyle::SingleQuoted
    } else if value.starts_with('"') {
        YamlScalarStyle::DoubleQuoted
    } else {
        YamlScalarStyle::Plain
    }
}

fn alias_scalar(value: &str) -> bool {
    let value = value.trim();
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

fn block_scalar_style(header: &str) -> YamlScalarStyle {
    if header.contains('|') {
        YamlScalarStyle::LiteralBlock
    } else {
        YamlScalarStyle::FoldedBlock
    }
}

fn scalar_semantic_with_tag(value: &str, tag: Option<&str>) -> YamlScalarSemantic {
    match tag {
        Some("!!str") => YamlScalarSemantic::String,
        Some("!!bool") => scalar_value_for_core_tag(value)
            .filter(|value| yaml_bool_value(value).is_some())
            .map(|_| YamlScalarSemantic::Boolean)
            .unwrap_or(YamlScalarSemantic::Unknown),
        Some("!!null") => scalar_value_for_core_tag(value)
            .filter(|value| yaml_null_value(value))
            .map(|_| YamlScalarSemantic::Null)
            .unwrap_or(YamlScalarSemantic::Unknown),
        Some("!!int") => scalar_value_for_core_tag(value)
            .filter(|value| yaml_integer_value(value))
            .map(|_| YamlScalarSemantic::Integer)
            .unwrap_or(YamlScalarSemantic::Unknown),
        Some("!!float") => scalar_value_for_core_tag(value)
            .filter(|value| yaml_float_value(value))
            .map(|_| YamlScalarSemantic::Float)
            .unwrap_or(YamlScalarSemantic::Unknown),
        Some(_) => YamlScalarSemantic::Unknown,
        None => scalar_semantic(value),
    }
}

fn scalar_semantic_with_tag_and_style(
    value: &str,
    tag: Option<&str>,
    style: YamlScalarStyle,
) -> YamlScalarSemantic {
    if tag.is_some() {
        scalar_semantic_with_tag(value, tag)
    } else {
        scalar_semantic_for_style(value.trim(), style)
    }
}

fn empty_scalar_semantic_with_tag(tag: Option<&str>) -> YamlScalarSemantic {
    match tag {
        Some("!!str") => YamlScalarSemantic::String,
        Some("!!null") | None => YamlScalarSemantic::Null,
        Some(_) => YamlScalarSemantic::Unknown,
    }
}

fn scalar_value_for_core_tag(value: &str) -> Option<String> {
    let trimmed = value.trim();
    match scalar_style(trimmed) {
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            decode_quoted_scalar(trimmed)
        }
        YamlScalarStyle::Plain => Some(trimmed.to_owned()),
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => None,
    }
}

fn scalar_semantic(value: &str) -> YamlScalarSemantic {
    let trimmed = value.trim();
    scalar_semantic_for_style(trimmed, scalar_style(trimmed))
}

fn scalar_semantic_for_style(value: &str, style: YamlScalarStyle) -> YamlScalarSemantic {
    match style {
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => YamlScalarSemantic::String,
        YamlScalarStyle::Plain => {
            non_string_core_semantic_bytes(value.as_bytes()).unwrap_or(YamlScalarSemantic::String)
        }
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => YamlScalarSemantic::Unknown,
    }
}

fn yaml_null_value(value: &str) -> bool {
    matches!(value, "~" | "null" | "Null" | "NULL")
}

fn yaml_bool_value(value: &str) -> Option<bool> {
    match value {
        "true" | "True" | "TRUE" => Some(true),
        "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

fn yaml_integer_value(value: &str) -> bool {
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    yaml_decimal_digits(digits)
}

fn yaml_float_value(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
        return true;
    }
    if matches!(value, ".nan" | ".NaN" | ".NAN") {
        return true;
    }

    if !yaml_float_spelling(value) {
        return false;
    }
    if value.contains('_') {
        return value.replace('_', "").parse::<f64>().is_ok();
    }
    value.parse::<f64>().is_ok()
}

fn yaml_float_spelling(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if unsigned.is_empty() {
        return false;
    }
    if yaml_decimal_digits(unsigned) {
        return true;
    }
    let Some(number_end) = unsigned.find(['e', 'E']) else {
        return yaml_decimal_fraction(unsigned);
    };
    let (number, exponent) = unsigned.split_at(number_end);
    let exponent = &exponent[1..];
    yaml_float_number_part(number)
        && yaml_decimal_digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent))
}

fn yaml_float_number_part(value: &str) -> bool {
    yaml_decimal_digits(value) || yaml_decimal_fraction(value)
}

fn yaml_decimal_fraction(value: &str) -> bool {
    let Some((before, after)) = value.split_once('.') else {
        return false;
    };
    (before.is_empty() || yaml_decimal_digits(before))
        && (after.is_empty() || yaml_decimal_digits(after))
        && !(before.is_empty() && after.is_empty())
}

fn yaml_decimal_digits(value: &str) -> bool {
    let mut saw_digit = false;
    let mut previous_underscore = false;
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            previous_underscore = false;
        } else if ch == '_' && saw_digit && !previous_underscore {
            previous_underscore = true;
        } else {
            return false;
        }
    }
    saw_digit && !previous_underscore
}

fn document_marker(text: &str) -> bool {
    document_marker_kind(text).is_some()
}

#[derive(Debug, Clone, Copy)]
struct DocumentMarkerLine {
    kind: DocumentMarkerKind,
    inline_content_start: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentMarkerKind {
    Start,
    End,
}

fn document_marker_kind(text: &str) -> Option<DocumentMarkerKind> {
    document_marker_line_info(text).map(|line| line.kind)
}

fn document_marker_line_info(text: &str) -> Option<DocumentMarkerLine> {
    let (body, _) = strip_newline(text);
    let marker_start = body.len() - body.trim_start().len();
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
    let content = content[..comment_start].trim();
    !content.is_empty()
        && content
            .split_whitespace()
            .all(|token| token.starts_with('!') || token.starts_with('&'))
}

fn standard_yaml_directive(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with("%YAML ") || trimmed.starts_with("%TAG ")
}

fn line_ending_for_span(source: &SourceBuffer, span: impl Into<Span>) -> &'static str {
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

fn preserves_yaml_template_span(
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
    let trimmed = body[indent_len..].trim_end();

    if let Some(rest) = trimmed.strip_prefix("-") {
        if rest.is_empty() {
            return format!("{indent}-{newline}");
        }
        if rest.chars().next().is_some_and(char::is_whitespace) {
            return format!(
                "{indent}- {}{newline}",
                normalize_flow_value(rest.trim_start())
            );
        }
    }

    if let Some(colon) = simple_mapping_colon(trimmed) {
        let key = trimmed[..colon].trim_end();
        let value = normalize_flow_value(trimmed[colon + 1..].trim_start());
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
        let trimmed = body[indent_len..].trim();
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

fn simple_mapping_colon(text: &str) -> Option<usize> {
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

fn strip_newline(line: &str) -> (&str, &str) {
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

#[derive(Debug, Clone, Copy)]
struct QuotedScalarBlock {
    full: Span,
    value: Span,
    trailing_comment: Option<Span>,
}

fn quoted_scalar_block_from_value(
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

fn quoted_scalar_close(text: &str, start: usize, quote: u8) -> Option<usize> {
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
struct BlockScalar {
    full: Span,
    header: Span,
    header_info: YamlBlockScalarHeader,
    value: Span,
    body: Span,
    trailing_comment: Option<Span>,
}

fn block_scalar_at(source: &SourceBuffer, line: usize, end: usize) -> Option<BlockScalar> {
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
    let header_info = block_scalar_header_info(&trimmed[marker_index + 1..])?;
    let value_start = block_scalar_value_start(text, marker_index)?;
    if !block_scalar_metadata_prefix(&text[value_start..marker_index]) {
        return None;
    }
    let base_indent = text.bytes().take_while(|byte| *byte == b' ').count();
    let mut i = line + 1;
    while i < end {
        let candidate = source.line_text(i);
        if candidate.trim().is_empty() {
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
    for ch in indicators.trim().chars() {
        match ch {
            '+' if chomp == YamlBlockChomp::Clip => chomp = YamlBlockChomp::Keep,
            '-' if chomp == YamlBlockChomp::Clip => chomp = YamlBlockChomp::Strip,
            '1'..='9' if indent.is_none() => indent = ch.to_digit(10).map(|value| value as u8),
            '0' => return None,
            _ => return None,
        }
    }
    Some(YamlBlockScalarHeader { indent, chomp })
}

fn block_scalar_value_start(text: &str, marker_index: usize) -> Option<usize> {
    let indent = indentation(text);
    if let Some(colon) = mapping_colon_at(text, indent)
        && colon < marker_index
    {
        let mut value_start = colon + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some(value_start);
    }
    if let Some(marker) = sequence_line(text, indent)
        && marker < marker_index
    {
        let mut value_start = marker + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some(value_start);
    }
    if let Some(marker) = explicit_key_line(text, indent)
        && marker < marker_index
    {
        let mut value_start = marker + 1;
        while value_start < marker_index && text.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }
        return Some(value_start);
    }
    None
}

fn block_scalar_metadata_prefix(prefix: &str) -> bool {
    let prefix = prefix.trim();
    prefix.is_empty()
        || prefix
            .split_whitespace()
            .all(|token| token.starts_with('!') || token.starts_with('&'))
}

#[derive(Debug, Clone, Copy)]
struct FlowCollectionBlock {
    span: Span,
    value: Span,
    collection: Span,
    trailing_comment: Option<Span>,
}

#[derive(Debug, Clone, Copy)]
struct FlowCollapseHint {
    opener: Span,
    rest: FlowCollapseRest,
}

#[derive(Debug, Clone, Copy)]
enum FlowCollapseRest {
    Empty,
    InlineMapping { key_start: usize },
}

#[derive(Debug, Clone, Copy)]
enum FlowCollectionScan {
    Complete(FlowCollectionBlock),
    Incomplete,
}

impl FlowCollectionScan {
    fn complete(self) -> Option<FlowCollectionBlock> {
        match self {
            Self::Complete(block) => Some(block),
            Self::Incomplete => None,
        }
    }
}

fn flow_collapse_hint_from_value(
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

fn line_has_flow_collapse_hint_value(source: &SourceBuffer, line: usize, end: usize) -> bool {
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

fn flow_collapse_hint_from_standalone_opener(
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
    if text.trim().is_empty()
        || text.trim_start().starts_with('#')
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

fn flow_collection_block_from_value(
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

fn line_flow_value_start(text: &str) -> Option<usize> {
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

fn skip_ascii_whitespace(text: &str, mut cursor: usize) -> usize {
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

fn unsupported_flow_block_at(source: &SourceBuffer, line: usize, end: usize) -> Option<Span> {
    let text = source.line_text(line);
    if let Some(value_start) = line_flow_value_start(text)
        && let Some(FlowCollectionScan::Complete(block)) = flow_collection_block_from_value(
            source,
            line,
            end,
            value_start,
            source.lines[line].full.start(),
        )
        && flow_collection_parseable(source, block.value)
    {
        return None;
    }

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

fn yaml_error_at(source: &SourceBuffer, byte: usize, message: impl Into<String>) -> YamarkError {
    let (line, column) = source.line_column_at_byte(byte);
    YamarkError::at(message, line, column)
}

fn parse_flow_mapping_row(trimmed: &str) -> Option<Vec<(String, String)>> {
    let inner = trimmed.strip_prefix("- {")?.strip_suffix('}')?;
    let mut fields = Vec::new();
    for part in inner.split(',') {
        let (key, value) = part.split_once(':')?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return None;
        }
        fields.push((key.to_owned(), value.to_owned()));
    }
    Some(fields)
}

fn format_markdown_yaml_block_value(value: &str) -> String {
    replace_folded_block_marker(value.trim())
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
    let trimmed = value.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let values = inner.split(',').map(|item| item.trim()).collect::<Vec<_>>();
        return format!("[{}]", values.join(", "));
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let values = inner
            .split(',')
            .map(|item| {
                if let Some((key, value)) = item.split_once(':') {
                    format!("{}: {}", key.trim(), value.trim())
                } else {
                    item.trim().to_owned()
                }
            })
            .collect::<Vec<_>>();
        return format!("{{{}}}", values.join(", "));
    }
    trimmed.to_owned()
}
