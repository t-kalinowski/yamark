//! Flow collection layout, inline widths, properties, and preservation decisions.

use super::block_scalar::{line_ending_or_default, yaml_scalar_is_markdown_target};
use super::emit::IntoOptionalSpan;
use super::scalar::{
    emit_yaml_scalar_inline_into, render_folded_prose_scalar_with_layout,
    render_quoted_literal_scalar_with_layout, render_yaml_scalar_inline_width,
};
use super::syntax::{indentation, line_ending_for_span, preserves_yaml_template_span};
use crate::core::document::{Document, FormatOptions};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlAstNode, YamlDocumentAst, YamlEmitPlan, YamlNodeId, YamlRenderedKind,
    YamlScalar, YamlScalarStyle,
};
use memchr::memchr;

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_yaml_flow_collection(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
    node: &YamlAstNode,
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
                out.push_str(source.slice(sequence.value).trim_ascii());
            }
            YamlAstKind::FlowMapping(mapping) => {
                out.push_str(source.slice(mapping.value).trim_ascii());
            }
            _ => out.push_str(source.slice(node.span).trim_ascii()),
        },
    }
    emit_inline_comment(out, source, flow_trailing_comment(&node.kind));
    out.push_str(line_ending_for_span(source, node.span));
}

pub(super) fn planned_yaml_inline_width_or_source(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
        YamlAstKind::FlowSequence(sequence) => {
            source.slice(sequence.value).trim_ascii().chars().count()
        }
        YamlAstKind::FlowMapping(mapping) => {
            source.slice(mapping.value).trim_ascii().chars().count()
        }
        _ => source.slice(node.span).trim_ascii().chars().count(),
    }
}

fn emit_yaml_inline_node_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<()> {
    emit_yaml_inline_node_into_with_context(out, source, document, ast, id, false)
}

