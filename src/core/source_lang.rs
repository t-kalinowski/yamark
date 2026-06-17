use crate::config::Config;
use crate::core::directives::{
    Directive, DirectiveDelta, DirectiveEngine, DirectiveState, Scope, TemplateDelimiter,
    contains_markdown_template_span, file_scope_delta, parse_hash_directive,
    parse_hash_directive_checked,
};
use crate::core::document::{
    Document, DocumentKind, EmitPlan, FormatOptions, MarkdownWrap, Node, NodeKind, SourceNodeKind,
    SourceText,
};
use crate::core::source::{LineEnding, SourceBuffer, SourceSpan, Span};
use crate::diagnostic::Result;
use unicode_width::UnicodeWidthStr;

const NON_RAW_PYTHON_BACKSLASH_ERROR: &str =
    "non-raw Python Markdown strings must not contain backslashes";
const NON_RAW_R_BACKSLASH_ERROR: &str = "non-raw R Markdown strings must not contain backslashes";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    Python,
    R,
}

pub fn parse_source_language<'src>(
    source: &'src SourceBuffer,
    range: Span,
    language: SourceLanguage,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_source_language_with_mode(
        source,
        range,
        language,
        options,
        config,
        EmbeddedMarkdownParseMode::Concrete,
    )
}

pub(crate) fn parse_source_language_for_formatting<'src>(
    source: &'src SourceBuffer,
    range: Span,
    language: SourceLanguage,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_source_language_with_mode(
        source,
        range,
        language,
        options,
        config,
        EmbeddedMarkdownParseMode::SemanticOnly,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmbeddedMarkdownParseMode {
    Concrete,
    SemanticOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmbeddedMarkdownTemplateMode {
    Configured,
    PythonFString,
}

fn parse_source_language_with_mode<'src>(
    source: &'src SourceBuffer,
    range: Span,
    language: SourceLanguage,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
) -> Result<Document<'src>> {
    let kind = match language {
        SourceLanguage::Python => DocumentKind::Python,
        SourceLanguage::R => DocumentKind::R,
    };
    let mut doc = Document::new(kind, range);
    doc.options = options;
    let mut engine = DirectiveEngine::new_with_template_delimiters(
        config.embedded_markdown_template_delimiters.clone(),
    );
    let start = source
        .lines
        .partition_point(|line| line.full.end() <= range.start);
    let mut i = start;
    let end = source
        .lines
        .partition_point(|line| line.full.start() < range.end);
    let mut comment_target_allowed = true;

    while i < end {
        let line = source.lines[i];
        let text = source.line_text(i);

        if engine.formatting_disabled() {
            if source_on_directive_line(text) {
                let state = engine.state_for_node(&mut doc, false);
                doc.push_node(Node {
                    kind: NodeKind::Source(SourceNodeKind::Directive),
                    span: line.full.into(),
                    state,
                    emit: EmitPlan::Preserve,
                });
                engine.apply_directive(&mut doc, Directive::On);
                i += 1;
                continue;
            }

            let start = i;
            i = source_disabled_region_end(source, i, end, language);
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::Raw),
                span: Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                state,
                emit: EmitPlan::Preserve,
            });
            continue;
        }

        if let Some(unsupported) = find_unsupported_string_literal(source, i, end, language) {
            if let Some(message) = engine.pending_target_error() {
                return Err(source_error_at(source, unsupported.start, message));
            }
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::Raw),
                span: unsupported,
                state,
                emit: EmitPlan::Copy,
            });
            i = source.line_at_byte(unsupported.end.saturating_sub(1)) + 1;
            continue;
        }

        if let Some(directive) = parse_hash_directive_checked(text).map_err(|message| {
            source_error_at(
                source,
                line.text.start() + text.find('#').unwrap_or(0),
                message,
            )
        })? {
            let directive = infer_source_directive_scope(
                source, start, i, end, language, directive,
            )
            .map_err(|message| {
                source_error_at(
                    source,
                    line.text.start() + text.find('#').unwrap_or(0),
                    message,
                )
            })?;
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::Directive),
                span: line.full.into(),
                state,
                emit: EmitPlan::Preserve,
            });
            let delta = file_scope_delta(&directive);
            engine.apply_directive(&mut doc, directive);
            if doc.skip_file {
                return Ok(doc);
            }
            if let Some(delta) = delta {
                patch_source_nodes_after_file_scope_delta(
                    source, &mut doc, options, config, &delta,
                )?;
            }
            comment_target_allowed = true;
            i += 1;
            continue;
        }

        if let Some(comment) = find_hashpipe_comment_block(source, i, end) {
            let state = engine.state_for_node(&mut doc, true);
            let state_value = doc.state(state).clone();
            let (prefix, nested) = plan_hashpipe_yaml(
                source,
                comment,
                state_value.yaml_options(options),
                config,
                mode,
            )?;
            let nested = doc.push_nested(nested);
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::Comment),
                span: comment,
                state,
                emit: EmitPlan::EmbeddedYamlComment { prefix, nested },
            });
            i = source.line_at_byte(comment.end.saturating_sub(1)) + 1;
            continue;
        }

        if let Some(comment) = find_comment_block(source, i, end) {
            let state = engine.state_for_node(&mut doc, comment_target_allowed);
            let state_value = doc.state(state).clone();
            let emit = if state_value.markdown_target {
                let (prefix, nested) = plan_comment_markdown(
                    source,
                    comment,
                    state_value.markdown_options(options),
                    config,
                    mode,
                )?;
                let nested = doc.push_nested(nested);
                EmitPlan::EmbeddedMarkdownComment { prefix, nested }
            } else {
                EmitPlan::Copy
            };
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::Comment),
                span: comment,
                state,
                emit,
            });
            i = source.line_at_byte(comment.end.saturating_sub(1)) + 1;
            continue;
        }

        if let Some(literal) = find_markdown_string_candidate(source, i, end, language) {
            comment_target_allowed = true;
            let state = engine.state_for_node(&mut doc, true);
            let state_value = doc.state(state).clone();
            let emit = if state_value.markdown_target {
                validate_markdown_literal_target(source, literal)?;
                let (nested, indent, closing_indent) = plan_string_markdown(
                    source,
                    literal,
                    state_value.markdown_options(options),
                    config,
                    mode,
                )?;
                let nested = doc.push_nested(nested);
                EmitPlan::EmbeddedMarkdownString {
                    opening: literal.opening,
                    body: literal.body,
                    closing: literal.closing,
                    nested,
                    indent,
                    closing_indent,
                }
            } else if let Some(name) = state_value.embedded_formatter {
                EmitPlan::ExternalPlugin {
                    name: name.into_boxed_str(),
                    body: literal.body,
                    normalized_opening: None,
                    fence_safety: None,
                }
            } else {
                EmitPlan::Copy
            };
            doc.push_node(Node {
                kind: NodeKind::Source(SourceNodeKind::StringLiteral),
                span: literal.full,
                state,
                emit,
            });
            i = source.line_at_byte(literal.full.end().saturating_sub(1)) + 1;
            continue;
        }

        if !text.trim().is_empty() && engine.has_pending_target_directives() {
            comment_target_allowed = false;
        }

        i += 1;
    }

    if let Some(message) = engine.pending_target_error() {
        return Err(source_error_at(
            source,
            range.end.saturating_sub(1),
            message,
        ));
    }

    plan_source_template_preservation(source, &mut doc);
    Ok(doc)
}

