use crate::core::directives::contains_markdown_template_span;
use crate::core::document::{
    CodeFenceSafety, Document, EmitPlan, FormatOptions, MarkdownNodeKind, Node, NodeKind,
};
use crate::core::source::{SourceBuffer, Span};
use crate::diagnostic::{Result, YamarkError};
use crate::plugins::PluginRegistry;
use memchr::memchr2;

pub fn emit_document(
    source: &SourceBuffer,
    document: &Document,
    options: FormatOptions,
    plugins: &PluginRegistry,
) -> Result<String> {
    emit_document_with_normalization(source, document, options, plugins, false)
}

pub(crate) fn emit_markdown_document(
    source: &SourceBuffer,
    document: &Document,
    options: FormatOptions,
    plugins: &PluginRegistry,
) -> Result<String> {
    emit_document_with_normalization(source, document, options, plugins, true)
}

fn emit_document_with_normalization(
    source: &SourceBuffer,
    document: &Document,
    options: FormatOptions,
    plugins: &PluginRegistry,
    normalize_markdown_output: bool,
) -> Result<String> {
    emit_document_inner(
        source,
        document,
        options,
        plugins,
        EmitContext {
            blank_between_adjacent_divs: false,
        },
        normalize_markdown_output,
    )
}

#[derive(Debug, Clone, Copy)]
struct EmitContext {
    blank_between_adjacent_divs: bool,
}

