//! Emission of YAML documents, block mappings, and block sequences.

use super::block_scalar::{
    ExternalBlockScalarAction, emit_yaml_alias,
    emit_yaml_formatted_external_block_scalar_after_prefix, emit_yaml_nested_markdown_block_scalar,
    emit_yaml_scalar_after_prefix, external_block_scalar_action, line_ending_or_default,
};
use super::flow_emit::{
    emit_inline_comment, emit_spaces, emit_yaml_flow_collection,
    emit_yaml_flow_collection_block_into, emit_yaml_node_properties, mapping_pair_child_indent,
    yaml_output_is_at_line_start,
};
use super::scalar::{YamlScalarEmitPosition, emit_yaml_rendered_scalar_plan, emit_yaml_scalar};
use super::simple::strip_newline;
use super::syntax::{explicit_indicator_at, indentation, line_ending_for_span};
use super::table::{
    compact_root_collection_allowed, emit_compact_yaml_node, emit_flow_table_sequence_into,
    yaml_block_collection_has_flow_collapse_hint,
};
use crate::core::document::{Document, FormatOptions};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlDocumentAst, YamlEmitPlan, YamlMapping, YamlMappingPair, YamlNodeId,
    YamlRenderedKind, YamlSequenceItem, YamlTrivia,
};
use crate::diagnostic::Result;
use crate::plugins::PluginRegistry;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct YamlEmissionStats {
    pub emitted_nodes: usize,
}

pub(super) trait IntoOptionalSpan {
    fn into_optional_span(self) -> Option<Span>;
}

impl IntoOptionalSpan for Option<Span> {
    fn into_optional_span(self) -> Option<Span> {
        self
    }
}

impl IntoOptionalSpan for Option<SourceSpan> {
    fn into_optional_span(self) -> Option<Span> {
        self.map(SourceSpan::span)
    }
}

pub(crate) fn emit_planned_yaml_document_with_stats(
    source: &SourceBuffer,
    document: &Document,
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
    let insert_document_markers = yaml_is_unmarked_flow_mapping_stream(source, ast);
    for root in &ast.roots {
        if insert_document_markers {
            out.push_str("---");
            out.push_str(options.default_line_ending);
        }
        if let Some(node) = root.node {
            let context = YamlEmitContext {
                source,
                document,
                ast,
                options,
                plugins,
            };
            emit_yaml_node(&mut out, context, node, None, true, &mut stats)?;
        }
    }
    emit_trivia(&mut out, source, &ast.trailing_trivia);
    restore_yaml_bom(source, document, &mut out);
    Ok((out, stats))
}

fn yaml_is_unmarked_flow_mapping_stream(source: &SourceBuffer, ast: &YamlDocumentAst) -> bool {
    // Use the AST produced by the normal parse rather than scanning the source
    // again. Multiple one-line flow mappings need document markers, while the
    // mapping shape excludes valid YAML multiline scalars. The line count also
    // excludes multiline roots and intervening trivia.
    ast.roots.len() >= 2
        && ast.roots.len() == source.lines.len()
        && ast.roots.iter().all(|root| {
            root.start_marker.is_none()
                && root.end_marker.is_none()
                && root
                    .node
                    .is_some_and(|node| matches!(&ast.node(node).kind, YamlAstKind::FlowMapping(_)))
        })
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
pub(super) struct YamlEmitContext<'a> {
    pub(super) source: &'a SourceBuffer,
    pub(super) document: &'a Document,
    pub(super) ast: &'a YamlDocumentAst,
    pub(super) options: FormatOptions,
    pub(super) plugins: &'a PluginRegistry,
}

impl YamlEmitContext<'_> {
    pub(super) fn with_options(self, options: FormatOptions) -> Self {
        Self { options, ..self }
    }
}

