//! Retained Markdown plans and their finalization after directive policy settles.
//! Recognition state is released before document emission.

use super::{
    MarkdownBlockFormatKind, markdown_error_at, render_markdown_heading,
    render_markdown_setext_heading,
};
use crate::core::directives::{
    DirectiveState, DirectiveStateTable, StateId, TemplateDelimiter,
    contains_markdown_template_span,
};
use crate::core::document::{Document, EmitPlan, FormatOptions, MarkdownNodeKind, Node, NodeKind};
use crate::core::source::{SourceBuffer, Span};
use crate::core::yaml_model::{YamlAstKind, YamlDocumentAst, YamlEmitPlan, YamlRenderedKind};
use crate::diagnostic::Result;

/// Only formatted nodes own a retained record; other nodes need one small index.
#[derive(Debug, Clone, Default)]
pub(crate) struct MarkdownPlans {
    by_node: Vec<Option<std::num::NonZeroUsize>>,
    // Both vectors share each slot through finalization; emission reads only plans.
    plans: Vec<RetainedMarkdown>,
    planning: Vec<MarkdownPlanning>,
}

impl MarkdownPlans {
    pub(crate) fn block_count(&self) -> usize {
        self.plans.len()
    }

    pub(crate) fn push_node(&mut self) {
        self.by_node.push(None);
    }

    pub(crate) fn get(&self, node: usize) -> Option<&RetainedMarkdown> {
        let id = self.by_node.get(node).copied().flatten()?;
        Some(&self.plans[id.get() - 1])
    }

    pub(crate) fn take_fragment_block(&mut self, node: usize) -> Option<crate::core::wrap::Plan> {
        let retained = self.get_mut(node)?;
        if retained.preserve {
            None
        } else {
            retained.plan.take()
        }
    }

    fn get_mut(&mut self, node: usize) -> Option<&mut RetainedMarkdown> {
        let id = self.by_node.get(node).copied().flatten()?;
        Some(&mut self.plans[id.get() - 1])
    }

    fn get_pair_mut(
        &mut self,
        node: usize,
    ) -> Option<(&mut RetainedMarkdown, &mut MarkdownPlanning)> {
        let id = self.by_node.get(node).copied().flatten()?;
        Some((
            &mut self.plans[id.get() - 1],
            &mut self.planning[id.get() - 1],
        ))
    }

    fn planning(&self, node: usize) -> Option<&MarkdownPlanning> {
        let id = self.by_node.get(node).copied().flatten()?;
        Some(&self.planning[id.get() - 1])
    }

