//! YAML scalar normalization, quoting, widths, and prose layout.

use super::block_scalar::{
    ExternalBlockScalarAction, block_scalar_body_indent, block_scalar_output_body_indent,
    emit_yaml_formatted_external_block_scalar, emit_yaml_nested_markdown_block_scalar,
    explicit_block_scalar_body_indent, external_block_scalar_action, line_ending_or_default,
    normalize_nontrailing_block_scalar_blank_lines, reindent_block_lines,
    render_empty_markdown_scalar, render_inline_markdown_scalar,
    render_yaml_block_scalar_value_header,
};
use super::flow_emit::{emit_inline_comment, emit_spaces, scalar_has_properties};
use super::simple::strip_newline;
use super::syntax::{
    ScalarMetadata, document_marker, find_trailing_comment, indentation, line_ending_for_span,
    scalar_metadata, scalar_style,
};
use crate::core::document::{Document, FormatOptions};
use crate::core::source::{SourceBuffer, Span};
use crate::core::yaml_model::{
    YamlAstNode, YamlEmitPlan, YamlRenderedKind, YamlScalar, YamlScalarSemantic, YamlScalarStyle,
};
use crate::core::yaml_scalar;
use crate::core::yaml_scalar::{DecodePolicy, decode_quoted_chars};
use crate::diagnostic::Result;
use crate::plugins::PluginRegistry;
use std::borrow::Cow;

