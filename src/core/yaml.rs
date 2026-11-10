//! YAML parsing and emission entry points and the shared block-parser state.

mod block_scalar;
mod directives;
mod emit;
mod flow_emit;
mod flow_parse;
mod parse;
mod plan;
mod scalar;
mod simple;
mod syntax;
mod table;

use crate::config::Config;
use crate::core::directives::{
    DirectiveDelta, DirectiveEngine, StateId, directive_delta_affects_markdown,
};
use crate::core::document::{
    Document, DocumentKind, EmitPlan, FormatOptions, Node, NodeKind, YamlNodeKind,
};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlAstNode, YamlDocumentAst, YamlNodeId, YamlRoot, YamlTrivia, YamlTriviaKind,
};
use crate::core::yaml_scan::{YamlLineScan, scan_yaml_lines, scan_yaml_lines_basic};
use crate::diagnostic::Result;
pub(crate) use block_scalar::{InlineMarkdownPlan, resolve_inline_markdown_plan};
pub use emit::YamlEmissionStats;
pub(crate) use emit::emit_planned_yaml_document_with_stats;
use flow_emit::source_contains_any_template_opener;
use flow_parse::RecognizedFlowCollection;
use plan::YamlPlanRefresh;
pub use simple::{format_flow_table, format_simple_yaml_line};
use std::cell::RefCell;
use syntax::{
    DocumentMarkerKind, document_marker_kind, yaml_error_at, yaml_scan_has_active_fmt_directive,
    yaml_scan_has_tab_indentation, yaml_scan_has_unsupported_preserve_syntax,
};

