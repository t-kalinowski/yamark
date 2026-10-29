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
use crate::core::source::{SourceBuffer, Span};
use crate::core::yaml_model::{YamlDocumentAst, YamlNodeId, YamlRoot, YamlTrivia};
use crate::core::yaml_scan::{YamlLineScan, scan_yaml_lines, scan_yaml_lines_basic};
use crate::diagnostic::Result;
pub(crate) use block_scalar::{InlineMarkdownPlan, resolve_inline_markdown_plan};
pub use emit::YamlEmissionStats;
pub(crate) use emit::emit_planned_yaml_document_with_stats;
use flow_emit::source_contains_any_template_opener;
use flow_parse::RecognizedFlowCollection;
pub use simple::{format_flow_table, format_simple_yaml_line};
use std::cell::RefCell;
use syntax::{
    yaml_error_at, yaml_scan_has_active_fmt_directive, yaml_scan_has_tab_indentation,
    yaml_scan_has_unsupported_preserve_syntax,
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
        recognized_flow: None,
        default_template_openers_present: source_contains_any_template_opener(
            plan_source.as_str(),
            &config.template_delimiters,
        ),
        template_spans_possible_by_state: RefCell::new(Vec::new()),
        collect_trace: false,
        plan_emits,
    };
    parser.patch_existing_ast_states(delta);
    let result = if directive_delta_affects_markdown(delta) {
        parser.patch_nested_markdown_documents(delta)
    } else {
        Ok(())
    };
    if result.is_ok() && plan_emits {
        parser.plan_yaml_emits(true);
    }
    if result.is_ok() {
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
        recognized_flow: _,
        default_template_openers_present: _,
        template_spans_possible_by_state: _,
        collect_trace: _,
        plan_emits: _,
    } = parser;
    doc.yaml = Some(ast);
    (doc, result)
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
    default_template_openers_present: bool,
    template_spans_possible_by_state: RefCell<Vec<Option<bool>>>,
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
            default_template_openers_present: source_contains_any_template_opener(
                source.as_str(),
                &config.template_delimiters,
            ),
            template_spans_possible_by_state: RefCell::new(Vec::new()),
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
        if self.plan_emits {
            self.plan_yaml_emits(clear_inline_width_cache);
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
}