fn source_on_directive_line(text: &str) -> bool {
    matches!(parse_hash_directive(text), Some(Directive::On))
}

fn source_disabled_region_end(
    source: &SourceBuffer,
    mut i: usize,
    end: usize,
    language: SourceLanguage,
) -> usize {
    while i < end && !source_on_directive_line(source.line_text(i)) {
        if let Some(unsupported) = find_unsupported_string_literal(source, i, end, language) {
            i = source.line_at_byte(unsupported.end.saturating_sub(1)) + 1;
            continue;
        }
        if let Some(literal) = find_markdown_string_candidate(source, i, end, language) {
            i = source.line_at_byte(literal.full.end().saturating_sub(1)) + 1;
            continue;
        }
        i += 1;
    }
    i
}

fn infer_source_directive_scope(
    source: &SourceBuffer,
    start: usize,
    line: usize,
    end: usize,
    language: SourceLanguage,
    directive: Directive,
) -> std::result::Result<Directive, &'static str> {
    let Directive::Template { scope, delimiter } = directive else {
        return Ok(directive);
    };
    if scope != Scope::Next || source.line_text(line).contains("scope=") {
        return Ok(Directive::Template { scope, delimiter });
    }
    if source_directive_is_isolated(source, start, line, end) {
        Ok(Directive::Template {
            scope: Scope::FromHere,
            delimiter,
        })
    } else if line + 1 < end && source_line_starts_target(source, line + 1, end, language) {
        Ok(Directive::Template { scope, delimiter })
    } else {
        Err("fmt: template.delimiters needs explicit scope")
    }
}

fn source_directive_is_isolated(
    source: &SourceBuffer,
    start: usize,
    line: usize,
    end: usize,
) -> bool {
    let before_blank = line == start || source.line_text(line - 1).trim().is_empty();
    let after_blank = line + 1 >= end || source.line_text(line + 1).trim().is_empty();
    before_blank && after_blank
}

fn source_line_starts_target(
    source: &SourceBuffer,
    line: usize,
    end: usize,
    language: SourceLanguage,
) -> bool {
    parse_hash_directive_checked(source.line_text(line))
        .ok()
        .flatten()
        .is_none()
        && (find_comment_block(source, line, end).is_some()
            || find_markdown_string_candidate(source, line, end, language).is_some())
}

fn patch_source_nodes_after_file_scope_delta<'src>(
    source: &'src SourceBuffer,
    doc: &mut Document<'src>,
    options: FormatOptions,
    config: &Config,
    delta: &DirectiveDelta,
) -> Result<()> {
    for index in 0..doc.nodes.len() {
        let state = doc.state(doc.nodes[index].state).clone();
        match doc.nodes[index].emit.clone() {
            EmitPlan::EmbeddedMarkdownComment { nested, .. }
            | EmitPlan::EmbeddedMarkdownString { nested, .. } => {
                let nested_config = embedded_config_for_directive_state(config, &state);
                crate::core::markdown::apply_file_scope_delta_to_markdown_document(
                    source,
                    &mut doc.nested[nested],
                    delta,
                    state.markdown_options(options),
                    &nested_config,
                )?;
            }
            EmitPlan::EmbeddedYamlComment { .. } => {}
            _ => {}
        }
    }
    Ok(())
}