fn emit_document_inner(
    source: &SourceBuffer,
    document: &Document,
    options: FormatOptions,
    plugins: &PluginRegistry,
    context: EmitContext,
    normalize_markdown_output: bool,
) -> Result<String> {
    let source = document.source.as_ref().unwrap_or(source);
    if document.skip_file {
        return Ok(source.slice(document.range).to_owned());
    }
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

    let mut out = EmitOutput::with_capacity(
        document.range.len(),
        normalize_markdown_output,
        options.default_line_ending,
    );
    let mut cursor = document.range.start;
    let mut previous_adjacent_div = false;
    let mut index = 0usize;
    while index < document.nodes.len() {
        let node = &document.nodes[index];
        if let Some(blank_run_end) = paragraph_separator_blank_run_end(document, index) {
            if cursor < node.span.start {
                out.push_str(source.slice(Span::new(cursor, node.span.start)));
            }
            let line_ending = line_ending_for_span(source, node.span);
            out.push_str(if line_ending.is_empty() {
                options.default_line_ending
            } else {
                line_ending
            });
            cursor = document.nodes[blank_run_end - 1].span.end;
            previous_adjacent_div = false;
            index = blank_run_end;
            continue;
        }

        if cursor < node.span.start {
            out.push_str(source.slice(Span::new(cursor, node.span.start)));
        } else if context.blank_between_adjacent_divs
            && previous_adjacent_div
            && matches!(node.emit, EmitPlan::MarkdownDiv { .. })
            && out.as_str().ends_with('\n')
            && !out.as_str().ends_with("\n\n")
            && !out.as_str().ends_with("\r\n\r\n")
        {
            let line_ending = line_ending_for_span(source, node.span);
            out.push_str(if line_ending.is_empty() {
                options.default_line_ending
            } else {
                line_ending
            });
        }
        let state = document.state(node.state);
        if state.preserve || matches!(node.emit, EmitPlan::Preserve) {
            out.push_str(source.slice(node.span));
            cursor = node.span.end;
            previous_adjacent_div = matches!(node.emit, EmitPlan::MarkdownDiv { .. });
            index += 1;
            continue;
        }
        if markdown_emit_should_preserve_template_span(source, node.span, state, &node.emit) {
            out.push_str(source.slice(node.span));
            cursor = node.span.end;
            previous_adjacent_div = matches!(node.emit, EmitPlan::MarkdownDiv { .. });
            index += 1;
            continue;
        }
        match &node.emit {
            EmitPlan::Copy | EmitPlan::Preserve => out.push_str(source.slice(node.span)),
            EmitPlan::MarkdownHeading {
                marker, content, ..
            } => out.push_str(&crate::core::markdown::render_markdown_heading(
                source,
                node.span,
                *marker,
                *content,
                state.markdown_options(options),
            )),
            EmitPlan::MarkdownSetextHeading { content, depth } => {
                out.push_str(&crate::core::markdown::render_markdown_setext_heading(
                    source,
                    node.span,
                    *content,
                    *depth,
                    state.markdown_options(options),
                ))
            }
            EmitPlan::MarkdownThematicBreak => {
                out.push_str(&crate::core::markdown::render_markdown_thematic_break(
                    source,
                    node.span,
                    state.markdown_options(options),
                ))
            }
            EmitPlan::MarkdownParagraph => {
                out.push_str(&crate::core::markdown::render_markdown_format(
                    source,
                    node.span,
                    state.markdown_options(options),
                    crate::core::markdown::MarkdownBlockFormatKind::Paragraph,
                ))
            }
            EmitPlan::MarkdownTable => {
                out.push_str(&crate::core::markdown::render_markdown_format(
                    source,
                    node.span,
                    state.markdown_options(options),
                    crate::core::markdown::MarkdownBlockFormatKind::Table,
                ))
            }
            EmitPlan::MarkdownPandocTable => {
                out.push_str(&crate::core::markdown::render_markdown_format(
                    source,
                    node.span,
                    state.markdown_options(options),
                    crate::core::markdown::MarkdownBlockFormatKind::PandocTable,
                ))
            }
            EmitPlan::MarkdownList => out.push_str(&crate::core::markdown::render_markdown_format(
                source,
                node.span,
                state.markdown_options(options),
                crate::core::markdown::MarkdownBlockFormatKind::List,
            )),
            EmitPlan::MarkdownDefinitionList => {
                out.push_str(&crate::core::markdown::render_markdown_format(
                    source,
                    node.span,
                    state.markdown_options(options),
                    crate::core::markdown::MarkdownBlockFormatKind::DefinitionList,
                ))
            }
            EmitPlan::MarkdownBlockquote => {
                out.push_str(&crate::core::markdown::render_markdown_format(
                    source,
                    node.span,
                    state.markdown_options(options),
                    crate::core::markdown::MarkdownBlockFormatKind::Blockquote,
                ))
            }
            EmitPlan::MarkdownFrontMatter {
                opening,
                closing,
                nested,
            } => {
                emit_front_matter_marker(&mut out, source, *opening, true);
                let mut nested_output = emit_document_inner(
                    source,
                    &document.nested[*nested],
                    state.markdown_options(options),
                    plugins,
                    context,
                    false,
                )?;
                if !nested_output.is_empty()
                    && !nested_output.ends_with('\n')
                    && !nested_output.ends_with('\r')
                {
                    nested_output.push_str(line_ending_for_span(source, *opening));
                }
                out.push_str(&nested_output);
                emit_front_matter_marker(&mut out, source, *closing, false);
            }
            EmitPlan::MarkdownCodeFence {
                opening,
                normalized_opening,
                closing,
                nested,
                safety,
                ..
            } => {
                emit_opening(&mut out, source, *opening, normalized_opening.as_deref());
                if let Some(nested) = nested {
                    let mut nested_output = emit_document_inner(
                        source,
                        &document.nested[*nested],
                        state.markdown_options(options),
                        plugins,
                        context,
                        false,
                    )?;
                    if !nested_output.is_empty()
                        && !nested_output.ends_with('\n')
                        && !nested_output.ends_with('\r')
                    {
                        nested_output.push_str(line_ending_for_span(source, *opening));
                    }
                    ensure_code_fence_safe(&nested_output, *safety, source, *opening)?;
                    out.push_str(&nested_output);
                } else {
                    out.push_str(source.slice(Span::new(opening.end, closing.start)));
                }
                out.push_str(source.slice(*closing));
            }
            EmitPlan::MarkdownDiv {
                opening,
                closing,
                nested,
            } => {
                out.push_str(source.slice(*opening));
                let mut nested_output = emit_document_inner(
                    source,
                    &document.nested[*nested],
                    state.markdown_options(options),
                    plugins,
                    EmitContext {
                        blank_between_adjacent_divs: true,
                    },
                    false,
                )?;
                if !nested_output.is_empty()
                    && !nested_output.ends_with('\n')
                    && !nested_output.ends_with('\r')
                {
                    nested_output.push_str(line_ending_for_span(source, *opening));
                }
                out.push_str(&nested_output);
                out.push_str(source.slice(*closing));
            }
            EmitPlan::MarkdownOpaque => out.push_str(source.slice(node.span)),
            EmitPlan::YamlDocument => {
                out.push_str(&crate::core::yaml::emit_yaml_document(
                    source, document, options, plugins,
                )?);
            }
            EmitPlan::EmbeddedMarkdownString {
                opening,
                body,
                closing,
                nested,
                indent,
                closing_indent,
            } => {
                out.push_str(source.slice(*opening));
                let nested_output = emit_document_inner(
                    source,
                    &document.nested[*nested],
                    state.markdown_options(options),
                    plugins,
                    context,
                    false,
                )?;
                crate::core::source_lang::validate_formatted_string_markdown(
                    source,
                    *body,
                    source.slice(*opening),
                    source.slice(*closing),
                    &nested_output,
                )?;
                out.push_str(&crate::core::source_lang::reindent_markdown_body(
                    &nested_output,
                    source.slice(*indent),
                ));
                out.push_str(source.slice(*closing_indent));
                out.push_str(source.slice(*closing));
            }
            EmitPlan::EmbeddedMarkdownComment { prefix, nested } => {
                let nested_output = emit_document_inner(
                    source,
                    &document.nested[*nested],
                    state.markdown_options(options),
                    plugins,
                    context,
                    false,
                )?;
                out.push_str(&crate::core::source_lang::restore_comment_prefix(
                    &nested_output,
                    prefix.as_str(source),
                ));
            }
            EmitPlan::EmbeddedYamlComment { prefix, nested } => {
                let nested_output = emit_document_inner(
                    source,
                    &document.nested[*nested],
                    state.yaml_options(options),
                    plugins,
                    context,
                    false,
                )?;
                out.push_str(&crate::core::source_lang::restore_comment_prefix(
                    &nested_output,
                    prefix.as_str(source),
                ));
            }
            EmitPlan::ExternalPlugin {
                name,
                body,
                string_indent,
                normalized_opening,
                fence_safety,
            } => {
                if options.skip_embedded_formatters {
                    emit_external_preserved(
                        &mut out,
                        source,
                        node.span,
                        *body,
                        normalized_opening.as_deref(),
                    );
                } else {
                    let body_text = source.slice(*body);
                    let (formatter_source, body_suffix, indent) =
                        if let Some(indent) = string_indent {
                            let formatter_source =
                                crate::core::source_lang::body_without_delimiter_padding(body_text);
                            let body_suffix = &body_text[formatter_source.len()..];
                            (formatter_source, body_suffix, indent.as_str(source))
                        } else {
                            (body_text, "", "")
                        };
                    let (preamble, formatter_input) = split_renderer_preamble(formatter_source);
                    let dedented;
                    let formatter_input = if string_indent.is_some() {
                        dedented = crate::core::source_lang::dedent_body(formatter_input, indent);
                        dedented.as_str()
                    } else {
                        formatter_input
                    };
                    let line = source.line_column_at_byte(body.start).0;
                    if let Some(mut formatted) =
                        plugins.run(name.as_ref(), formatter_input, line)?
                    {
                        append_trailing_line_ending_if_missing(
                            &mut formatted,
                            external_plugin_line_ending(source, node.span, *body, options),
                        );
                        if let Some(safety) = fence_safety {
                            ensure_code_fence_safe(&formatted, *safety, source, *body)?;
                        }
                        emit_opening(
                            &mut out,
                            source,
                            Span::new(node.span.start, body.start),
                            normalized_opening.as_deref(),
                        );
                        out.push_str(preamble);
                        if string_indent.is_some() {
                            formatted = crate::core::source_lang::reindent_markdown_body(
                                &formatted, indent,
                            );
                        }
                        out.push_str(&formatted);
                        out.push_str(body_suffix);
                        out.push_str(source.slice(Span::new(body.end, node.span.end)));
                    } else {
                        emit_external_preserved(
                            &mut out,
                            source,
                            node.span,
                            *body,
                            normalized_opening.as_deref(),
                        );
                    }
                }
            }
        }
        cursor = node.span.end;
        previous_adjacent_div = matches!(node.emit, EmitPlan::MarkdownDiv { .. });
        index += 1;
    }
    if cursor < document.range.end {
        out.push_str(source.slice(Span::new(cursor, document.range.end)));
    }
    Ok(out.finish())
}

