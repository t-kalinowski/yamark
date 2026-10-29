//! Block-scalar indentation, Markdown fragments, and external formatter output.

use super::emit::{IntoOptionalSpan, YamlEmitContext};
use super::flow_emit::emit_inline_comment;
use super::scalar::{
    YamlScalarEmitPosition, decode_quoted_scalar, emit_yaml_rendered_scalar_plan, emit_yaml_scalar,
    scalar_property_prefix, simple_quoted_scalar_inner,
};
use super::simple::{format_markdown_yaml_block_value, format_simple_yaml_line, strip_newline};
use super::syntax::{find_trailing_comment, indentation, line_ending_for_span, scalar_metadata};
use crate::core::document::{Document, FormatOptions};
use crate::core::source::{SourceBuffer, SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAlias, YamlAstKind, YamlAstNode, YamlBlockChomp, YamlEmitPlan, YamlScalar, YamlScalarStyle,
    YamlTriviaKind,
};
use crate::diagnostic::Result;
use crate::plugins::PluginRegistry;
use std::borrow::Cow;

pub(super) enum ExternalBlockScalarAction {
    Preserve,
    Formatted(String),
}

pub(super) fn yaml_scalar_keeps_parent_header(
    node: &YamlAstNode,
    parent_line: SourceSpan,
    parent_comment: Option<SourceSpan>,
) -> bool {
    matches!(&node.kind, YamlAstKind::Scalar(scalar)
        if scalar.value.start() >= parent_line.end()
            && (parent_comment.is_some() || node.leading_trivia.iter().any(|trivia|
                matches!(trivia.kind, YamlTriviaKind::Comment | YamlTriviaKind::Directive))))
}

pub(super) fn line_ending_or_default(
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

/// Keep the decoded fragment, including the existing no-output result when
/// decoding fails. The node and its source remain in the same owning tree.
#[derive(Debug, Clone)]
pub(crate) struct InlineMarkdownPlan {
    fragment: Option<crate::core::wrap::Fragment>,
}

pub(crate) fn resolve_inline_markdown_plan(
    source: &SourceBuffer,
    node: &mut YamlAstNode,
    options: FormatOptions,
) {
    let YamlAstKind::Scalar(scalar) = &node.kind else {
        unreachable!("inline Markdown scalar plan")
    };
    if let Some(plan) = &mut node.inline_markdown {
        if let Some(fragment) = &mut plan.fragment {
            fragment.resolve_options(options);
        }
    } else {
        let metadata = scalar_metadata(source, scalar.value);
        let fragment = inline_markdown_scalar_content(source, scalar, metadata.content)
            .map(|content| crate::core::wrap::Fragment::plan(content.into_owned(), options));
        node.inline_markdown = Some(Box::new(InlineMarkdownPlan { fragment }));
    }
}

pub(super) fn render_inline_markdown_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    _state: &crate::core::directives::DirectiveState,
    options: FormatOptions,
    body_indent: Option<usize>,
) -> Option<String> {
    let metadata = scalar_metadata(source, scalar.value);
    let fragment = node
        .inline_markdown
        .as_ref()
        .expect("retained inline Markdown scalar")
        .fragment
        .as_ref()?;
    let prefix = scalar_property_prefix(source, scalar, metadata);
    let newline = line_ending_or_default(source, node.span, options);

    let mut out = String::new();
    if fragment.is_empty() {
        if !prefix.is_empty() {
            out.push_str(&prefix);
            out.push(' ');
        }
        out.push_str("\"\"");
        emit_inline_comment(&mut out, source, scalar.trailing_comment);
        out.push_str(newline);
        return Some(out);
    }

    if !prefix.is_empty() {
        out.push_str(&prefix);
        out.push(' ');
    }
    out.push('|');
    emit_inline_comment(&mut out, source, scalar.trailing_comment);
    out.push_str(newline);

    let mut formatted = fragment.emit();
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

pub(super) fn inline_markdown_scalar_is_renderable(
    source: &SourceBuffer,
    scalar: &YamlScalar,
) -> bool {
    let metadata = scalar_metadata(source, scalar.value);
    inline_markdown_scalar_content(source, scalar, metadata.content).is_some()
}

fn inline_markdown_scalar_content<'a>(
    source: &'a SourceBuffer,
    scalar: &YamlScalar,
    content: Span,
) -> Option<Cow<'a, str>> {
    let raw = source.slice(content).trim_ascii();
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
    scalar: &YamlScalar,
    node: &YamlAstNode,
    body: &str,
    options: FormatOptions,
) -> String {
    let Some(source_body) = scalar.body else {
        return body.to_owned();
    };
    let strip_indent = explicit_block_scalar_body_indent(scalar)
        .or_else(|| block_scalar_body_indent(source.slice(source_body)))
        .unwrap_or_else(|| {
            let line_index = source.line_at_byte(node.span.start());
            indentation(source.line_text(line_index)) + options.indent_width
        });
    let emit_indent = strip_indent;
    reindent_block_lines(body, strip_indent, emit_indent)
}