fn embedded_config_for_directive_state(config: &Config, state: &DirectiveState) -> Config {
    let mut config = config.clone();
    config.template_delimiters = state.template_delimiters.clone();
    config.embedded_markdown_template_delimiters = state.template_delimiters.clone();
    config
}

fn plan_source_template_preservation(source: &SourceBuffer, doc: &mut Document) {
    let preserve = doc
        .nodes
        .iter()
        .map(|node| {
            matches!(node.kind, NodeKind::Source(_))
                && !source_node_uses_known_template_delimiters(source, node)
                && contains_markdown_template_span(
                    source.slice(node.span),
                    &doc.state(node.state).template_delimiters,
                )
        })
        .collect::<Vec<_>>();
    for (node, preserve) in doc.nodes.iter_mut().zip(preserve) {
        if preserve {
            node.emit = EmitPlan::Preserve;
        }
    }
}

fn source_node_uses_known_template_delimiters(source: &SourceBuffer, node: &Node) -> bool {
    match &node.emit {
        EmitPlan::EmbeddedMarkdownString { opening, .. } => {
            python_markdown_string_is_f_string(source.slice(*opening))
        }
        _ => false,
    }
}

fn plan_comment_markdown<'src>(
    source: &'src SourceBuffer,
    span: Span,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
) -> Result<(SourceText<'src>, Document<'static>)> {
    let mut prefix = None::<(String, SourceText<'src>)>;
    let mut body = String::new();
    for line in source_lines(source.slice(span)) {
        let Some((line_prefix, content)) = split_comment_line(line.body) else {
            return Err(source_error_at(
                source,
                span.start,
                "fmt: markdown has no target",
            ));
        };
        let mut line_prefix = line_prefix.to_owned();
        let mut line_prefix_source = Some(SourceText::span(Span::new(
            span.start + line.body_start,
            span.start + line.body_start + line_prefix.len(),
        )));
        if content.is_empty() && !line_prefix.ends_with(' ') {
            if let Some((prefix, prefix_source)) = &prefix {
                line_prefix = prefix.clone();
                line_prefix_source = Some(prefix_source.clone());
            } else {
                line_prefix.push(' ');
                line_prefix_source = None;
            }
        }
        if let Some((prefix, _)) = &prefix {
            if prefix != &line_prefix {
                return Err(source_error_at(
                    source,
                    span.start,
                    "fmt: markdown has no target",
                ));
            }
        } else {
            let source_text =
                line_prefix_source.unwrap_or_else(|| SourceText::owned(line_prefix.clone()));
            prefix = Some((line_prefix, source_text));
        }
        body.push_str(content);
        body.push_str(line.newline);
    }
    let (prefix_text, prefix) = prefix.unwrap_or_else(|| {
        let text = "# ".to_owned();
        (text.clone(), SourceText::owned(text))
    });
    let options = markdown_options_with_reduced_width(options, display_width(&prefix_text));
    let nested = parse_generated_embedded_markdown(
        body,
        options,
        config,
        mode,
        EmbeddedMarkdownTemplateMode::Configured,
    )?;
    Ok((prefix, nested))
}

fn plan_hashpipe_yaml<'src>(
    source: &'src SourceBuffer,
    span: Span,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
) -> Result<(SourceText<'src>, Document<'static>)> {
    let mut prefix = None::<(String, SourceText<'src>)>;
    let mut body = String::new();
    for line in source_lines(source.slice(span)) {
        let Some((line_prefix, content)) = split_hashpipe_comment_line(line.body) else {
            return Err(source_error_at(
                source,
                span.start,
                "hashpipe YAML has no target",
            ));
        };
        let mut line_prefix = line_prefix.to_owned();
        let mut line_prefix_source = Some(SourceText::span(Span::new(
            span.start + line.body_start,
            span.start + line.body_start + line_prefix.len(),
        )));
        if content.is_empty() && !line_prefix.ends_with(' ') {
            if let Some((prefix, prefix_source)) = &prefix {
                line_prefix = prefix.clone();
                line_prefix_source = Some(prefix_source.clone());
            } else {
                line_prefix.push(' ');
                line_prefix_source = None;
            }
        }
        if let Some((prefix, _)) = &prefix {
            if prefix != &line_prefix {
                return Err(source_error_at(
                    source,
                    span.start,
                    "hashpipe YAML has no target",
                ));
            }
        } else {
            let source_text =
                line_prefix_source.unwrap_or_else(|| SourceText::owned(line_prefix.clone()));
            prefix = Some((line_prefix, source_text));
        }
        body.push_str(content);
        body.push_str(line.newline);
    }
    let (prefix_text, prefix) = prefix.unwrap_or_else(|| {
        let text = "#| ".to_owned();
        (text.clone(), SourceText::owned(text))
    });
    let options = yaml_options_with_reduced_width(options, display_width(&prefix_text));
    let nested = parse_generated_embedded_yaml(body, options, config, mode)?;
    Ok((prefix, nested))
}

fn plan_string_markdown<'src>(
    source: &'src SourceBuffer,
    literal: LiteralCandidate,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
) -> Result<(Document<'static>, SourceSpan<'src>, SourceSpan<'src>)> {
    let body_text = source.slice(literal.body);
    let body_text = body_without_delimiter_padding(body_text);
    let indent = common_body_indent(body_text);
    let indent_span = common_body_indent_span(source, literal.body, body_text, indent.len());
    let closing_indent = closing_delimiter_indent(source, literal);
    let dedented = dedent_body(body_text, &indent);
    let options = markdown_options_with_reduced_width(options, display_width(&indent));
    let nested = parse_generated_embedded_markdown(
        dedented,
        options,
        config,
        mode,
        literal.embedded_markdown_template_mode,
    )?;
    Ok((nested, indent_span, closing_indent))
}