struct EmitOutput {
    text: String,
    normalize_markdown: bool,
    final_line_ending: &'static str,
    line_trim_end: usize,
}

impl EmitOutput {
    fn with_capacity(
        capacity: usize,
        normalize_markdown: bool,
        final_line_ending: &'static str,
    ) -> Self {
        Self {
            text: String::with_capacity(capacity),
            normalize_markdown,
            final_line_ending,
            line_trim_end: 0,
        }
    }

    fn as_str(&self) -> &str {
        &self.text
    }

    fn push(&mut self, ch: char) {
        let mut buffer = [0u8; 4];
        self.push_str(ch.encode_utf8(&mut buffer));
    }

    fn push_str(&mut self, text: &str) {
        if !self.normalize_markdown {
            self.text.push_str(text);
            return;
        }
        self.push_markdown_normalized_str(text);
    }

    fn finish(mut self) -> String {
        if self.normalize_markdown
            && !self.text.is_empty()
            && !self.text.ends_with('\n')
            && !self.text.ends_with('\r')
        {
            self.text.truncate(self.line_trim_end);
            self.text.push_str(self.final_line_ending);
        }
        self.text
    }

    fn push_markdown_normalized_str(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let Some(mut cursor) = memchr2(b'\r', b'\n', bytes) else {
            self.push_line_body(text);
            return;
        };
        let mut segment_start = 0usize;
        loop {
            match bytes[cursor] {
                b'\r' => {
                    self.push_line_body(&text[segment_start..cursor]);
                    self.finish_line();
                    if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'\n' {
                        self.text.push_str("\r\n");
                        cursor += 2;
                    } else {
                        self.text.push('\r');
                        cursor += 1;
                    }
                    self.line_trim_end = self.text.len();
                    segment_start = cursor;
                }
                b'\n' => {
                    self.push_line_body(&text[segment_start..cursor]);
                    self.finish_line();
                    self.text.push('\n');
                    cursor += 1;
                    self.line_trim_end = self.text.len();
                    segment_start = cursor;
                }
                _ => unreachable!("memchr returned a non-line-break byte"),
            }
            let Some(offset) = memchr2(b'\r', b'\n', &bytes[segment_start..]) else {
                break;
            };
            cursor = segment_start + offset;
        }
        self.push_line_body(&text[segment_start..]);
    }