pub(super) fn emit_yaml_scalar(
    out: &mut String,
    source: &SourceBuffer,
    document: &Document,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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
            emit_yaml_rendered_scalar_plan(
                out,
                source,
                scalar,
                node,
                state,
                options,
                YamlScalarEmitPosition {
                    body_indent: None,
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
                None,
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

#[derive(Clone, Copy)]
pub(super) struct YamlScalarEmitPosition {
    pub(super) body_indent: Option<usize>,
    pub(super) root_line_start: bool,
}

pub(super) fn emit_yaml_rendered_scalar_plan(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    state: &crate::core::directives::DirectiveState,
    options: FormatOptions,
    position: YamlScalarEmitPosition,
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
            if let Some(output) = render_inline_markdown_scalar(
                source,
                scalar,
                node,
                state,
                options,
                position.body_indent,
            ) {
                out.push_str(&output);
            }
        }
        YamlEmitPlan::Rendered(YamlRenderedKind::Scalar) => {
            emit_yaml_scalar_plan_output_into(
                out,
                source,
                scalar,
                node,
                options,
                position.body_indent,
                position.root_line_start,
            );
        }
        _ => unreachable!("non-scalar YAML render plan used for scalar output"),
    }
}

fn emit_yaml_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    body_indent: Option<usize>,
    root_line_start: bool,
) {
    if let Some(body) = scalar.body {
        output.push_str(&render_yaml_block_scalar_value_header(source, scalar));
        let body_indent = body_indent
            .map(|body_indent| block_scalar_output_body_indent(scalar, body_indent, options));
        if let Some(rewrapped) =
            render_rewrapped_folded_block_scalar(source, scalar, node, options, body_indent)
        {
            output.push_str(&rewrapped);
        } else if let Some(body_indent) = body_indent
            && let Some(source_indent) = explicit_block_scalar_body_indent(scalar)
                .or_else(|| block_scalar_body_indent(source.slice(body)))
        {
            if source_indent == body_indent {
                output.push_str(&normalize_nontrailing_block_scalar_blank_lines(
                    source.slice(body),
                    source_indent,
                ));
            } else {
                output.push_str(&reindent_block_lines(
                    source.slice(body),
                    source_indent,
                    body_indent,
                ));
            }
        } else {
            output.push_str(source.slice(body));
        }
        return;
    }
    if scalar.value.is_empty() && scalar_has_properties(scalar) {
        let (body, newline) = strip_newline(source.slice(node.span));
        output.push_str(body.trim_ascii_end());
        output.push_str(newline);
        return;
    }
    if emit_non_string_plain_scalar_plan_output_into(output, source, scalar, node).is_some() {
        return;
    }
    // A new literal header follows the emitted prefix, not the source indentation.
    let literal_body_indent =
        body_indent.unwrap_or_else(|| yaml_output_line_indent(output) + options.indent_width);
    if let Some(rendered) =
        render_quoted_literal_scalar(source, scalar, node, options, literal_body_indent)
    {
        output.push_str(&rendered);
        return;
    }
    if let Some(rendered) = render_folded_prose_scalar(source, scalar, node, options, body_indent) {
        output.push_str(&rendered);
        return;
    }
    if emit_plain_scalar_plan_output_into(output, source, scalar, node, body_indent).is_some() {
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
    if let Some(mut normalized) = normalize_quoted_string_scalar(source, scalar, false) {
        if root_line_start && document_marker(&normalized) {
            normalized = quote_yaml_string(&normalized, scalar.style);
        }
        output.push_str(&normalized);
        emit_inline_comment(output, source, scalar.trailing_comment);
        output.push_str(line_ending_for_span(source, node.span));
        return;
    }
    if let Some(rendered) = render_unsafe_plain_string_scalar(source, scalar, node) {
        output.push_str(&rendered);
        return;
    }
    let raw = source.slice(scalar.value).trim_ascii();
    let continuation_indent_delta = if raw.contains(['\r', '\n']) {
        yaml_scalar_continuation_indent_delta(output, source, scalar, body_indent)
    } else {
        0
    };
    output.push_str(&reindent_yaml_scalar_continuations(
        raw,
        continuation_indent_delta,
    ));
    emit_inline_comment(output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
}

fn emit_plain_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    body_indent: Option<usize>,
) -> Option<()> {
    if scalar.body.is_some()
        || scalar.header.is_some()
        || scalar.style != YamlScalarStyle::Plain
        || scalar.tag.is_some()
        || scalar.anchor.is_some()
    {
        return None;
    }
    let raw = source.slice(scalar.value).trim_ascii();
    match scalar.semantic {
        YamlScalarSemantic::Boolean => {
            let normalized = match raw {
                "true" | "True" | "TRUE" => "true",
                "false" | "False" | "FALSE" => "false",
                _ => return None,
            };
            output.push_str(normalized);
        }
        YamlScalarSemantic::Null => output.push_str(normalize_yaml_null(raw)?),
        _ => {
            if let Some(rendered) = render_unsafe_plain_string_scalar(source, scalar, node) {
                output.push_str(&rendered);
                return Some(());
            }
            let continuation_indent_delta = if raw.contains(['\r', '\n']) {
                yaml_scalar_continuation_indent_delta(output, source, scalar, body_indent)
            } else {
                0
            };
            output.push_str(&reindent_yaml_scalar_continuations(
                raw,
                continuation_indent_delta,
            ));
        }
    }
    emit_inline_comment(output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
    Some(())
}

fn yaml_scalar_continuation_indent_delta(
    output: &str,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    body_indent: Option<usize>,
) -> isize {
    let source_line = source.line_at_byte(scalar.value.start());
    let source_line_info = source.lines[source_line];
    let source_indent = indentation(source.line_text(source_line));
    let value_column = scalar
        .value
        .start()
        .saturating_sub(source_line_info.text.start());
    let output_indent = if value_column == source_indent {
        body_indent.unwrap_or_else(|| yaml_output_line_indent(output))
    } else {
        yaml_output_line_indent(output)
    };
    output_indent as isize - source_indent as isize
}

fn yaml_output_line_indent(output: &str) -> usize {
    let line_start = output
        .rfind(['\r', '\n'])
        .map(|index| index + 1)
        .unwrap_or(0);
    output[line_start..]
        .bytes()
        .take_while(|byte| *byte == b' ')
        .count()
}

fn reindent_yaml_scalar_continuations(raw: &str, indent_delta: isize) -> Cow<'_, str> {
    if indent_delta == 0 || !raw.contains(['\r', '\n']) {
        return Cow::Borrowed(raw);
    }

    let source = SourceBuffer::new(raw.to_owned());
    let mut output = String::with_capacity(raw.len());
    for (index, line) in source.lines.iter().enumerate() {
        let body = source.slice(line.text);
        if index == 0 || body.is_empty() {
            output.push_str(body);
        } else if indent_delta > 0 {
            emit_spaces(&mut output, indent_delta as usize);
            output.push_str(body);
        } else {
            let remove = (-indent_delta) as usize;
            let leading = body.bytes().take_while(|byte| *byte == b' ').count();
            if body.trim_ascii().is_empty() && leading < remove {
                output.push_str(body);
            } else {
                output.push_str(
                    body.strip_prefix(&" ".repeat(remove))
                        .expect("YAML scalar continuation indentation follows its parent"),
                );
            }
        }
        output.push_str(line.ending.as_str());
    }
    Cow::Owned(output)
}

fn emit_non_string_plain_scalar_plan_output_into(
    output: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
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
    let raw = source.slice(scalar.value).trim_ascii();
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => output.push_str("true"),
            "false" | "False" | "FALSE" => output.push_str("false"),
            _ => return None,
        },
        YamlScalarSemantic::Null => output.push_str(normalize_yaml_null(raw)?),
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
    scalar: &YamlScalar,
    node: &YamlAstNode,
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
    let content = source.slice(metadata.content).trim_ascii();
    if content.contains(['\n', '\r']) || block_plain_string_safe(content) {
        return None;
    }
    let quoted = quote_yaml_single_for_flow(content)?;
    let mut output = render_explicit_core_scalar(source, scalar, &quoted);
    emit_inline_comment(&mut output, source, scalar.trailing_comment);
    output.push_str(line_ending_for_span(source, node.span));
    Some(output)
}