fn parse_generated_embedded_markdown(
    body: String,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
    template_mode: EmbeddedMarkdownTemplateMode,
) -> Result<Document<'static>> {
    let generated_source = SourceBuffer::new(body);
    let mut embedded_config = config.clone();
    let template_delimiters = match template_mode {
        EmbeddedMarkdownTemplateMode::Configured => {
            config.embedded_markdown_template_delimiters.clone()
        }
        EmbeddedMarkdownTemplateMode::PythonFString => Vec::new(),
    };
    embedded_config.template_delimiters = template_delimiters.clone();
    embedded_config.embedded_markdown_template_delimiters = template_delimiters;
    if matches!(template_mode, EmbeddedMarkdownTemplateMode::PythonFString) {
        embedded_config.markdown_standalone_template_delimiters =
            python_f_string_template_delimiters();
    }
    let range = Span::new(0, generated_source.as_str().len());
    let nested = match mode {
        EmbeddedMarkdownParseMode::Concrete => crate::core::markdown::parse_markdown(
            &generated_source,
            range,
            options,
            &embedded_config,
        )?,
        EmbeddedMarkdownParseMode::SemanticOnly => {
            crate::core::markdown::parse_markdown_for_formatting(
                &generated_source,
                range,
                options,
                &embedded_config,
            )?
        }
    };
    let mut nested = nested.retag_source_lifetime();
    nested.source = Some(generated_source);
    Ok(nested)
}

fn python_f_string_template_delimiters() -> Vec<TemplateDelimiter> {
    vec![TemplateDelimiter {
        open: "{".to_owned(),
        close: "}".to_owned(),
    }]
}

fn parse_generated_embedded_yaml(
    body: String,
    options: FormatOptions,
    config: &Config,
    mode: EmbeddedMarkdownParseMode,
) -> Result<Document<'static>> {
    let generated_source = SourceBuffer::new(body);
    let range = Span::new(0, generated_source.as_str().len());
    let nested = match mode {
        EmbeddedMarkdownParseMode::Concrete => {
            crate::core::yaml::parse_yaml(&generated_source, range, options, config)?
        }
        EmbeddedMarkdownParseMode::SemanticOnly => {
            crate::core::yaml::parse_yaml_for_formatting(&generated_source, range, options, config)?
        }
    };
    let mut nested = nested.retag_source_lifetime();
    nested.source = Some(generated_source);
    Ok(nested)
}

pub(crate) fn restore_comment_prefix(source: &str, prefix: &str) -> String {
    source_lines(source)
        .into_iter()
        .map(|line| {
            let prefix = if line.body.is_empty() {
                prefix.trim_end()
            } else {
                prefix
            };
            format!("{prefix}{}{}", line.body, line.newline)
        })
        .collect()
}

pub(crate) fn validate_formatted_string_markdown(
    source: &SourceBuffer,
    body: Span,
    opening: &str,
    closing: &str,
    formatted: &str,
) -> Result<()> {
    let closing_delimiter = closing.trim();
    if !closing_delimiter.is_empty() && formatted.contains(closing_delimiter) {
        return Err(source_error_at(
            source,
            body.start,
            "formatted Markdown conflicts with string delimiter",
        ));
    }
    if let Some(message) = markdown_string_backslash_error(opening)
        && formatted.contains('\\')
    {
        return Err(source_error_at(source, body.start, message));
    }
    Ok(())
}

pub(crate) fn reindent_markdown_body(source: &str, indent: &str) -> String {
    reindent_body(source, indent)
}

fn markdown_options_with_reduced_width(
    mut options: FormatOptions,
    prefix_width: usize,
) -> FormatOptions {
    if matches!(options.markdown_wrap, MarkdownWrap::Column) {
        options.markdown_wrap_at_column = options
            .markdown_wrap_at_column
            .saturating_sub(prefix_width)
            .max(1);
    }
    options
}

fn yaml_options_with_reduced_width(
    mut options: FormatOptions,
    prefix_width: usize,
) -> FormatOptions {
    options.line_width = options.line_width.saturating_sub(prefix_width).max(1);
    options
}

fn common_body_indent(source: &str) -> String {
    let mut indents = source_lines(source)
        .into_iter()
        .filter(|line| !line.body.trim().is_empty())
        .map(|line| line_indent(line.body));
    let Some(mut common) = indents.next() else {
        return String::new();
    };
    for indent in indents {
        common = common_indent_prefix(&common, &indent);
        if common.is_empty() {
            break;
        }
    }
    common
}