pub(super) fn emit_yaml_inline_node_into_for_flow(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<()> {
    emit_yaml_inline_node_into_with_context(out, source, document, ast, id, true)
}

fn emit_yaml_inline_node_into_with_context(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
            YamlAstKind::Scalar(scalar) => out.push_str(source.slice(scalar.value).trim_ascii()),
            YamlAstKind::Alias(alias) => out.push_str(source.slice(alias.value).trim_ascii()),
            YamlAstKind::FlowSequence(sequence) => {
                out.push_str(source.slice(sequence.value).trim_ascii());
            }
            YamlAstKind::FlowMapping(mapping) => {
                out.push_str(source.slice(mapping.value).trim_ascii())
            }
            _ => out.push_str(source.slice(node.span).trim_ascii()),
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
            out.push_str(source.slice(alias.value).trim_ascii());
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
                out.push_str(source.slice(mapping.value).trim_ascii());
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

pub(super) fn render_yaml_inline_node_width(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<usize> {
    render_yaml_inline_node_width_with_context(source, document, ast, id, false)
}

pub(super) fn render_yaml_inline_node_width_for_flow(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
) -> Option<usize> {
    render_yaml_inline_node_width_with_context(source, document, ast, id, true)
}

fn render_yaml_inline_node_width_with_context(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
            YamlAstKind::Scalar(scalar) => source.slice(scalar.value).trim_ascii().chars().count(),
            YamlAstKind::Alias(alias) => source.slice(alias.value).trim_ascii().chars().count(),
            YamlAstKind::FlowSequence(sequence) => {
                source.slice(sequence.value).trim_ascii().chars().count()
            }
            YamlAstKind::FlowMapping(mapping) => {
                source.slice(mapping.value).trim_ascii().chars().count()
            }
            _ => source.slice(node.span).trim_ascii().chars().count(),
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
            Some(source.slice(alias.value).trim_ascii().chars().count())
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
                return Some(source.slice(mapping.value).trim_ascii().chars().count());
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
pub(super) fn yaml_flow_collection_block_renderable(
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
                    let child_indent =
                        if matches!(ast.node(*entry).kind, YamlAstKind::FlowMapping(_)) {
                            indent + 2
                        } else {
                            indent + options.indent_width
                        };
                    yaml_flow_collection_block_renderable(
                        source,
                        document,
                        ast,
                        *entry,
                        child_indent,
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
pub(super) fn emit_yaml_flow_collection_block_into(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    ast: &YamlDocumentAst,
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
    ast: &YamlDocumentAst,
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
                if let Some(rendered) = render_yaml_flow_block_scalar(
                    source,
                    ast,
                    *entry,
                    indent + options.indent_width,
                    newline,
                    options,
                ) {
                    emit_yaml_flow_block_line_prefix(out, indent, first_line_prefix);
                    out.push_str("- ");
                    out.push_str(&rendered);
                    continue;
                }
                let value_width =
                    render_yaml_inline_node_width_for_flow(source, document, ast, *entry)?;
                emit_yaml_flow_block_line_prefix(out, indent, first_line_prefix);
                if yaml_node_is_flow_collection(ast.node(*entry))
                    && indent + 2 + value_width > options.line_width
                {
                    out.push('-');
                    if matches!(ast.node(*entry).kind, YamlAstKind::FlowMapping(_)) {
                        let mut mapping_first_line_prefix = Some(" ");
                        emit_yaml_flow_collection_block_lines(
                            out,
                            source,
                            document,
                            ast,
                            *entry,
                            indent + 2,
                            newline,
                            options,
                            &mut mapping_first_line_prefix,
                        )?;
                    } else {
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
                    }
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
                    if let Some(rendered) = render_yaml_flow_block_scalar(
                        source,
                        ast,
                        value_id,
                        indent + options.indent_width,
                        newline,
                        options,
                    ) {
                        Some((value_id, false, Some(rendered)))
                    } else {
                        let value_width = render_yaml_inline_node_width_for_flow(
                            source, document, ast, value_id,
                        )?;
                        Some((
                            value_id,
                            yaml_node_is_flow_collection(ast.node(value_id))
                                && indent + key_width + 2 + value_width > options.line_width,
                            None,
                        ))
                    }
                } else {
                    None
                };
                emit_yaml_flow_block_line_prefix(out, indent, first_line_prefix);
                emit_yaml_inline_node_into_for_flow(out, source, document, ast, pair.key)?;
                out.push(':');
                if let Some((value_id, should_expand, rendered)) = value_action {
                    if let Some(rendered) = rendered {
                        out.push(' ');
                        out.push_str(&rendered);
                    } else if should_expand {
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

fn render_yaml_flow_block_scalar(
    source: &SourceBuffer,
    ast: &YamlDocumentAst,
    id: YamlNodeId,
    body_indent: usize,
    newline: &str,
    options: FormatOptions,
) -> Option<String> {
    let node = ast.node(id);
    let YamlAstKind::Scalar(scalar) = &node.kind else {
        return None;
    };
    if !matches!(node.emit, YamlEmitPlan::Rendered(YamlRenderedKind::Scalar)) {
        return None;
    }
    let raw = source.slice(scalar.value).trim_ascii();
    let literal_candidate = match scalar.style {
        YamlScalarStyle::DoubleQuoted => memchr(b'\\', raw.as_bytes()).is_some(),
        YamlScalarStyle::SingleQuoted => raw.contains("''") && raw.contains(['"', '\\']),
        YamlScalarStyle::Plain | YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => {
            false
        }
    };
    if literal_candidate
        && let Some(rendered) = render_quoted_literal_scalar_with_layout(
            source,
            scalar,
            node,
            options,
            Some(body_indent),
            newline,
            true,
        )
    {
        return Some(rendered);
    }
    // Wrapping only breaks at decoded ASCII spaces. Reject opaque values
    // before decoding them again for folded prose.
    let folded_candidate = raw.contains(' ')
        || scalar.style == YamlScalarStyle::DoubleQuoted
            && (raw.contains("\\x20") || raw.contains("\\u0020") || raw.contains("\\U00000020"));
    if !folded_candidate {
        return None;
    }
    render_folded_prose_scalar_with_layout(
        source,
        scalar,
        node,
        options,
        Some(body_indent),
        newline,
        true,
    )
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

pub(super) fn emit_spaces(out: &mut String, count: usize) {
    const SPACES: &str = "                                ";
    let mut remaining = count;
    while remaining >= SPACES.len() {
        out.push_str(SPACES);
        remaining -= SPACES.len();
    }
    out.push_str(&SPACES[..remaining]);
}

fn yaml_node_is_flow_collection(node: &YamlAstNode) -> bool {
    matches!(
        node.kind,
        YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_)
    )
}

pub(super) fn yaml_flow_collection_should_expand(
    source: &SourceBuffer,
    node: &YamlAstNode,
    inline_width: usize,
    options: FormatOptions,
) -> bool {
    inline_width > options.line_width
        || yaml_flow_collection_has_multiline_intent(source, node, options)
}

pub(super) fn yaml_flow_collection_has_multiline_intent(
    source: &SourceBuffer,
    node: &YamlAstNode,
    options: FormatOptions,
) -> bool {
    !options.yaml_compact && yaml_flow_collection_has_source_newline(source, node)
}

fn yaml_flow_collection_has_source_newline(source: &SourceBuffer, node: &YamlAstNode) -> bool {
    let Some(span) = yaml_flow_collection_source_span(node) else {
        return false;
    };
    !span.is_empty() && source.line_at_byte(span.start()) != source.line_at_byte(span.end())
}

fn yaml_flow_collection_source_contains_tab(source: &SourceBuffer, node: &YamlAstNode) -> bool {
    let Some(span) = yaml_flow_collection_source_span(node) else {
        return false;
    };
    source.slice(span).as_bytes().contains(&b'\t')
}

fn yaml_flow_collection_source_span(node: &YamlAstNode) -> Option<SourceSpan> {
    match &node.kind {
        YamlAstKind::FlowSequence(sequence) => Some(sequence.value),
        YamlAstKind::FlowMapping(mapping) => Some(mapping.value),
        _ => None,
    }
}

pub(super) fn flow_trailing_comment(kind: &YamlAstKind) -> Option<SourceSpan> {
    match kind {
        YamlAstKind::FlowSequence(sequence) => sequence.trailing_comment,
        YamlAstKind::FlowMapping(mapping) => mapping.trailing_comment,
        _ => None,
    }
}

pub(super) fn yaml_node_should_preserve(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode,
) -> bool {
    if let Some(must_preserve_source) = node.must_preserve_source {
        return must_preserve_source;
    }
    yaml_node_should_preserve_uncached(source, document, node)
}

pub(super) fn yaml_node_should_preserve_uncached(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode,
) -> bool {
    yaml_node_should_preserve_uncached_with_template_possible(source, document, node, true)
}

pub(super) fn yaml_node_should_preserve_uncached_with_template_possible(
    source: &SourceBuffer,
    document: &Document,
    node: &YamlAstNode,
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

fn yaml_node_preserves_own_template_span(kind: &YamlAstKind) -> bool {
    matches!(
        kind,
        YamlAstKind::Scalar(_)
            | YamlAstKind::FlowSequence(_)
            | YamlAstKind::FlowMapping(_)
            | YamlAstKind::Opaque(_)
    )
}

pub(super) fn source_contains_any_template_opener(
    source: &str,
    delimiters: &[crate::core::directives::TemplateDelimiter],
) -> bool {
    delimiters.iter().any(|delimiter| {
        !delimiter.open.is_empty()
            && !delimiter.close.is_empty()
            && source.contains(&delimiter.open)
    })
}

pub(super) fn source_may_contain_template_span(
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

pub(super) fn yaml_node_has_properties(source: &SourceBuffer, node: &YamlAstNode) -> bool {
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

pub(super) fn scalar_has_properties(scalar: &YamlScalar) -> bool {
    scalar.tag.is_some() || scalar.anchor.is_some()
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
    kind: &YamlAstKind,
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
    tag: Option<SourceSpan>,
    anchor: Option<SourceSpan>,
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
    tag: Option<SourceSpan>,
    anchor: Option<SourceSpan>,
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

pub(super) fn mapping_pair_child_indent(_node: &YamlAstNode, default: usize) -> usize {
    default
}

pub(super) fn yaml_node_source_indent(source: &SourceBuffer, node: &YamlAstNode) -> usize {
    if let Some(indent) = node.source_indent() {
        return indent;
    }
    let line = source.line_at_byte(node.span.start());
    let indent = indentation(source.line_text(line));
    node.set_source_indent(indent);
    indent
}

fn yaml_node_properties(kind: &YamlAstKind) -> Option<(Option<SourceSpan>, Option<SourceSpan>)> {
    match kind {
        YamlAstKind::Mapping(mapping) => Some((mapping.tag, mapping.anchor)),
        YamlAstKind::Sequence(sequence) => Some((sequence.tag, sequence.anchor)),
        YamlAstKind::FlowMapping(mapping) => Some((mapping.tag, mapping.anchor)),
        YamlAstKind::FlowSequence(sequence) => Some((sequence.tag, sequence.anchor)),
        _ => None,
    }
}

pub(super) fn emit_yaml_node_properties(
    out: &mut String,
    source: &SourceBuffer,
    node: &YamlAstNode,
) {
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

pub(super) fn emit_inline_comment(
    out: &mut String,
    source: &SourceBuffer,
    comment: impl IntoOptionalSpan,
) {
    if let Some(comment) = comment.into_optional_span() {
        out.push(' ');
        out.push_str(source.slice(comment).trim_ascii_end());
    }
}

pub(super) fn yaml_output_is_at_line_start(output: &str) -> bool {
    output.is_empty() || output.ends_with(['\n', '\r'])
}