fn emit_yaml_node(
    out: &mut String,
    context: YamlEmitContext<'_>,
    id: YamlNodeId,
    forced_indent: Option<usize>,
    is_root: bool,
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
    let forced_indent = forced_indent.filter(|_| !node.requires_source_indent);
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
        let root_line_start = is_root && yaml_output_is_at_line_start(out);
        emit_yaml_rendered_scalar_plan(
            out,
            source,
            scalar,
            node,
            state,
            options,
            YamlScalarEmitPosition {
                body_indent: None,
                root_line_start,
            },
        );
        return Ok(());
    }
    if matches!(
        node.emit,
        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
    ) && is_root
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
                let entry_span = Span::new(item.line.start(), value_node.span.end());
                if matches!(value_node.kind, YamlAstKind::Opaque(_)) {
                    out.push_str(source.slice(entry_span));
                    continue;
                }
                if yaml_block_collection_has_flow_collapse_hint(value_node)
                    && matches!(value_node.emit, YamlEmitPlan::PreserveSource)
                {
                    out.push_str(source.slice(entry_span));
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
                    out.push_str(source.slice(entry_span));
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
                        out.push_str(source.slice(entry_span));
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
                            false,
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
                            out.push_str(source.slice(entry_span));
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
                    let mapping_indent = if value_node.requires_source_indent {
                        mapping.indent
                    } else {
                        indent + 2
                    };
                    emit_yaml_inline_sequence_mapping(
                        out,
                        source,
                        document,
                        ast,
                        mapping,
                        item,
                        indent,
                        mapping_indent,
                        options,
                        plugins,
                        stats,
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
                            false,
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
                                YamlScalarEmitPosition {
                                    body_indent: None,
                                    root_line_start: false,
                                },
                            );
                        } else {
                            emit_inline_comment(out, source, item.trailing_comment);
                            out.push_str(line_ending_for_span(source, item.line));
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
                            Some(indent + options.indent_width),
                        )?;
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
                            emit_yaml_node(out, context, value, None, false, stats)?;
                        }
                    }
                    _ => {
                        out.push(' ');
                        emit_yaml_node(out, context, value, None, false, stats)?;
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
    ast: &YamlDocumentAst,
    mapping: &YamlMapping,
    item: &YamlSequenceItem,
    indent: usize,
    mapping_indent: usize,
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

    let first_prefix = format!(
        "{}-{}",
        " ".repeat(indent),
        " ".repeat(mapping_indent - indent - 1)
    );
    let child_indent = mapping_indent + options.indent_width;
    let continuation_prefix = " ".repeat(mapping_indent);
    let pair_child_indent = |pair: &YamlMappingPair| {
        if pair
            .value
            .is_some_and(|value| matches!(ast.node(value).kind, YamlAstKind::Sequence(_)))
        {
            mapping_indent
        } else {
            child_indent
        }
    };
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
            pair_child_indent(first),
            child_indent,
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
            pair_child_indent(first),
            child_indent,
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
            pair_child_indent(pair),
            child_indent,
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
    ast: &YamlDocumentAst,
    pair: &YamlMappingPair,
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
    if matches!(value_node.kind, YamlAstKind::Opaque(_)) {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }
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
        out.push_str(source.slice(pair.source));
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
                out.push_str(source.slice(pair.source));
            }
            ExternalBlockScalarAction::Formatted(formatted) => {
                emit_yaml_line_prefix(out, prefix);
                let key = source.slice(pair.key).trim_ascii();
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
            out.push_str(source.slice(pair.source));
        } else {
            emit_yaml_line_prefix(out, prefix);
            out.push_str(source.slice(pair.key).trim_ascii());
            out.push(':');
            emit_yaml_node_properties(out, source, value_node);
            emit_inline_comment(out, source, pair.trailing_comment);
            out.push_str(line_ending_for_span(source, pair.line));
            emit_yaml_node(
                out,
                context,
                value,
                Some(mapping_pair_child_indent(value_node, child_indent)),
                false,
                stats,
            )?;
        }
        return Ok(());
    }

    emit_yaml_line_prefix(out, prefix);
    let key = source.slice(pair.key).trim_ascii();
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
    ast: &YamlDocumentAst,
    pair: &YamlMappingPair,
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
    if matches!(value_node.kind, YamlAstKind::Opaque(_)) {
        out.push_str(source.slice(pair.source));
        return Ok(());
    }
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
    out.push_str(key_body.trim_ascii_end());
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
    pair: &YamlMappingPair,
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
    pair: &YamlMappingPair,
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
                    false,
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
                let newline = line_ending_or_default(source, pair.line, options);
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
                emit_yaml_node(out, context, value, None, false, stats)?;
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
                    YamlScalarEmitPosition {
                        body_indent: None,
                        root_line_start: false,
                    },
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
            emit_yaml_node(out, context, value, None, false, stats)?;
        }
    }
    Ok(())
}

pub(super) fn emit_trivia(out: &mut String, source: &SourceBuffer, trivia: &[YamlTrivia]) {
    for item in trivia {
        let _ = item.kind;
        out.push_str(source.slice(item.span));
    }
}