    fn push_line_body(&mut self, body: &str) {
        let start = self.text.len();
        self.text.push_str(body);
        let trimmed_len = body.trim_end_matches([' ', '\t']).len();
        if trimmed_len > 0 {
            self.line_trim_end = start + trimmed_len;
        }
    }

    fn finish_line(&mut self) {
        self.text.truncate(self.line_trim_end);
    }
}

fn paragraph_separator_blank_run_end(document: &Document, index: usize) -> Option<usize> {
    if !node_is_markdown_blank(document.nodes.get(index)?) {
        return None;
    }
    let before = document.nodes.get(index.checked_sub(1)?)?;
    if !node_is_emitted_markdown_paragraph(document, before) {
        return None;
    }

    let mut end = index;
    while document.nodes.get(end).is_some_and(node_is_markdown_blank) {
        end += 1;
    }
    let after = document.nodes.get(end)?;
    node_is_emitted_markdown_paragraph(document, after).then_some(end)
}

fn node_is_markdown_blank(node: &Node) -> bool {
    matches!(node.kind, NodeKind::Markdown(MarkdownNodeKind::Blank))
}

fn node_is_emitted_markdown_paragraph(document: &Document, node: &Node) -> bool {
    !document.state(node.state).preserve
        && matches!(node.kind, NodeKind::Markdown(MarkdownNodeKind::Paragraph))
        && matches!(node.emit, EmitPlan::MarkdownParagraph)
}