pub(crate) fn parse_yaml(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document> {
    parse_yaml_impl(source, range, options, config, YamlParseMode::CONCRETE)
}

pub(crate) fn parse_yaml_for_formatting(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document> {
    parse_yaml_impl(source, range, options, config, YamlParseMode::FORMATTING)
}

pub(crate) fn parse_yaml_for_formatting_with_trace(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document> {
    parse_yaml_impl(
        source,
        range,
        options,
        config,
        YamlParseMode::FORMATTING_WITH_TRACE,
    )
}

pub(crate) fn parse_yaml_for_validation(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
    node_capacity_hint: usize,
) -> Result<Document> {
    parse_yaml_impl(
        source,
        range,
        options,
        config,
        YamlParseMode::validation(true, node_capacity_hint),
    )
}

pub(crate) fn parse_yaml_for_concrete_validation(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document> {
    parse_yaml_impl(
        source,
        range,
        options,
        config,
        YamlParseMode::validation(false, 0),
    )
}

#[derive(Debug, Clone, Copy)]
struct YamlParseMode {
    collect_trace: bool,
    preserve_unsupported_for_formatting: bool,
    plan_emits: bool,
    node_capacity_hint: usize,
}

impl YamlParseMode {
    const CONCRETE: Self = Self {
        collect_trace: true,
        preserve_unsupported_for_formatting: false,
        plan_emits: true,
        node_capacity_hint: 0,
    };
    const FORMATTING: Self = Self {
        collect_trace: false,
        preserve_unsupported_for_formatting: true,
        plan_emits: true,
        node_capacity_hint: 0,
    };
    const FORMATTING_WITH_TRACE: Self = Self {
        collect_trace: true,
        ..Self::FORMATTING
    };

    const fn validation(
        preserve_unsupported_for_formatting: bool,
        node_capacity_hint: usize,
    ) -> Self {
        Self {
            collect_trace: false,
            preserve_unsupported_for_formatting,
            plan_emits: false,
            node_capacity_hint,
        }
    }
}

fn parse_yaml_impl(
    source: &SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
    mode: YamlParseMode,
) -> Result<Document> {
    crate::core::parser::validate_compact_source_range(range)?;
    let mut options = options;
    if !matches!(
        source.dominant_line_ending,
        crate::core::source::LineEnding::None
    ) {
        options.default_line_ending = source.dominant_line_ending.as_str();
    }
    let mut scan = scan_yaml_lines_basic(source, range);
    if mode.preserve_unsupported_for_formatting
        && scan.has_tab_indentation(source).is_some()
        && yaml_scan_has_tab_indentation(source, &scan)
    {
        scan = scan_yaml_lines(source, range);
        scan.source_scans += 1;
        if !yaml_scan_has_active_fmt_directive(source, &scan) {
            return Ok(preserved_yaml_document(range, options, &scan));
        }
    }
    if mode.preserve_unsupported_for_formatting
        && yaml_scan_has_unsupported_preserve_syntax(source, &scan)
        && !yaml_scan_has_active_fmt_directive(source, &scan)
    {
        return Ok(preserved_yaml_document(range, options, &scan));
    }
    let parser = YamlParser::new(source, range, options, config, scan, mode);
    let mut doc = parser.parse()?;
    doc.push_node(Node {
        kind: NodeKind::Yaml(YamlNodeKind::Document),
        span: range,
        state: StateId(0),
        emit: EmitPlan::YamlDocument,
    });
    Ok(doc)
}

fn preserved_yaml_document(range: Span, options: FormatOptions, scan: &YamlLineScan) -> Document {
    let mut doc = Document::new(DocumentKind::Yaml, range);
    doc.options = options;
    doc.trace.source_scans = scan.source_scans;
    doc.trace.parse_passes = 1;
    doc.trace.yaml_scanned_lines = scan.scanned_lines;
    doc.skip_file = true;
    doc
}

pub(crate) fn apply_file_scope_delta_to_yaml_document(
    source: &SourceBuffer,
    document: &mut Document,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    apply_file_scope_delta_to_yaml_document_with_mode(
        source, document, delta, options, config, true,
    )
}

pub(crate) fn apply_file_scope_delta_to_yaml_document_for_validation(
    source: &SourceBuffer,
    document: &mut Document,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    apply_file_scope_delta_to_yaml_document_with_mode(
        source, document, delta, options, config, false,
    )
}

fn apply_file_scope_delta_to_yaml_document_with_mode(
    source: &SourceBuffer,
    document: &mut Document,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
    plan_emits: bool,
) -> Result<()> {
    let owned_source = document.source.take();
    let placeholder = Document::new(DocumentKind::Yaml, document.range);
    let doc = std::mem::replace(document, placeholder);
    let (mut doc, result) = replan_yaml_document_with_delta(
        owned_source.as_ref().unwrap_or(source),
        doc,
        delta,
        options,
        config,
        plan_emits,
    );
    doc.source = owned_source;
    *document = doc;
    result
}

fn replan_yaml_document_with_delta(
    plan_source: &SourceBuffer,
    mut doc: Document,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
    plan_emits: bool,
) -> (Document, Result<()>) {
    doc.patch_all_states(delta.clone());
    let Some(mut ast) = doc.yaml.take() else {
        return (doc, Ok(()));
    };
    let template_presence = YamlTemplatePresence::new(plan_source, config);
    directives::patch_existing_ast_states(&mut doc, &mut ast, delta);
    let result = if directive_delta_affects_markdown(delta) {
        directives::patch_nested_markdown_documents(
            plan_source,
            &mut doc,
            &ast,
            delta,
            options,
            config,
            plan_emits,
        )
    } else {
        Ok(())
    };
    if result.is_ok() && plan_emits {
        plan::YamlPlanner::new(
            plan_source,
            options,
            config,
            &mut doc,
            &mut ast,
            &template_presence,
        )
        .plan_yaml_emits(YamlPlanRefresh::for_delta(delta));
    }
    if result.is_ok() {
        doc.trace.yaml_semantic_nodes = ast.nodes.len();
    }
    doc.yaml = Some(ast);
    (doc, result)
}

/// Conservative template-presence facts shared by recognition and planning.
/// State IDs refer to interned states, so reusing a fact does not depend on an
/// emission plan. The cache stays with the same source and base configuration.
struct YamlTemplatePresence {
    default_openers_present: bool,
    by_state: RefCell<Vec<Option<bool>>>,
}

impl YamlTemplatePresence {
    fn new(source: &SourceBuffer, config: &Config) -> Self {
        Self {
            default_openers_present: source_contains_any_template_opener(
                source.as_str(),
                &config.template_delimiters,
            ),
            by_state: RefCell::new(Vec::new()),
        }
    }

    fn spans_possible_for_state(&self, doc: &Document, config: &Config, state: StateId) -> bool {
        if self.default_openers_present {
            return true;
        }
        let mut cache = self.by_state.borrow_mut();
        let state = state.index();
        if state >= cache.len() {
            cache.resize(state + 1, None);
        }
        if let Some(template_spans_possible) = cache[state] {
            return template_spans_possible;
        }
        let template_spans_possible =
            doc.state(StateId::new(state)).template_delimiters != config.template_delimiters;
        cache[state] = Some(template_spans_possible);
        template_spans_possible
    }
}

struct YamlParser<'src, 'cfg> {
    source: &'src SourceBuffer,
    options: FormatOptions,
    config: &'cfg Config,
    doc: Document,
    ast: YamlDocumentAst,
    engine: DirectiveEngine,
    file_scope_delta: DirectiveDelta,
    start: usize,
    line: usize,
    end: usize,
    held_trivia: Vec<YamlTrivia>,
    flow_collection_nodes: Vec<YamlNodeId>,
    recognized_flow: Option<RecognizedFlowCollection>,
    template_presence: YamlTemplatePresence,
    collect_trace: bool,
    plan_emits: bool,
}

impl<'src, 'cfg> YamlParser<'src, 'cfg> {
    fn new(
        source: &'src SourceBuffer,
        range: Span,
        options: FormatOptions,
        config: &'cfg Config,
        scan: YamlLineScan,
        mode: YamlParseMode,
    ) -> Self {
        let source_scans = scan.source_scans;
        let yaml_scanned_lines = scan.scanned_lines;
        let start_line = scan.start_line;
        let end_line = scan.end_line;
        let mut doc = Document::new(DocumentKind::Yaml, range);
        let mut ast = YamlDocumentAst::new(range);
        let estimated_nodes = yaml_scanned_lines.saturating_add(yaml_scanned_lines / 16);
        ast.nodes
            .reserve(estimated_nodes.max(mode.node_capacity_hint));
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
            recognized_flow: None,
            template_presence: YamlTemplatePresence::new(source, config),
            collect_trace: mode.collect_trace,
            plan_emits: mode.plan_emits,
        }
    }

    fn parse(mut self) -> Result<Document> {
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
        // The replay applies the file-scoped delta over every node, including
        // one whose state was set by a later `scope=next` or `scope=from-here`
        // directive. In YAML, a file-scoped directive therefore overrides
        // narrower directives for the same option. See `docs/TODO.md`.
        let plan_refresh = if self.file_scope_delta != DirectiveDelta::default() {
            let delta = self.file_scope_delta.clone();
            self.patch_existing_ast_states(&delta);
            if directive_delta_affects_markdown(&delta) {
                self.patch_nested_markdown_documents(&delta)?;
            }
            YamlPlanRefresh::for_delta(&delta)
        } else {
            YamlPlanRefresh::RetainCaches
        };
        self.populate_document_root_markers();
        if self.plan_emits {
            self.plan_yaml_emits(plan_refresh);
            if self.collect_trace {
                self.record_plan_trace();
            }
        }
        self.finish_ast();
        self.doc.yaml = Some(self.ast);
        Ok(self.doc)
    }

    fn finish_ast(&mut self) {
        self.doc.trace.yaml_semantic_nodes = self.ast.nodes.len();
    }

    fn template_spans_possible_for_state(&self, state: StateId) -> bool {
        self.template_presence
            .spans_possible_for_state(&self.doc, self.config, state)
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
        roots: &mut Vec<YamlRoot>,
        pending_start: &mut Option<SourceSpan>,
        marker: SourceSpan,
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
}