pub(super) fn explicit_block_scalar_body_indent(scalar: &YamlScalar) -> Option<usize> {
    let header = scalar.block_header?;
    Some(header.base_indent + header.indent? as usize)
}

pub(super) fn block_scalar_output_body_indent(
    scalar: &YamlScalar,
    body_indent: usize,
    options: FormatOptions,
) -> usize {
    scalar
        .block_header
        .and_then(|header| header.indent)
        .map(|indent| body_indent.saturating_sub(options.indent_width) + indent as usize)
        .unwrap_or(body_indent)
}

pub(super) fn block_scalar_body_reindent_is_safe(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    body_indent: Option<usize>,
    options: FormatOptions,
) -> bool {
    let Some(body) = scalar.body else {
        return true;
    };
    let Some(body_indent) = body_indent else {
        return true;
    };
    let body = source.slice(body);
    let Some(source_indent) =
        explicit_block_scalar_body_indent(scalar).or_else(|| block_scalar_body_indent(body))
    else {
        return true;
    };
    if source_indent == block_scalar_output_body_indent(scalar, body_indent, options) {
        return true;
    }

    let implicit_indent = scalar
        .block_header
        .is_some_and(|header| header.indent.is_none());
    let mut saw_content = false;
    for line in body.split(['\r', '\n']) {
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        if line.trim_matches(' ').is_empty() {
            if implicit_indent && !saw_content && indent > source_indent {
                return false;
            }
            continue;
        }
        saw_content = true;
        if indent < source_indent {
            return false;
        }
    }
    true
}

pub(super) fn block_scalar_body_indent(body: &str) -> Option<usize> {
    body.split(['\r', '\n'])
        .find(|line| !line.trim_matches(' ').is_empty())
        .map(|line| line.bytes().take_while(|byte| *byte == b' ').count())
}

pub(super) fn normalize_nontrailing_block_scalar_blank_lines(
    body: &str,
    indent: usize,
) -> Cow<'_, str> {
    let mut content_end = 0;
    let mut offset = 0;
    for line in body.split_inclusive(['\r', '\n']) {
        offset += line.len();
        let (line_body, _) = strip_newline(line);
        if line_body.trim_matches(' ').is_empty() {
            continue;
        }
        if line_body.bytes().take_while(|byte| *byte == b' ').count() < indent {
            break;
        }
        content_end = offset;
    }
    let content = &body[..content_end];
    let needs_normalizing = content.split(['\r', '\n']).any(|line| {
        !line.is_empty() && line.len() <= indent && line.bytes().all(|byte| byte == b' ')
    });
    if !needs_normalizing {
        return Cow::Borrowed(body);
    }
    let mut output = reindent_block_lines(content, indent, indent);
    output.push_str(&body[content_end..]);
    Cow::Owned(output)
}

pub(super) fn reindent_block_lines(body: &str, strip_indent: usize, emit_indent: usize) -> String {
    let prefix = " ".repeat(emit_indent);
    let strip = " ".repeat(strip_indent);
    let mut out = String::with_capacity(body.len() + prefix.len());
    let body = SourceBuffer::new(body.to_owned());
    for line in &body.lines {
        let line_body = body.slice(line.text);
        let newline = line.ending.as_str();
        if line_body.is_empty() {
            out.push_str(newline);
            continue;
        }
        if line_body.bytes().all(|byte| byte == b' ') {
            if let Some(content) = line_body.strip_prefix(&strip)
                && !content.is_empty()
            {
                out.push_str(&prefix);
                out.push_str(content);
            }
            out.push_str(newline);
            continue;
        }
        let content = line_body.strip_prefix(&strip).unwrap_or(line_body);
        out.push_str(&prefix);
        out.push_str(content);
        out.push_str(newline);
    }
    out
}