fn common_body_indent_span<'src>(
    source: &'src SourceBuffer,
    body: Span,
    body_text: &str,
    indent_len: usize,
) -> SourceSpan<'src> {
    if indent_len == 0 {
        return SourceSpan::empty(body.start);
    }
    let source_body = source.slice(body);
    assert!(source_body.starts_with(body_text));
    let mut cursor = 0usize;
    for line in source_lines(body_text) {
        if !line.body.trim().is_empty() {
            let start = body.start + cursor;
            return SourceSpan::new(Span::new(start, start + indent_len));
        }
        cursor += line.body.len() + line.newline.len();
    }
    SourceSpan::empty(body.start)
}

fn closing_delimiter_indent<'src>(
    source: &'src SourceBuffer,
    literal: LiteralCandidate,
) -> SourceSpan<'src> {
    let closing_line = source.line_at_byte(literal.closing.start);
    SourceSpan::new(Span::new(
        source.lines[closing_line].text.start(),
        literal.closing.start,
    ))
}

fn body_without_delimiter_padding(source: &str) -> &str {
    let bytes = source.as_bytes();
    let mut start = 0usize;
    let mut last_content_end = 0usize;
    while start < source.len() {
        let mut body_end = start;
        while body_end < source.len() && !matches!(bytes[body_end], b'\r' | b'\n') {
            body_end += 1;
        }
        let full_end = if body_end == source.len() {
            body_end
        } else if bytes[body_end] == b'\r'
            && body_end + 1 < source.len()
            && bytes[body_end + 1] == b'\n'
        {
            body_end + 2
        } else {
            body_end + 1
        };
        if !source[start..body_end].trim().is_empty() {
            last_content_end = full_end;
        }
        start = full_end;
    }
    &source[..last_content_end]
}

fn line_indent(line: &str) -> String {
    line.char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(&line[..index]))
        .unwrap_or(line)
        .to_owned()
}

fn common_indent_prefix(left: &str, right: &str) -> String {
    let mut end = 0usize;
    for ((left_index, left_ch), (_right_index, right_ch)) in
        left.char_indices().zip(right.char_indices())
    {
        if left_ch != right_ch {
            break;
        }
        end = left_index + left_ch.len_utf8();
    }
    left[..end].to_owned()
}

fn dedent_body(source: &str, indent: &str) -> String {
    if indent.is_empty() {
        return source.to_owned();
    }
    source_lines(source)
        .into_iter()
        .map(|line| {
            let body = line.body.strip_prefix(indent).unwrap_or(line.body);
            format!("{}{}", body, line.newline)
        })
        .collect()
}

fn reindent_body(source: &str, indent: &str) -> String {
    if indent.is_empty() {
        return source.to_owned();
    }
    source_lines(source)
        .into_iter()
        .map(|line| {
            if line.body.is_empty() {
                line.newline.to_owned()
            } else {
                format!("{indent}{}{}", line.body, line.newline)
            }
        })
        .collect()
}

fn display_width(source: &str) -> usize {
    UnicodeWidthStr::width(source)
}

fn markdown_string_backslash_error(opening: &str) -> Option<&'static str> {
    if python_markdown_string_rejects_backslashes(opening) {
        return Some(NON_RAW_PYTHON_BACKSLASH_ERROR);
    }
    if r_standard_markdown_string_rejects_backslashes(opening) {
        return Some(NON_RAW_R_BACKSLASH_ERROR);
    }
    None
}

fn python_markdown_string_rejects_backslashes(opening: &str) -> bool {
    python_markdown_string_prefix(opening)
        .is_some_and(|prefix| python_string_prefix_supported(&prefix) && !prefix.contains('r'))
}

fn python_markdown_string_is_f_string(opening: &str) -> bool {
    python_markdown_string_prefix(opening)
        .is_some_and(|prefix| python_string_prefix_supported(&prefix) && prefix.contains('f'))
}

fn python_markdown_string_prefix(opening: &str) -> Option<String> {
    let delimiter_start = opening.rfind("\"\"\"").or_else(|| opening.rfind("'''"))?;
    let prefix_start = opening[..delimiter_start]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| (!ch.is_ascii_alphabetic()).then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    Some(opening[prefix_start..delimiter_start].to_ascii_lowercase())
}

fn r_standard_markdown_string_rejects_backslashes(opening: &str) -> bool {
    matches!(opening.trim(), "\"" | "'")
}

#[derive(Debug, Clone, Copy)]
struct LiteralCandidate {
    full: Span,
    opening: Span,
    body: Span,
    closing: Span,
    backslash_error: Option<&'static str>,
    embedded_markdown_template_mode: EmbeddedMarkdownTemplateMode,
}

fn validate_markdown_literal_target(
    source: &SourceBuffer,
    literal: LiteralCandidate,
) -> Result<()> {
    if let Some(message) = literal.backslash_error
        && source.slice(literal.body).contains('\\')
    {
        return Err(source_error_at(source, literal.body.start, message));
    }
    Ok(())
}

fn source_error_at(
    source: &SourceBuffer,
    byte: usize,
    message: impl Into<String>,
) -> crate::diagnostic::YamarkError {
    let (line, column) = source.line_column_at_byte(byte);
    crate::diagnostic::YamarkError::at(message, line, column)
}