fn normalize_quoted_string_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
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
    let raw = source.slice(scalar.value).trim_ascii();
    if !flow_context && raw.contains(['\n', '\r']) {
        return None;
    }
    let decoded = if let Some(decoded) = simple_quoted_scalar_inner(raw) {
        Cow::Borrowed(decoded)
    } else {
        Cow::Owned(decode_quoted_scalar(raw)?)
    };
    if plain_string_safe(&decoded, flow_context) {
        Some(decoded.into_owned())
    } else {
        Some(quote_yaml_string(&decoded, scalar.style))
    }
}

fn render_quoted_literal_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    body_indent: usize,
) -> Option<String> {
    let newline = line_ending_or_default(source, node.span, options);
    let final_newline = !line_ending_for_span(source, node.span).is_empty();
    render_quoted_literal_scalar_with_layout(
        source,
        scalar,
        options,
        body_indent,
        newline,
        final_newline,
    )
}

pub(super) fn render_quoted_literal_scalar_with_layout(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    options: FormatOptions,
    body_indent: usize,
    newline: &str,
    final_newline: bool,
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
    let raw = source.slice(metadata.content).trim_ascii();
    let decoded = decode_quoted_scalar(raw)?;
    if decoded == "\n" {
        return None;
    }
    let needs_indent_indicator = decoded
        .split('\n')
        .find(|line| !line.is_empty())
        .is_some_and(|line| line.starts_with(' '));
    if needs_indent_indicator && !(1..=9).contains(&options.indent_width) {
        return None;
    }
    if !decoded.contains('\n')
        && (plain_string_safe(&decoded, false)
            || !decoded.contains('\'')
            || !decoded.contains(['"', '\\']))
    {
        return None;
    }
    if decoded.contains('\r') {
        return None;
    }
    if decoded
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
    {
        return None;
    }

    let indent = " ".repeat(body_indent);

    let mut out = String::new();
    let trailing_newlines = decoded
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\n')
        .count();
    let chomp = match trailing_newlines {
        0 => "-",
        1 => "",
        _ => "+",
    };
    let marker = if needs_indent_indicator {
        Cow::Owned(format!("|{}{chomp}", options.indent_width))
    } else {
        Cow::Borrowed(match trailing_newlines {
            0 => "|-",
            1 => "|",
            _ => "|+",
        })
    };
    out.push_str(&render_explicit_core_scalar(source, scalar, &marker));
    out.push_str(newline);

    if trailing_newlines == 0 {
        let lines = decoded.split('\n').collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            if !line.is_empty() {
                out.push_str(&indent);
                out.push_str(line);
            }
            if index + 1 < lines.len() || final_newline {
                out.push_str(newline);
            }
        }
    } else {
        let body = &decoded[..decoded.len() - trailing_newlines];
        for line in body.split('\n') {
            if !line.is_empty() {
                out.push_str(&indent);
                out.push_str(line);
            }
            out.push_str(newline);
        }
        for _ in 1..trailing_newlines {
            out.push_str(newline);
        }
    }
    Some(out)
}

