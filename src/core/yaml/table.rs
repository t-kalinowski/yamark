//! Compact collections and aligned flow-table emission.

use super::emit::emit_trivia;
use super::flow_emit::{
    emit_yaml_inline_node_into_for_flow, render_yaml_inline_node_width_for_flow,
    yaml_node_has_properties, yaml_node_should_preserve,
};
use super::scalar::{
    flow_plain_scalar_safe, quote_yaml_single_for_flow, quote_yaml_single_for_flow_width,
};
use super::syntax::line_ending_for_span;
use crate::core::document::Document;
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlAstNode, YamlDocumentAst, YamlFlowMapping, YamlMapping, YamlMappingPair,
    YamlNodeId, YamlSequence, YamlTrivia,
};

pub(super) fn compact_root_collection_allowed(ast: &YamlDocumentAst, id: YamlNodeId) -> bool {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(mapping) => {
            mapping.pairs.len() > 1
                && mapping.pairs.iter().all(|pair| {
                    pair.value
                        .is_some_and(|value| compact_root_child_allowed(ast, value))
                })
        }
        YamlAstKind::Sequence(sequence) => sequence.items.iter().all(|item| {
            item.value
                .is_some_and(|value| compact_root_child_allowed(ast, value))
        }),
        _ => true,
    }
}

fn compact_root_child_allowed(ast: &YamlDocumentAst, id: YamlNodeId) -> bool {
    // Keep the root layout stable when a child collection collapses to flow.
    !matches!(
        ast.node(id).kind,
        YamlAstKind::Mapping(_)
            | YamlAstKind::Sequence(_)
            | YamlAstKind::FlowMapping(_)
            | YamlAstKind::FlowSequence(_)
    )
}

pub(super) fn yaml_node_is_block_collection(ast: &YamlDocumentAst, id: YamlNodeId) -> bool {
    matches!(
        ast.node(id).kind,
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
    )
}

pub(super) fn yaml_block_collection_has_flow_collapse_hint(node: &YamlAstNode) -> bool {
    match &node.kind {
        YamlAstKind::Mapping(mapping) => mapping.flow_collapse_hint.is_some(),
        YamlAstKind::Sequence(sequence) => sequence.flow_collapse_hint.is_some(),
        _ => false,
    }
}

fn compact_yaml_value_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<usize> {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
            compact_yaml_node_width(source, document, ast, id)
        }
        _ => render_yaml_inline_node_width_for_flow(source, document, ast, id),
    }
}

pub(super) fn compact_yaml_node_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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

pub(super) fn emit_compact_yaml_node(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<()> {
    match &ast.node(id).kind {
        YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
            emit_compact_yaml_node(out, source, document, ast, id)
        }
        _ => emit_yaml_inline_node_into_for_flow(out, source, document, ast, id),
    }
}

pub(super) fn flow_table_sequence_renderable(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    sequence: &YamlSequence,
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
pub(super) fn emit_flow_table_sequence_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    sequence: &YamlSequence,
    indent: usize,
    compact_table: bool,
    node_leading: &[YamlTrivia],
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
    ast: &YamlDocumentAst,
    sequence: &YamlSequence,
    indent: usize,
    compact_table: bool,
    node_leading: &[YamlTrivia],
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
    ast: &YamlDocumentAst,
    value_node: &YamlAstNode,
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
    ast: &YamlDocumentAst,
    value_node: &YamlAstNode,
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
    ast: &YamlDocumentAst,
    mapping: &YamlFlowMapping,
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
    ast: &YamlDocumentAst,
    mapping: &YamlFlowMapping,
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

fn flow_mapping_table_supported(mapping: &YamlFlowMapping) -> bool {
    mapping.braced
        && !mapping.has_inner_trivia
        && mapping.tag.is_none()
        && mapping.anchor.is_none()
        && mapping.pairs.iter().all(|pair| !pair.explicit)
}

fn block_mapping_table_field_count(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    mapping: &YamlMapping,
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
    ast: &YamlDocumentAst,
    mapping: &YamlMapping,
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

fn block_mapping_table_supported(mapping: &YamlMapping) -> bool {
    mapping.tag.is_none()
        && mapping.anchor.is_none()
        && mapping.pairs.iter().all(|pair| !pair.explicit)
}

fn emit_mapping_key_for_flow(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    pair: &YamlMappingPair,
) -> Option<()> {
    emit_mapping_key_for_flow_parts(out, source, document, ast, pair.key.span(), pair.key_node)
}

fn emit_mapping_key_for_flow_parts(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    key: Span,
    key_node: Option<YamlNodeId>,
) -> Option<()> {
    if let Some(key) = key_node {
        return emit_yaml_inline_node_into_for_flow(out, source, document, ast, key);
    }
    let key = source.slice(key).trim_ascii();
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
    ast: &YamlDocumentAst,
    key: Span,
    key_node: Option<YamlNodeId>,
) -> Option<usize> {
    if let Some(key) = key_node {
        return render_yaml_inline_node_width_for_flow(source, document, ast, key);
    }
    let key = source.slice(key).trim_ascii();
    if flow_plain_scalar_safe(key) {
        Some(key.chars().count())
    } else {
        Some(quote_yaml_single_for_flow_width(key))
    }
}

fn render_mapping_key_width_for_flow(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    pair: &YamlMappingPair,
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
    document: &'a Document,
    ast: &'a YamlDocumentAst,
    fields: &'a [FlowTableField],
    field_widths: &'a [usize],
    indent: usize,
    align_closing_brace: bool,
}

fn emit_flow_table_rows(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
            out.push_str(source.slice(comment).trim_ascii_end());
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
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