fn find_markdown_string_candidate(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
    language: SourceLanguage,
) -> Option<LiteralCandidate> {
    match language {
        SourceLanguage::Python => find_python_triple_string(source, line_index, end_line),
        SourceLanguage::R => find_r_multiline_string(source, line_index, end_line),
    }
}

fn find_unsupported_string_literal(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
    language: SourceLanguage,
) -> Option<Span> {
    match language {
        SourceLanguage::Python => {
            find_python_unsupported_triple_string(source, line_index, end_line)
        }
        SourceLanguage::R => find_r_unsupported_raw_string(source, line_index, end_line),
    }
}

fn find_comment_block(source: &SourceBuffer, line_index: usize, end_line: usize) -> Option<Span> {
    if !is_comment_line(source.line_text(line_index)) {
        return None;
    }
    let hashpipe = is_hashpipe_comment_line(source.line_text(line_index));
    let start = source.lines[line_index].full.start();
    let mut i = line_index + 1;
    while i < end_line
        && is_comment_line(source.line_text(i))
        && is_hashpipe_comment_line(source.line_text(i)) == hashpipe
    {
        i += 1;
    }
    Some(Span::new(start, source.lines[i - 1].full.end()))
}

fn find_hashpipe_comment_block(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<Span> {
    if !is_hashpipe_comment_line(source.line_text(line_index)) {
        return None;
    }
    let start = source.lines[line_index].full.start();
    let mut i = line_index + 1;
    while i < end_line && is_hashpipe_comment_line(source.line_text(i)) {
        i += 1;
    }
    Some(Span::new(start, source.lines[i - 1].full.end()))
}

fn is_comment_line(text: &str) -> bool {
    text.trim_start().starts_with('#')
}

fn is_hashpipe_comment_line(text: &str) -> bool {
    split_hashpipe_comment_line(text).is_some()
}

fn split_comment_line(line: &str) -> Option<(&str, &str)> {
    let hash = line.find('#')?;
    let after_hash = hash + 1;
    if line.len() == after_hash {
        return Some((&line[..after_hash], ""));
    }
    if line.as_bytes().get(after_hash) == Some(&b' ') {
        Some((&line[..after_hash + 1], &line[after_hash + 1..]))
    } else {
        Some((&line[..after_hash], &line[after_hash..]))
    }
}

fn split_hashpipe_comment_line(line: &str) -> Option<(&str, &str)> {
    let hash = line.find('#')?;
    if !line[..hash].trim().is_empty() {
        return None;
    }
    let pipe = hash + 1;
    if line.as_bytes().get(pipe) != Some(&b'|') {
        return None;
    }
    let after_pipe = pipe + 1;
    if line.len() == after_pipe {
        return Some((&line[..after_pipe], ""));
    }
    if line.as_bytes().get(after_pipe) == Some(&b' ') {
        Some((&line[..after_pipe + 1], &line[after_pipe + 1..]))
    } else {
        Some((&line[..after_pipe], &line[after_pipe..]))
    }
}

fn find_python_triple_string(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<LiteralCandidate> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    let (local_start, delimiter) = find_python_triple_string_opening(line)?;
    let (prefix_start, prefix) = python_string_prefix(line, local_start)?;
    let open_start = line_start + prefix_start;
    let after_open = line_start + local_start + delimiter.len();
    let body_start = opening_line_body_start(source, line_index, after_open)?;
    let mut close_start = None;
    let mut scan = line_index + 1;
    while scan < end_line {
        let candidate = source.line_text(scan);
        let trimmed = candidate.trim_start();
        if trimmed.starts_with(delimiter) {
            let leading = candidate.len() - trimmed.len();
            let start = source.lines[scan].text.start() + leading;
            close_start = Some(start);
            break;
        }
        scan += 1;
    }
    let close_start = close_start?;
    let body = Span::new(body_start, close_start);
    let close_end = close_start + delimiter.len();
    Some(LiteralCandidate {
        full: Span::new(open_start, close_end),
        opening: Span::new(open_start, body_start),
        body,
        closing: Span::new(close_start, close_end),
        backslash_error: (!prefix.contains('r')).then_some(NON_RAW_PYTHON_BACKSLASH_ERROR),
        embedded_markdown_template_mode: if prefix.contains('f') {
            EmbeddedMarkdownTemplateMode::PythonFString
        } else {
            EmbeddedMarkdownTemplateMode::Configured
        },
    })
}

fn find_python_unsupported_triple_string(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<Span> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    let (local_start, delimiter) = find_python_triple_string_opening(line)?;
    let prefix = python_string_prefix_any(line, local_start);
    if python_string_prefix_supported(&prefix) {
        return None;
    }
    let open_start = line_start + local_start;
    let after_open = open_start + delimiter.len();
    let _body_start = opening_line_body_start(source, line_index, after_open)?;
    let mut scan = line_index + 1;
    while scan < end_line {
        let candidate = source.line_text(scan);
        let trimmed = candidate.trim_start();
        if trimmed.starts_with(delimiter) {
            let leading = candidate.len() - trimmed.len();
            let close_start = source.lines[scan].text.start() + leading;
            return Some(Span::new(open_start, close_start + delimiter.len()));
        }
        scan += 1;
    }
    None
}

fn find_python_triple_string_opening(line: &str) -> Option<(usize, &'static str)> {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < line.len() {
        if bytes[cursor] == b'#' {
            return None;
        }
        if line[cursor..].starts_with("\"\"\"") {
            return Some((cursor, "\"\"\""));
        }
        if line[cursor..].starts_with("'''") {
            return Some((cursor, "'''"));
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            cursor = skip_single_line_quoted_string(line, cursor, bytes[cursor]);
            continue;
        }
        let ch = line[cursor..].chars().next()?;
        cursor += ch.len_utf8();
    }
    None
}