pub(super) fn decode_quoted_scalar(raw: &str) -> Option<String> {
    yaml_scalar::decode_quoted(raw, DecodePolicy::Format)
}

fn render_folded_prose_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    body_indent: Option<usize>,
) -> Option<String> {
    let newline = line_ending_or_default(source, node.span, options);
    let final_newline = !line_ending_for_span(source, node.span).is_empty();
    render_folded_prose_scalar_with_layout(
        source,
        scalar,
        node,
        options,
        body_indent,
        newline,
        final_newline,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_folded_prose_scalar_with_layout(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    body_indent: Option<usize>,
    newline: &str,
    final_newline: bool,
) -> Option<String> {
    if scalar.tag.is_some_and(|tag| source.slice(tag) != "!!str")
        || scalar.anchor.is_some()
        || scalar.trailing_comment.is_some()
        || scalar.header.is_some()
    {
        return None;
    }
    let metadata = scalar_metadata(source, scalar.value);
    let raw = source.slice(metadata.content).trim_ascii();
    let prose: Cow<'_, str> = match scalar.style {
        YamlScalarStyle::Plain => {
            if scalar.semantic != YamlScalarSemantic::String {
                return None;
            }
            let decoded = yaml_scalar::decode_plain(raw);
            if !foldable_yaml_prose(&decoded) {
                return None;
            }
            decoded
        }
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            if let Some(decoded) = simple_quoted_scalar_inner(raw) {
                if !foldable_yaml_prose(decoded) {
                    return None;
                }
                Cow::Borrowed(decoded)
            } else {
                let decoded = decode_quoted_scalar(raw)?;
                if !foldable_yaml_prose(&decoded) {
                    return None;
                }
                Cow::Owned(decoded)
            }
        }
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => return None,
    };
    if prose.chars().count() <= options.prose_width {
        return None;
    }
    let lines = wrap_yaml_prose(&prose, options.prose_width.max(1));
    if lines.len() <= 1 {
        return None;
    }
    let indent = body_indent.unwrap_or_else(|| {
        let line_index = source.line_at_byte(node.span.start());
        indentation(source.line_text(line_index)) + options.indent_width
    });
    let indent = " ".repeat(indent);
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
    scalar: &YamlScalar,
    node: &YamlAstNode,
    options: FormatOptions,
    body_indent: Option<usize>,
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
    let indent = " ".repeat(body_indent.unwrap_or(prose.indent));
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
        if line_body.trim_ascii().is_empty() {
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
        if line_indent != indent {
            return None;
        }
        let content = &line_body[indent..];
        if content != content.trim_ascii() {
            return None;
        }
        parts.push(content);
    }
    let indent = indent?;
    let prose = parts.join(" ");
    (!prose.is_empty() && foldable_yaml_prose(&prose)).then_some(FoldedBlockProse {
        text: prose,
        indent,
        final_newline,
        trailing_blank_lines: pending_blank_lines,
    })
}

fn foldable_yaml_prose(source: &str) -> bool {
    !source.is_empty()
        && source == source.trim_ascii()
        && !source.contains(['\n', '\r', '\t'])
        && !source.chars().any(char::is_control)
}

fn wrap_yaml_prose(source: &str, width: usize) -> Vec<String> {
    let chars = source.char_indices().collect::<Vec<_>>();
    let mut breaks = safe_yaml_space_breaks(&chars).into_iter().peekable();
    let mut lines = Vec::new();
    let mut start_byte = 0;
    let mut start_char = 0;
    let width = width.max(1);

    while chars.len().saturating_sub(start_char) > width {
        let target_char = start_char + width;
        let Some(mut break_at) = breaks.next() else {
            break;
        };
        while breaks
            .peek()
            .is_some_and(|candidate| candidate.char_index <= target_char)
        {
            break_at = breaks.next().expect("peeked prose break");
        }

        lines.push(source[start_byte..break_at.byte].to_owned());
        start_byte = break_at.byte + 1;
        start_char = break_at.char_index + 1;
    }

    if start_byte < source.len() {
        lines.push(source[start_byte..].to_owned());
    }
    lines
}