    fn insert(&mut self, node: usize, plan: (RetainedMarkdown, MarkdownPlanning)) {
        if let Some(id) = self.by_node[node] {
            self.plans[id.get() - 1] = plan.0;
            self.planning[id.get() - 1] = plan.1;
        } else {
            if self.plans.is_empty() {
                self.plans.reserve_exact(1);
                self.planning.reserve_exact(1);
            }
            self.plans.push(plan.0);
            self.planning.push(plan.1);
            self.by_node[node] = std::num::NonZeroUsize::new(self.plans.len());
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RetainedMarkdown {
    pub(crate) preserve: bool,
    pub(crate) plan: Option<crate::core::wrap::Plan>,
}

/// Recognition and policy facts are needed only until the last node is finalized.
#[derive(Debug, Clone)]
struct MarkdownPlanning {
    state: StateId,
    preserve_raw: bool,
    options: Option<FormatOptions>,
    draft: Option<crate::core::wrap::Draft>,
    template_spans: Option<crate::core::wrap::ProseTemplateSpans>,
}

fn markdown_block_kind(emit: &EmitPlan) -> Option<MarkdownBlockFormatKind> {
    Some(match emit {
        EmitPlan::MarkdownParagraph => MarkdownBlockFormatKind::Paragraph,
        EmitPlan::MarkdownList => MarkdownBlockFormatKind::List,
        EmitPlan::MarkdownDefinitionList => MarkdownBlockFormatKind::DefinitionList,
        EmitPlan::MarkdownBlockquote => MarkdownBlockFormatKind::Blockquote,
        EmitPlan::MarkdownTable => MarkdownBlockFormatKind::Table,
        EmitPlan::MarkdownPandocTable => MarkdownBlockFormatKind::PandocTable,
        _ => return None,
    })
}

impl MarkdownPlanning {
    fn preserves_templates(
        &mut self,
        input: &str,
        kind: MarkdownBlockFormatKind,
        delimiters: &[TemplateDelimiter],
    ) -> bool {
        if matches!(
            kind,
            MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable
        ) || !contains_markdown_template_span(input, delimiters)
        {
            return false;
        }
        !self
            .template_spans
            .get_or_insert_with(|| {
                crate::core::wrap::ProseTemplateSpans::recognize(input, delimiters)
            })
            .contains_all_templates(input, delimiters)
    }

    fn resolve_plan(
        &mut self,
        retained: &mut RetainedMarkdown,
        input: &str,
        options: FormatOptions,
        delimiters: &[TemplateDelimiter],
    ) {
        retained.plan = self
            .draft
            .as_mut()
            .and_then(|draft| draft.resolve_with_templates(input, options, delimiters));
        self.options = Some(options);
    }

    fn template_policy_changed(&self, node: &Node, doc: &Document) -> bool {
        // Parsing and finalization share this tree's append-only state table.
        // A late directive can add delimiters without changing format options.
        self.state != node.state
            && doc.state(self.state).template_delimiters
                != doc.state(node.state).template_delimiters
    }
}

pub(super) fn push_markdown_format_node(
    source: &SourceBuffer,
    doc: &mut Document,
    state: StateId,
    kind: MarkdownNodeKind,
    span: Span,
    format_kind: MarkdownBlockFormatKind,
    options: FormatOptions,
) -> Result<()> {
    push_structural_markdown_format_node(source, doc, state, kind, span, format_kind, options, true)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_structural_markdown_format_node(
    source: &SourceBuffer,
    doc: &mut Document,
    state: StateId,
    kind: MarkdownNodeKind,
    span: Span,
    format_kind: MarkdownBlockFormatKind,
    options: FormatOptions,
    structurally_supported: bool,
) -> Result<()> {
    let mut node = Node {
        kind: NodeKind::Markdown(kind),
        span,
        state,
        emit: markdown_format_emit_plan(format_kind),
    };
    // Explicit targets are checked here to preserve diagnostic/skip precedence.
    // The successful plan is retained and reused after file directives settle.
    let planned = if structurally_supported && doc.state(state).markdown_target {
        Some(plan_markdown_block(
            source,
            &node,
            doc.state(state),
            options,
            format_kind,
            true,
        ))
    } else {
        None
    };
    let supported = structurally_supported
        && planned.as_ref().is_none_or(|(plan, _)| {
            !plan.preserve
                && (plan.plan.is_some()
                    || matches!(
                        format_kind,
                        MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable
                    ))
        });
    validate_markdown_format_target(source, doc, state, span, supported)?;
    if !supported {
        node.emit = EmitPlan::Copy;
    }
    doc.push_node(node);
    if let Some(planned) = planned {
        doc.markdown.insert(doc.nodes.len() - 1, planned);
    }
    Ok(())
}

fn plan_markdown_block(
    source: &SourceBuffer,
    node: &Node,
    state: &DirectiveState,
    options: FormatOptions,
    kind: MarkdownBlockFormatKind,
    resolve: bool,
) -> (RetainedMarkdown, MarkdownPlanning) {
    let input = source.slice(node.span);
    let options = state.markdown_options(options);
    let preserve_raw =
        crate::core::wrap::markdown_reflow_changes_raw_semantics(input, &state.template_delimiters);
    let mut retained = RetainedMarkdown {
        preserve: preserve_raw,
        plan: None,
    };
    let mut planning = MarkdownPlanning {
        state: node.state,
        preserve_raw,
        options: resolve.then_some(options),
        draft: None,
        template_spans: None,
    };
    retained.preserve =
        preserve_raw || planning.preserves_templates(input, kind, &state.template_delimiters);
    planning.draft = (!retained.preserve)
        .then(|| prepare_markdown_block(input, kind))
        .flatten();
    if resolve {
        planning.resolve_plan(&mut retained, input, options, &state.template_delimiters);
    }
    (retained, planning)
}

fn prepare_markdown_block(
    input: &str,
    kind: MarkdownBlockFormatKind,
) -> Option<crate::core::wrap::Draft> {
    match kind {
        MarkdownBlockFormatKind::Paragraph => crate::core::wrap::prepare_markdown_paragraph(input),
        MarkdownBlockFormatKind::List => crate::core::wrap::prepare_markdown_list(input),
        MarkdownBlockFormatKind::DefinitionList => {
            crate::core::wrap::prepare_markdown_definition_list(input)
        }
        MarkdownBlockFormatKind::Blockquote => {
            crate::core::wrap::prepare_markdown_blockquote(input)
        }
        MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable => None,
    }
}

pub(super) fn retain_markdown_drafts(
    source: &SourceBuffer,
    doc: &mut Document,
    options: FormatOptions,
) {
    if doc.skip_file {
        return;
    }
    for (index, node) in doc.nodes.iter().enumerate() {
        let state = doc.state(node.state);
        if !state.preserve
            && doc.markdown.get(index).is_none()
            && let Some(kind) = markdown_block_kind(&node.emit)
        {
            doc.markdown.insert(
                index,
                plan_markdown_block(source, node, state, options, kind, false),
            );
        }
    }
}

fn finalize_markdown_plans(source: &SourceBuffer, doc: &mut Document, options: FormatOptions) {
    if doc.skip_file {
        return;
    }
    for (index, node) in doc.nodes.iter().enumerate() {
        let state = doc.states.get(node.state);
        if state.preserve || matches!(node.emit, EmitPlan::Preserve) {
            if let Some((_, planning)) = doc.markdown.get_pair_mut(index) {
                planning.draft = None;
                planning.template_spans = None;
            }
            continue;
        }
        if let Some(kind) = markdown_block_kind(&node.emit) {
            let effective = state.markdown_options(options);
            let template_changed = doc
                .markdown
                .planning(index)
                .is_some_and(|planning| planning.template_policy_changed(node, doc));
            if let Some((retained, planning)) = doc.markdown.get_pair_mut(index) {
                let was_preserved = retained.preserve;
                if template_changed {
                    planning.template_spans = None;
                    planning.draft = None;
                    planning.preserve_raw =
                        crate::core::wrap::markdown_reflow_changes_raw_semantics(
                            source.slice(node.span),
                            &state.template_delimiters,
                        );
                    let template = planning.preserves_templates(
                        source.slice(node.span),
                        kind,
                        &state.template_delimiters,
                    );
                    retained.preserve = planning.preserve_raw || template;
                }
                if !retained.preserve && planning.draft.is_none() {
                    planning.draft = prepare_markdown_block(source.slice(node.span), kind);
                }
                if !retained.preserve
                    && (planning.options != Some(effective) || was_preserved || template_changed)
                {
                    planning.resolve_plan(
                        retained,
                        source.slice(node.span),
                        effective,
                        &state.template_delimiters,
                    );
                }
                planning.state = node.state;
            } else {
                doc.markdown.insert(
                    index,
                    plan_markdown_block(source, node, state, options, kind, true),
                );
            }
            if matches!(
                kind,
                MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable
            ) {
                let retained = doc.markdown.get_mut(index).expect("planned table");
                if !retained.preserve {
                    let input = source.slice(node.span);
                    let text = match kind {
                        MarkdownBlockFormatKind::Table => {
                            crate::core::wrap::format_markdown_table(input, effective)
                        }
                        _ => crate::core::wrap::format_markdown_pandoc_table(input, effective),
                    };
                    retained.plan = Some(crate::core::wrap::Plan::normalized_block(input, &text));
                }
            }
        } else if matches!(
            node.emit,
            EmitPlan::MarkdownHeading { .. } | EmitPlan::MarkdownSetextHeading { .. }
        ) {
            let preserve = contains_markdown_template_span(
                source.slice(node.span),
                &state.template_delimiters,
            );
            let options = state.markdown_options(options);
            let plan = if preserve {
                None
            } else {
                let text = match node.emit {
                    EmitPlan::MarkdownHeading { marker, content } => {
                        render_markdown_heading(source, node.span, marker, content, options)
                    }
                    EmitPlan::MarkdownSetextHeading { content, depth } => {
                        render_markdown_setext_heading(source, node.span, content, depth, options)
                    }
                    _ => unreachable!("heading plan"),
                };
                Some(crate::core::wrap::Plan::normalized_block(
                    source.slice(node.span),
                    &text,
                ))
            };
            doc.markdown.insert(
                index,
                (
                    RetainedMarkdown { preserve, plan },
                    MarkdownPlanning {
                        state: node.state,
                        preserve_raw: false,
                        options: Some(options),
                        draft: None,
                        template_spans: None,
                    },
                ),
            );
        }
        // This node's effective policy and execution plan are now settled.
        if let Some((_, planning)) = doc.markdown.get_pair_mut(index) {
            planning.draft = None;
            planning.template_spans = None;
        }
    }
}

/// All node and nested policies are settled; execution needs only the plans.
pub(crate) fn release_planning(document: &mut Document) {
    document.markdown.planning = Vec::new();
    for nested in &mut document.nested {
        release_planning(nested);
    }
}

pub(crate) fn document_emit_options(
    source: &SourceBuffer,
    document: &Document,
    options: FormatOptions,
) -> FormatOptions {
    let mut options = if document.options == FormatOptions::default() {
        options
    } else {
        document.options
    };
    if !matches!(
        source.dominant_line_ending,
        crate::core::source::LineEnding::None
    ) {
        options.default_line_ending = source.dominant_line_ending.as_str();
    }
    options
}

fn nested_emit_policies<'a>(
    nodes: &'a [Node],
    states: &'a DirectiveStateTable,
    yaml: Option<&'a YamlDocumentAst>,
    options: FormatOptions,
) -> impl Iterator<Item = (usize, FormatOptions)> + 'a {
    let nodes = nodes.iter().filter_map(move |node| {
        let id = match node.emit {
            EmitPlan::MarkdownFrontMatter { nested, .. }
            | EmitPlan::MarkdownDiv { nested, .. }
            | EmitPlan::EmbeddedMarkdownString { nested, .. }
            | EmitPlan::EmbeddedMarkdownComment { nested, .. }
            | EmitPlan::EmbeddedYamlComment { nested, .. }
            | EmitPlan::MarkdownCodeFence {
                nested: Some(nested),
                ..
            } => nested,
            _ => return None,
        };
        let state = states.get(node.state);
        (!state.preserve).then(|| (id, state.markdown_options(options)))
    });
    let scalars = yaml
        .into_iter()
        .flat_map(|ast| &ast.nodes)
        .filter_map(move |node| {
            // Finalize the child selected by the YAML executor.
            let (YamlAstKind::Scalar(_), YamlEmitPlan::NestedMarkdownBlockScalar { nested }) =
                (&node.kind, &node.emit)
            else {
                return None;
            };
            Some((
                *nested as usize,
                states.get(node.state).markdown_options(options),
            ))
        });
    nodes.chain(scalars)
}

/// Resolve the privately owned tree under its settled directives. Called only
/// while constructing PreparedTree; execution has no mutable tree access.
pub(crate) fn finalize_document(
    source: &SourceBuffer,
    document: &mut Document,
    options: FormatOptions,
    document_options: bool,
) {
    if document_options && document.skip_file {
        return;
    }
    let owned_source = document.source.take();
    let source = if document_options {
        owned_source.as_ref().unwrap_or(source)
    } else {
        source
    };
    let options = if document_options {
        document_emit_options(source, document, options)
    } else {
        options
    };
    finalize_markdown_plans(source, document, options);
    let yaml_may_need_markdown_finalization = document.yaml_may_need_markdown_finalization;
    if let Some(ast) = document
        .yaml
        .as_mut()
        .filter(|_| yaml_may_need_markdown_finalization)
    {
        for node in &mut ast.nodes {
            if matches!(
                node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar)
            ) {
                let state = document.states.get(node.state);
                crate::core::yaml::resolve_inline_markdown_plan(
                    source,
                    node,
                    state.markdown_options(state.yaml_options(options)),
                );
            }
        }
    }
    for (id, options) in nested_emit_policies(
        &document.nodes,
        &document.states,
        document
            .yaml
            .as_ref()
            .filter(|_| yaml_may_need_markdown_finalization),
        options,
    ) {
        finalize_document(source, &mut document.nested[id], options, true);
    }
    document.source = owned_source;
}

pub(super) fn validate_markdown_format_target(
    source: &SourceBuffer,
    doc: &Document,
    state: StateId,
    span: Span,
    supported: bool,
) -> Result<()> {
    if doc.state(state).markdown_target && !supported {
        return Err(markdown_error_at(
            source,
            span.start,
            "fmt: markdown targets an unsupported Markdown block",
        ));
    }
    Ok(())
}

fn markdown_format_emit_plan(kind: MarkdownBlockFormatKind) -> EmitPlan {
    match kind {
        MarkdownBlockFormatKind::Paragraph => EmitPlan::MarkdownParagraph,
        MarkdownBlockFormatKind::Table => EmitPlan::MarkdownTable,
        MarkdownBlockFormatKind::PandocTable => EmitPlan::MarkdownPandocTable,
        MarkdownBlockFormatKind::List => EmitPlan::MarkdownList,
        MarkdownBlockFormatKind::DefinitionList => EmitPlan::MarkdownDefinitionList,
        MarkdownBlockFormatKind::Blockquote => EmitPlan::MarkdownBlockquote,
    }
}