fn skip_single_line_quoted_string(line: &str, start: usize, quote: u8) -> usize {
    let bytes = line.as_bytes();
    let mut escaped = false;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return cursor + 1;
        }
        cursor += 1;
    }
    line.len()
}

fn python_string_prefix(line: &str, delimiter_start: usize) -> Option<(usize, String)> {
    let prefix_start = python_string_prefix_start(line, delimiter_start);
    let prefix = line[prefix_start..delimiter_start].to_ascii_lowercase();
    if python_string_prefix_supported(&prefix) {
        Some((prefix_start, prefix))
    } else {
        None
    }
}

fn python_string_prefix_supported(prefix: &str) -> bool {
    matches!(prefix, "" | "r" | "f" | "rf" | "fr")
}

fn python_string_prefix_any(line: &str, delimiter_start: usize) -> String {
    let prefix_start = python_string_prefix_start(line, delimiter_start);
    line[prefix_start..delimiter_start].to_ascii_lowercase()
}

fn python_string_prefix_start(line: &str, delimiter_start: usize) -> usize {
    line[..delimiter_start]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| (!ch.is_ascii_alphabetic()).then_some(index + ch.len_utf8()))
        .unwrap_or(0)
}

fn opening_line_body_start(
    source: &SourceBuffer,
    line_index: usize,
    after_open: usize,
) -> Option<usize> {
    let line = source.lines[line_index];
    if line.ending == LineEnding::None {
        return None;
    }
    if after_open == line.text.end() {
        return Some(line.full.end());
    }
    let escaped_newline = after_open + 1 == line.text.end()
        && source
            .as_str()
            .as_bytes()
            .get(after_open)
            .is_some_and(|byte| *byte == b'\\');
    if escaped_newline {
        return Some(line.full.end());
    }
    if after_open < line.text.end() {
        let rest = source.slice(Span::new(after_open, line.text.end()));
        return Some(if rest.trim().is_empty() {
            line.full.end()
        } else {
            after_open
        });
    }
    None
}

#[derive(Debug, Clone, Copy)]
struct SourceLine<'a> {
    body: &'a str,
    newline: &'a str,
    body_start: usize,
}

fn source_lines(source: &str) -> Vec<SourceLine<'_>> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0usize;
    while start < source.len() {
        let mut end = start;
        while end < source.len() && !matches!(bytes[end], b'\r' | b'\n') {
            end += 1;
        }
        let (full_end, newline) = if end == source.len() {
            (end, "")
        } else if bytes[end] == b'\r' && end + 1 < source.len() && bytes[end + 1] == b'\n' {
            (end + 2, "\r\n")
        } else if bytes[end] == b'\r' {
            (end + 1, "\r")
        } else {
            (end + 1, "\n")
        };
        lines.push(SourceLine {
            body: &source[start..end],
            newline,
            body_start: start,
        });
        start = full_end;
    }
    lines
}

fn find_r_multiline_string(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<LiteralCandidate> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    if let Some(raw) = find_r_raw_string(source, line_index, end_line) {
        return Some(raw);
    }
    let quote_start = find_r_standard_string_opening(line)?;
    let quote = *line.as_bytes().get(quote_start)?;
    if !matches!(quote, b'\'' | b'\"') {
        return None;
    }
    let open_start = line_start + quote_start;
    let after_open = quote_start + 1;
    let rest = &line[after_open..];
    let body_start = if rest.trim().is_empty() {
        source.lines[line_index].full.end()
    } else if source.lines[line_index].ending == LineEnding::None {
        return None;
    } else {
        line_start + after_open
    };
    let mut i = line_index + 1;
    while i < end_line {
        let candidate = source.line_text(i);
        let trimmed = candidate.trim();
        if trimmed.as_bytes().first().copied() == Some(quote) {
            let close_start = source.lines[i].text.start() + candidate.find(trimmed)?;
            let close_end = close_start + 1;
            return Some(LiteralCandidate {
                full: Span::new(open_start, close_end),
                opening: Span::new(open_start, body_start),
                body: Span::new(body_start, close_start),
                closing: Span::new(close_start, close_end),
                backslash_error: Some(NON_RAW_R_BACKSLASH_ERROR),
                embedded_markdown_template_mode: EmbeddedMarkdownTemplateMode::Configured,
            });
        }
        i += 1;
    }
    None
}

fn find_r_raw_string(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<LiteralCandidate> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    let opening = find_r_raw_string_opening(line, 0)?;
    let rest = &line[opening.opening_end..];
    let body_start = if rest.trim().is_empty() {
        source.lines[line_index].full.end()
    } else if source.lines[line_index].ending == LineEnding::None {
        return None;
    } else {
        line_start + opening.opening_end
    };
    let open_start = line_start + opening.local_start;
    let mut i = line_index + 1;
    while i < end_line {
        let candidate = source.line_text(i);
        let trimmed = candidate.trim();
        if trimmed.starts_with(&opening.closing) {
            let close_start = source.lines[i].text.start() + candidate.find(trimmed)?;
            let close_end = close_start + opening.closing.len();
            return Some(LiteralCandidate {
                full: Span::new(open_start, close_end),
                opening: Span::new(open_start, body_start),
                body: Span::new(body_start, close_start),
                closing: Span::new(close_start, close_end),
                backslash_error: None,
                embedded_markdown_template_mode: EmbeddedMarkdownTemplateMode::Configured,
            });
        }
        i += 1;
    }
    None
}