pub(super) fn emit_yaml_alias(
    out: &mut String,
    source: &SourceBuffer,
    alias: &YamlAlias,
    node: &YamlAstNode,
) {
    out.push_str(source.slice(alias.value).trim_ascii());
    emit_inline_comment(out, source, alias.trailing_comment);
    out.push_str(line_ending_for_span(source, node.span));
}

pub(super) fn emit_yaml_scalar_after_prefix(
    out: &mut String,
    context: YamlEmitContext<'_>,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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
            emit_yaml_rendered_scalar_plan(
                out,
                source,
                scalar,
                node,
                state,
                options,
                YamlScalarEmitPosition {
                    body_indent,
                    root_line_start: false,
                },
            );
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
                body_indent,
            )
        }
        _ => emit_yaml_scalar(out, source, document, scalar, node, options, plugins),
    }
}

pub(super) fn render_yaml_block_scalar_value_header(
    source: &SourceBuffer,
    scalar: &YamlScalar,
) -> String {
    let header = scalar.header.expect("block scalars have headers");
    let metadata = scalar_metadata(source, scalar.value);
    let mut output = scalar_property_prefix(source, scalar, metadata);
    if !output.is_empty() {
        output.push(' ');
    }
    let value = if scalar.nested.is_some() && scalar.style == YamlScalarStyle::FoldedBlock {
        format_markdown_yaml_block_value(source.slice(metadata.content))
    } else {
        source.slice(metadata.content).trim_ascii().to_owned()
    };
    output.push_str(&value);
    emit_inline_comment(&mut output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, header));
    output
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_yaml_nested_markdown_block_scalar(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    plugins: &PluginRegistry,
    nested: usize,
    body_indent: Option<usize>,
) -> Result<()> {
    let header_span = scalar.header.expect("block scalars have headers");
    out.push_str(&render_yaml_block_scalar_value_header(source, scalar));
    let state = document.state(node.state);
    let mut nested_output = crate::core::emit::emit_planned_document(
        source,
        &document.nested[nested],
        state.markdown_options(options),
        plugins,
        false,
    )?;
    if !nested_output.is_empty() && !nested_output.ends_with('\n') && !nested_output.ends_with('\r')
    {
        nested_output.push_str(line_ending_for_span(source, header_span));
    }
    let body_indent = body_indent.unwrap_or_else(|| {
        let line = source.line_at_byte(header_span.start());
        indentation(source.line_text(line)) + options.indent_width
    });
    out.push_str(&reindent_block_lines(
        &nested_output,
        0,
        block_scalar_output_body_indent(scalar, body_indent, options),
    ));
    Ok(())
}

pub(super) fn external_block_scalar_action(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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
            crate::core::emit::append_trailing_line_ending_if_missing(
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
    scalar: &YamlScalar,
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

fn yaml_block_scalar_formatter_input(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
) -> String {
    let Some(source_body) = scalar.body else {
        return String::new();
    };
    let strip_indent = explicit_block_scalar_body_indent(scalar)
        .or_else(|| block_scalar_body_indent(source.slice(source_body)))
        .unwrap_or_else(|| {
            let line_index = source.line_at_byte(node.span.start());
            indentation(source.line_text(line_index)) + options.indent_width
        });
    reindent_block_lines(source.slice(source_body), strip_indent, 0)
}

pub(super) fn emit_yaml_formatted_external_block_scalar(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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

pub(super) fn emit_yaml_formatted_external_block_scalar_after_prefix(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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

pub(super) fn yaml_scalar_is_markdown_target(
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar,
    node: &YamlAstNode,
) -> bool {
    document.state(node.state).markdown_target
        || scalar
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(source.slice(tag)))
}

pub(super) fn yaml_tag_is_markdown(tag: &str) -> bool {
    matches!(tag, "!markdown" | "!md")
}

pub(super) fn markdown_block_scalar_body_is_empty(
    source: &SourceBuffer,
    scalar: &YamlScalar,
) -> bool {
    scalar
        .body
        .is_some_and(|body| source.slice(body).is_empty())
}

pub(super) fn render_empty_markdown_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
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