#[derive(Clone, Copy)]
struct SafeYamlSpaceBreak {
    byte: usize,
    char_index: usize,
}

fn safe_yaml_space_breaks(chars: &[(usize, char)]) -> Vec<SafeYamlSpaceBreak> {
    chars
        .iter()
        .enumerate()
        .filter_map(|(index, (byte, ch))| {
            if *ch != ' ' || index == 0 || index + 1 == chars.len() {
                return None;
            }
            let previous = chars[index - 1].1;
            let next = chars[index + 1].1;
            if previous == ' ' || next == ' ' {
                return None;
            }
            Some(SafeYamlSpaceBreak {
                byte: *byte,
                char_index: index,
            })
        })
        .collect()
}

pub(super) fn emit_yaml_scalar_inline_into(
    out: &mut String,
    source: &SourceBuffer,
    scalar: &YamlScalar,
    flow_context: bool,
) -> Option<()> {
    let raw = source.slice(scalar.value).trim_ascii();
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
    if let Some(normalized) = normalize_quoted_string_scalar(source, scalar, flow_context) {
        out.push_str(&normalized);
        return Some(());
    }
    if flow_context && raw.contains(['\n', '\r']) {
        return None;
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
    scalar: &YamlScalar,
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
            out.push_str(normalize_yaml_null(raw)?);
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
    scalar: &YamlScalar,
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

pub(super) fn render_yaml_scalar_inline_width(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    flow_context: bool,
) -> Option<usize> {
    let raw = source.slice(scalar.value).trim_ascii();
    if let Some(width) = normalize_core_scalar_width(source, scalar, flow_context) {
        return Some(width);
    }
    if scalar.tag.is_some() || scalar.anchor.is_some() {
        return None;
    }
    if let Some(width) = normalize_quoted_string_scalar_width(source, scalar, flow_context) {
        return Some(width);
    }
    if flow_context && raw.contains(['\n', '\r']) {
        return None;
    }
    if !flow_context || scalar.style != YamlScalarStyle::Plain || flow_plain_scalar_safe(raw) {
        return Some(raw.chars().count());
    }
    Some(quote_yaml_single_for_flow_width(raw))
}

fn normalize_core_scalar(
    source: &SourceBuffer,
    scalar: &YamlScalar,
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
    let content = source.slice(metadata.content).trim_ascii();
    let tag = scalar.tag.map(|tag| source.slice(tag));
    let decoded = scalar_value_for_core_tag(content)?;
    match tag {
        Some("!!bool") => normalize_yaml_bool(&decoded)
            .map(|value| render_explicit_core_scalar(source, scalar, &value)),
        Some("!!null") => normalize_yaml_null(&decoded)
            .map(|value| render_explicit_core_scalar(source, scalar, value)),
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
    scalar: &YamlScalar,
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
    let content = source.slice(metadata.content).trim_ascii();
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
    let decoded = yaml_scalar::decode_plain(content);
    let content = decoded.as_ref();
    match tag {
        Some("!!bool") => normalized_yaml_bool_width(content)
            .map(|width| explicit_core_scalar_width(source, scalar, metadata, width)),
        Some("!!null") => normalize_yaml_null(content)
            .map(|value| explicit_core_scalar_width(source, scalar, metadata, value.len())),
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
    scalar: &YamlScalar,
    metadata: ScalarMetadata,
    tag: Option<&str>,
    decoded: &str,
    flow_context: bool,
) -> Option<usize> {
    let value_width = match tag {
        Some("!!bool") => normalized_yaml_bool_width(decoded)?,
        Some("!!null") => normalize_yaml_null(decoded)?.len(),
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

pub(super) fn simple_quoted_scalar_inner(raw: &str) -> Option<&str> {
    if let Some(inner) = raw
        .strip_prefix('\'')
        .and_then(|raw| raw.strip_suffix('\''))
    {
        return (!inner.contains(['\'', '\n', '\r'])).then_some(inner);
    }
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains(['\\', '\n', '\r'])).then_some(inner)
}

fn normalize_implicit_core_scalar(raw: &str, scalar: &YamlScalar) -> Option<String> {
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => Some("true".to_owned()),
            "false" | "False" | "FALSE" => Some("false".to_owned()),
            _ => None,
        },
        YamlScalarSemantic::Null => normalize_yaml_null(raw).map(ToOwned::to_owned),
        _ => None,
    }
}

fn normalize_implicit_core_scalar_width(raw: &str, scalar: &YamlScalar) -> Option<usize> {
    match scalar.semantic {
        YamlScalarSemantic::Boolean => match raw {
            "true" | "True" | "TRUE" => Some("true".len()),
            "false" | "False" | "FALSE" => Some("false".len()),
            _ => None,
        },
        YamlScalarSemantic::Null => normalize_yaml_null(raw).map(str::len),
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

fn normalize_yaml_null(value: &str) -> Option<&'static str> {
    match value {
        "~" => Some("~"),
        "" | "null" | "Null" | "NULL" => Some("null"),
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
    scalar: &YamlScalar,
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

fn render_explicit_core_scalar(source: &SourceBuffer, scalar: &YamlScalar, value: &str) -> String {
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
    scalar: &YamlScalar,
    metadata: ScalarMetadata,
) -> usize {
    let inline_prefix = source
        .slice(Span::new(scalar.value.start(), metadata.content.start))
        .trim_ascii();
    if !inline_prefix.is_empty()
        && [scalar.tag, scalar.anchor]
            .into_iter()
            .flatten()
            .all(|property| property.start() >= scalar.value.start())
    {
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

fn normalize_quoted_string_scalar_width(
    source: &SourceBuffer,
    scalar: &YamlScalar,
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
    let raw = source.slice(scalar.value).trim_ascii();
    if !flow_context && raw.contains(['\n', '\r']) {
        return None;
    }
    if let Some(decoded) = simple_quoted_scalar_inner(raw) {
        return Some(if plain_string_safe(decoded, flow_context) {
            decoded.chars().count()
        } else {
            quote_yaml_string_width(decoded, scalar.style)
        });
    }
    if let Some(state) = decoded_quoted_string_width_state(raw, flow_context)
        && let Some(width) = state.normalized_string_width(scalar.style)
    {
        return Some(width);
    }
    normalize_quoted_string_scalar(source, scalar, flow_context).map(|value| value.chars().count())
}

pub(super) fn scalar_property_prefix(
    source: &SourceBuffer,
    scalar: &YamlScalar,
    metadata: ScalarMetadata,
) -> String {
    let inline_prefix = source
        .slice(Span::new(scalar.value.start(), metadata.content.start))
        .trim_ascii();
    if !inline_prefix.is_empty()
        && [scalar.tag, scalar.anchor]
            .into_iter()
            .flatten()
            .all(|property| property.start() >= scalar.value.start())
    {
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
    if previous.is_whitespace() || yaml_scalar_character_requires_escape(previous) {
        return false;
    }
    for ch in chars {
        if yaml_scalar_character_requires_escape(ch)
            || matches!((previous, ch), (':', ' ') | (' ', '#'))
        {
            return false;
        }
        previous = ch;
    }
    !previous.is_whitespace() && previous != ':'
}

pub(super) fn flow_plain_scalar_safe(value: &str) -> bool {
    if value.is_empty() || plain_scalar_unsafe_start_value(value) {
        return false;
    }
    let mut previous = None;
    for ch in value.chars() {
        if previous.is_none() && ch.is_whitespace()
            || yaml_scalar_character_requires_escape(ch)
            || matches!(ch, ',' | '[' | ']' | '{' | '}' | '\'' | '"')
            || matches!((previous, ch), (Some(':'), ' ') | (Some(' '), '#'))
        {
            return false;
        }
        previous = Some(ch);
    }
    previous.is_some_and(|ch| !ch.is_whitespace() && ch != ':')
}

fn decoded_quoted_explicit_string_width(raw: &str, flow_context: bool) -> Option<usize> {
    decoded_quoted_string_width_state(raw, flow_context)?.explicit_string_width()
}

fn decoded_quoted_explicit_core_width(raw: &str, tag: Option<&str>) -> Option<usize> {
    let mut state = DecodedCoreScalarState::new();
    decode_quoted_chars(raw, DecodePolicy::Width, |ch| state.push(ch))?;
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
    decode_quoted_chars(raw, DecodePolicy::Width, |ch| state.push(ch))?;
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
            b"~" => Some("~".len()),
            b"null" | b"Null" | b"NULL" => Some("null".len()),
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
    requires_double_quote_escape: bool,
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
            requires_double_quote_escape: false,
            double_bytes: 2,
            double_chars: 2,
        }
    }

    fn push(&mut self, ch: char) {
        if self.first.is_none() {
            self.first = Some(ch);
            if matches!(ch, '-' | '?' | ':') || plain_scalar_unsafe_first_char(ch) {
                self.plain_unsafe = true;
            }
        } else if self.width == 1 && matches!(self.first, Some('-' | '?' | ':')) {
            self.plain_unsafe = ch.is_whitespace();
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
        if yaml_scalar_character_requires_escape(ch)
            || self.flow_context && matches!(ch, ',' | '[' | ']' | '{' | '}' | '\'' | '"')
            || matches!((self.previous, ch), (Some(':'), ' ') | (Some(' '), '#'))
        {
            self.plain_unsafe = true;
        }
        if yaml_scalar_character_requires_escape(ch) {
            self.requires_double_quote_escape = true;
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
            && self.last != Some(':')
        {
            Some(self.width)
        } else {
            None
        }
    }

    fn explicit_string_width(&self) -> Option<usize> {
        self.normalized_string_width(YamlScalarStyle::SingleQuoted)
    }

    fn normalized_string_width(&self, preferred_style: YamlScalarStyle) -> Option<usize> {
        if let Some(width) = self.plain_safe_width() {
            return Some(width);
        }
        if self.known_not_plain_string_safe() {
            return Some(self.quoted_width(preferred_style));
        }
        None
    }

    fn known_not_plain_string_safe(&self) -> bool {
        self.width == 0
            || self.plain_unsafe
            || matches!(self.first, Some(ch) if ch.is_whitespace())
            || matches!(self.last, Some(ch) if ch.is_whitespace())
            || self.last == Some(':')
            || self.non_string_core_semantic().is_some()
    }

    fn non_string_core_semantic(&self) -> Option<YamlScalarSemantic> {
        non_string_core_semantic_bytes(self.core_value()?)
    }

    fn core_value(&self) -> Option<&[u8]> {
        self.core_valid.then_some(&self.core_bytes[..self.core_len])
    }

    fn quoted_width(&self, preferred_style: YamlScalarStyle) -> usize {
        if self.requires_double_quote_escape {
            return self.double_chars;
        }
        let single_bytes = self.bytes + 2 + self.single_quote_count;
        let single_chars = self.width + 2 + self.single_quote_count;
        if self.double_bytes < single_bytes {
            self.double_chars
        } else if single_bytes < self.double_bytes
            || preferred_style == YamlScalarStyle::SingleQuoted
        {
            single_chars
        } else {
            self.double_chars
        }
    }
}

// Include every character accepted by the null/bool/int/float recognizers,
// including radix prefixes and hexadecimal digits. Otherwise an escaped
// numeric-looking string can be assigned a width that omits its quotes.
fn yaml_core_non_string_semantic_char(ch: char) -> bool {
    ch.is_ascii_hexdigit()
        || matches!(
            ch,
            '+' | '-'
                | '_'
                | '.'
                | '~'
                | 'i'
                | 'I'
                | 'l'
                | 'L'
                | 'n'
                | 'N'
                | 'o'
                | 'O'
                | 'r'
                | 'R'
                | 's'
                | 'S'
                | 't'
                | 'T'
                | 'u'
                | 'U'
                | 'x'
                | 'X'
        )
}

fn plain_scalar_unsafe_start_value(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes.first().copied() {
        Some(b'-' | b'?' | b':') => bytes.get(1).is_none_or(u8::is_ascii_whitespace),
        Some(byte) => plain_scalar_unsafe_start_byte(byte),
        None => true,
    }
}

fn plain_scalar_unsafe_first_char(ch: char) -> bool {
    ch.is_ascii() && plain_scalar_unsafe_start_byte(ch as u8)
}

fn yaml_scalar_character_requires_escape(ch: char) -> bool {
    ch.is_control() || matches!(ch, '\u{feff}' | '\u{fffe}' | '\u{ffff}')
}

fn plain_scalar_unsafe_start_byte(byte: u8) -> bool {
    matches!(
        byte,
        b',' | b'['
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

fn double_quote_char_metrics(ch: char) -> (usize, usize) {
    match ch {
        '\0' | '\u{0007}' | '\u{0008}' | '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r'
        | '\u{001b}' | '"' | '\\' => (2, 2),
        '\u{feff}' | '\u{fffe}' | '\u{ffff}' => (6, 6),
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

pub(super) fn quote_yaml_single_for_flow(value: &str) -> Option<String> {
    Some(quote_yaml_string(value, YamlScalarStyle::SingleQuoted))
}

pub(super) fn quote_yaml_single_for_flow_width(value: &str) -> usize {
    quote_yaml_string_width(value, YamlScalarStyle::SingleQuoted)
}

fn quote_yaml_string(value: &str, preferred_style: YamlScalarStyle) -> String {
    if value.chars().any(yaml_scalar_character_requires_escape) {
        return quote_yaml_double_for_flow(value);
    }
    let single = format!("'{}'", value.replace('\'', "''"));
    let double = quote_yaml_double_for_flow(value);
    if double.len() < single.len() {
        double
    } else if single.len() < double.len() || preferred_style == YamlScalarStyle::SingleQuoted {
        single
    } else {
        double
    }
}

fn quote_yaml_string_width(value: &str, preferred_style: YamlScalarStyle) -> usize {
    let single_bytes = value.len() + 2 + value.matches('\'').count();
    let single_chars = value.chars().count() + 2 + value.matches('\'').count();
    if value.chars().any(yaml_scalar_character_requires_escape) {
        let (_, double_chars) = quote_yaml_double_for_flow_metrics(value);
        return double_chars;
    }
    let (double_bytes, double_chars) = quote_yaml_double_for_flow_metrics(value);
    if double_bytes < single_bytes {
        double_chars
    } else if single_bytes < double_bytes || preferred_style == YamlScalarStyle::SingleQuoted {
        single_chars
    } else {
        double_chars
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
            '\u{feff}' | '\u{fffe}' | '\u{ffff}' => {
                out.push_str(&format!("\\u{:04X}", ch as u32));
            }
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
            '\u{feff}' | '\u{fffe}' | '\u{ffff}' => {
                bytes += 6;
                chars += 6;
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

pub(super) fn scalar_semantic_with_tag(value: &str, tag: Option<&str>) -> YamlScalarSemantic {
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

pub(super) fn scalar_semantic_with_tag_and_style(
    value: &str,
    tag: Option<&str>,
    style: YamlScalarStyle,
) -> YamlScalarSemantic {
    if tag.is_some() {
        scalar_semantic_with_tag(value, tag)
    } else {
        scalar_semantic_for_style(value.trim_ascii(), style)
    }
}

pub(super) fn empty_scalar_semantic_with_tag(tag: Option<&str>) -> YamlScalarSemantic {
    match tag {
        Some("!!str") => YamlScalarSemantic::String,
        Some("!!null") | None => YamlScalarSemantic::Null,
        Some(_) => YamlScalarSemantic::Unknown,
    }
}

fn scalar_value_for_core_tag(value: &str) -> Option<String> {
    let trimmed = value.trim_ascii();
    match scalar_style(trimmed) {
        YamlScalarStyle::SingleQuoted | YamlScalarStyle::DoubleQuoted => {
            decode_quoted_scalar(trimmed)
        }
        YamlScalarStyle::Plain => Some(yaml_scalar::decode_plain(trimmed).into_owned()),
        YamlScalarStyle::LiteralBlock | YamlScalarStyle::FoldedBlock => None,
    }
}

fn scalar_semantic(value: &str) -> YamlScalarSemantic {
    let trimmed = value.trim_ascii();
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