fn find_r_unsupported_raw_string(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<Span> {
    if let Some(raw) = find_r_raw_literal_span(source, line_index, end_line) {
        if let Some(target) = find_r_raw_string(source, line_index, end_line)
            && target.full == raw
        {
            return None;
        }
        return Some(raw);
    }
    find_r_legacy_unsupported_raw_string_span(source, line_index, end_line)
}

#[derive(Debug, Clone)]
struct RRawStringOpening {
    local_start: usize,
    opening_end: usize,
    closing: String,
}

fn find_r_raw_string_opening(line: &str, mut search: usize) -> Option<RRawStringOpening> {
    let bytes = line.as_bytes();
    let comment_start = r_comment_start(line).unwrap_or(line.len());
    while search + 2 < bytes.len() {
        let local_start = bytes[search..]
            .iter()
            .position(|byte| matches!(byte, b'r' | b'R'))
            .map(|position| search + position)?;
        if local_start >= comment_start {
            return None;
        }
        let quote = bytes.get(local_start + 1).copied()?;
        if !matches!(quote, b'\'' | b'"') {
            search = local_start + 1;
            continue;
        }

        let mut cursor = local_start + 2;
        while bytes.get(cursor) == Some(&b'-') {
            cursor += 1;
        }
        let open_delimiter = bytes.get(cursor).copied()?;
        let close_delimiter = match open_delimiter {
            b'(' => ')',
            b'[' => ']',
            b'{' => '}',
            b'|' => '|',
            _ => {
                search = local_start + 1;
                continue;
            }
        };
        let dash_count = cursor - (local_start + 2);
        let mut closing = String::with_capacity(dash_count + 2);
        closing.push(close_delimiter);
        closing.extend(std::iter::repeat_n('-', dash_count));
        closing.push(quote as char);
        return Some(RRawStringOpening {
            local_start,
            opening_end: cursor + 1,
            closing,
        });
    }
    None
}

fn find_r_standard_string_opening(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < line.len() {
        if bytes[cursor] == b'#' {
            return None;
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            return Some(cursor);
        }
        let ch = line[cursor..].chars().next()?;
        cursor += ch.len_utf8();
    }
    None
}

fn r_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < line.len() {
        if bytes[cursor] == b'#' {
            return Some(cursor);
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            cursor = skip_single_line_quoted_string(line, cursor, bytes[cursor]);
            continue;
        }
        let ch = line[cursor..].chars().next()?;
        cursor += ch.len_utf8();
    }
    None
}

fn find_r_raw_literal_span(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<Span> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    let opening = find_r_raw_string_opening(line, 0)?;
    let open_start = line_start + opening.local_start;
    let mut i = line_index;
    while i < end_line {
        let search_start = if i == line_index {
            line_start + opening.opening_end
        } else {
            source.lines[i].text.start()
        };
        let line_end = source.lines[i].text.end();
        if search_start <= line_end {
            let search_span = Span::new(search_start, line_end);
            if let Some(relative) = source.slice(search_span).find(&opening.closing) {
                let close_start = search_start + relative;
                return Some(Span::new(open_start, close_start + opening.closing.len()));
            }
        }
        if source.lines[i].ending == LineEnding::None {
            break;
        }
        i += 1;
    }
    None
}

fn find_r_legacy_unsupported_raw_string_span(
    source: &SourceBuffer,
    line_index: usize,
    end_line: usize,
) -> Option<Span> {
    let line = source.line_text(line_index);
    let line_start = source.lines[line_index].text.start();
    let comment_start = r_comment_start(line).unwrap_or(line.len());
    let mut search = 0usize;
    while search < line.len() {
        let relative = line[search..]
            .find("r\"")
            .or_else(|| line[search..].find("R\""))?;
        let local_start = search + relative;
        if local_start >= comment_start {
            return None;
        }
        let delimiter_start = local_start + 2;
        let Some(open_paren_relative) = line[delimiter_start..].find('(') else {
            search = delimiter_start;
            continue;
        };
        let open_paren = delimiter_start + open_paren_relative;
        let delimiter = &line[delimiter_start..open_paren];
        if delimiter.is_empty() {
            search = open_paren + 1;
            continue;
        }
        if !line[open_paren + 1..].trim().is_empty() {
            search = open_paren + 1;
            continue;
        }
        let closing = format!("){delimiter}\"");
        let open_start = line_start + local_start;
        let mut i = line_index + 1;
        while i < end_line {
            let candidate = source.line_text(i);
            let trimmed = candidate.trim();
            if trimmed == closing {
                let close_start = source.lines[i].text.start() + candidate.find(trimmed)?;
                return Some(Span::new(open_start, close_start + closing.len()));
            }
            i += 1;
        }
        search = open_paren + 1;
    }
    None
}