fn markdown_emit_should_preserve_template_span(
    source: &SourceBuffer,
    span: Span,
    state: &crate::core::directives::DirectiveState,
    emit: &EmitPlan,
) -> bool {
    matches!(
        emit,
        EmitPlan::MarkdownHeading { .. }
            | EmitPlan::MarkdownSetextHeading { .. }
            | EmitPlan::MarkdownParagraph
            | EmitPlan::MarkdownList
            | EmitPlan::MarkdownDefinitionList
            | EmitPlan::MarkdownBlockquote
    ) && contains_markdown_template_span(source.slice(span), &state.template_delimiters)
}

fn emit_opening(
    out: &mut EmitOutput,
    source: &SourceBuffer,
    opening: Span,
    normalized_opening: Option<&str>,
) {
    if let Some(normalized_opening) = normalized_opening {
        out.push_str(normalized_opening);
    } else {
        out.push_str(source.slice(opening));
    }
}

fn emit_front_matter_marker(
    out: &mut EmitOutput,
    source: &SourceBuffer,
    marker: Span,
    preserve_bom: bool,
) {
    let text = source.slice(marker);
    if preserve_bom && text.starts_with('\u{feff}') {
        out.push('\u{feff}');
    }
    out.push_str("---");
    out.push_str(line_ending_for_span(source, marker));
}

fn emit_external_preserved(
    out: &mut EmitOutput,
    source: &SourceBuffer,
    span: Span,
    _body: Span,
    _normalized_opening: Option<&str>,
) {
    out.push_str(source.slice(span));
}

pub(crate) fn split_renderer_preamble(source: &str) -> (&str, &str) {
    let mut split = 0usize;
    let bytes = source.as_bytes();
    while split < source.len() {
        let line_start = split;
        let mut body_end = line_start;
        while body_end < source.len() && !matches!(bytes[body_end], b'\r' | b'\n') {
            body_end += 1;
        }
        let line_end = if body_end == source.len() {
            body_end
        } else if bytes[body_end] == b'\r'
            && body_end + 1 < source.len()
            && bytes[body_end + 1] == b'\n'
        {
            body_end + 2
        } else {
            body_end + 1
        };
        let trimmed = source[line_start..body_end].trim_start();
        if trimmed.starts_with("#|") || trimmed.starts_with("#@") {
            split = line_end;
        } else {
            break;
        }
    }
    source.split_at(split)
}

fn external_plugin_line_ending(
    source: &SourceBuffer,
    node_span: Span,
    body: Span,
    options: FormatOptions,
) -> &'static str {
    let opening = Span::new(node_span.start, body.start);
    let line_ending = line_ending_for_span(source, opening);
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

fn ensure_code_fence_safe(
    formatted: &str,
    safety: CodeFenceSafety,
    source: &SourceBuffer,
    span: Span,
) -> Result<()> {
    if code_fence_output_contains_closing_line(formatted, safety) {
        return Err(unsupported_slice_error(
            source,
            span,
            "formatted code fence would contain closing fence",
        ));
    }
    Ok(())
}

fn code_fence_output_contains_closing_line(formatted: &str, safety: CodeFenceSafety) -> bool {
    let bytes = formatted.as_bytes();
    let mut line_start = 0usize;
    while line_start < formatted.len() {
        let mut line_end = line_start;
        while line_end < formatted.len() && !matches!(bytes[line_end], b'\r' | b'\n') {
            line_end += 1;
        }
        if closes_code_fence(&formatted[line_start..line_end], safety) {
            return true;
        }
        if line_end == formatted.len() {
            break;
        }
        line_start = if bytes[line_end] == b'\r'
            && line_end + 1 < formatted.len()
            && bytes[line_end + 1] == b'\n'
        {
            line_end + 2
        } else {
            line_end + 1
        };
    }
    false
}

fn closes_code_fence(line: &str, safety: CodeFenceSafety) -> bool {
    let line = line.trim_end_matches('\r');
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    if indent > 3 {
        return false;
    }
    let marker_len = line[indent..]
        .chars()
        .take_while(|ch| *ch == safety.marker)
        .count();
    marker_len >= safety.min_len && line[indent + marker_len..].trim().is_empty()
}

fn line_ending_for_span(source: &SourceBuffer, span: Span) -> &'static str {
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

pub fn unsupported_slice_error(source: &SourceBuffer, span: Span, message: &str) -> YamarkError {
    let (line, column) = source.line_column_at_byte(span.start);
    YamarkError::at(message, line, column)
}
