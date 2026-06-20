use crate::config::Config;
use crate::core::directives::{
    Directive, DirectiveDelta, DirectiveEngine, DirectiveState, Scope, StateId, TemplateDelimiter,
    contains_markdown_template_span, file_scope_delta, parse_markdown_html_directive,
    parse_markdown_html_directive_checked,
};
use crate::core::document::{
    CodeFenceSafety, Document, DocumentKind, EmitPlan, FormatOptions, MarkdownNodeKind,
    MarkdownWrap, Node, NodeKind,
};
use crate::core::markdown_marker::markdown_list_marker_len;
use crate::core::source::{SourceBuffer, Span};
use crate::core::yaml_model::{YamlAstKind, YamlDocumentAst, YamlNodeId, YamlScalar};
use crate::diagnostic::Result;

pub fn parse_markdown<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_markdown_with_mode(source, range, options, config, MarkdownParseMode::Concrete)
}

pub(crate) fn parse_markdown_for_formatting<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    parse_markdown_with_mode(
        source,
        range,
        options,
        config,
        MarkdownParseMode::SemanticOnly,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownParseMode {
    Concrete,
    SemanticOnly,
}

fn parse_markdown_with_mode<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
    mode: MarkdownParseMode,
) -> Result<Document<'src>> {
    let mut options = options;
    if !matches!(
        source.dominant_line_ending,
        crate::core::source::LineEnding::None
    ) {
        options.default_line_ending = source.dominant_line_ending.as_str();
    }
    let mut doc = Document::new(DocumentKind::Markdown, range);
    doc.options = options;
    let mut engine =
        DirectiveEngine::new_with_template_delimiters(config.template_delimiters.clone());
    let start_line = first_line_index(source, range);
    let mut i = start_line;
    let end_line = end_line_index(source, range);

    if i < end_line
        && front_matter_opening(source.line_text(i))
        && let Some(closing) = find_front_matter_closing(source, i + 1, end_line)
    {
        let opening = source.lines[i].full;
        let closing_span = source.lines[closing].full;
        let content = Span::new(opening.end(), source.lines[closing].full.start());
        let nested_document = parse_nested_yaml(source, content, options, config, mode)?;
        if nested_document.skip_file {
            doc.skip_file = true;
            return Ok(doc);
        }
        let markdown_delta = options
            .respect_frontmatter_markdown_options
            .then(|| front_matter_markdown_delta(source, &nested_document))
            .flatten();
        let nested = doc.push_nested(nested_document);
        let state = engine.state_for_node(&mut doc, true);
        doc.push_node(Node {
            kind: NodeKind::Markdown(MarkdownNodeKind::FrontMatter),
            span: Span::new(opening.start(), closing_span.end()),
            state,
            emit: EmitPlan::MarkdownFrontMatter {
                opening: opening.into(),
                closing: closing_span.into(),
                nested,
            },
        });
        if let Some(delta) = markdown_delta {
            engine.apply_directive(
                &mut doc,
                Directive::Markdown {
                    scope: Scope::FromHere,
                    delta,
                },
            );
        }
        i = closing + 1;
    }

    while i < end_line {
        let line = source.lines[i];
        let text = source.line_text(i);

        if engine.formatting_disabled() {
            if markdown_on_directive_line(text) {
                let state = engine.state_for_node(&mut doc, false);
                doc.push_node(Node {
                    kind: NodeKind::Markdown(MarkdownNodeKind::Directive),
                    span: line.full.into(),
                    state,
                    emit: EmitPlan::Preserve,
                });
                engine.apply_directive(&mut doc, Directive::On);
                i += 1;
                continue;
            }

            let start = i;
            i = markdown_disabled_region_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Raw),
                span: Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                state,
                emit: EmitPlan::Preserve,
            });
            continue;
        }

        if text.trim().is_empty() {
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Blank),
                span: line.full.into(),
                state,
                emit: EmitPlan::Copy,
            });
            i += 1;
            continue;
        }

        if let Some(directive) = parse_markdown_html_directive_checked(text).map_err(|message| {
            markdown_error_at(
                source,
                line.text.start() + text.find("fmt:").unwrap_or(0),
                message,
            )
        })? {
            let directive =
                infer_markdown_directive_scope(source, start_line, i, end_line, directive)
                    .map_err(|message| {
                        markdown_error_at(
                            source,
                            line.text.start() + text.find("fmt:").unwrap_or(0),
                            message,
                        )
                    })?;
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Directive),
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
                patch_nested_documents_after_file_scope_delta(
                    source, &mut doc, &delta, options, config,
                )?;
            }
            i += 1;
            continue;
        }

        if let Some(marker_len) = quarto_div_opening(text) {
            let start = i;
            let Some(closing) = find_quarto_div_closing(source, i + 1, end_line, marker_len) else {
                i = end_line;
                let state = engine.state_for_node(&mut doc, true);
                let span = Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                );
                validate_markdown_format_target(source, &doc, state, span, false)?;
                doc.push_node(Node {
                    kind: NodeKind::Markdown(MarkdownNodeKind::QuartoDiv),
                    span,
                    state,
                    emit: EmitPlan::Copy,
                });
                continue;
            };
            let state = engine.state_for_node(&mut doc, true);
            let opening = source.lines[start].full;
            let closing_span = source.lines[closing].full;
            let content = Span::new(opening.end(), closing_span.start());
            let state_value = doc.state(state).clone();
            let nested_options = state_value.markdown_options(options);
            let nested_config = config_for_directive_state(config, &state_value);
            let nested = doc.push_nested(parse_markdown_with_mode(
                source,
                content,
                nested_options,
                &nested_config,
                mode,
            )?);
            let span = Span::new(opening.start(), closing_span.end());
            validate_markdown_format_target(source, &doc, state, span, true)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::QuartoDiv),
                span,
                state,
                emit: EmitPlan::MarkdownDiv {
                    opening: opening.into(),
                    closing: closing_span.into(),
                    nested,
                },
            });
            i = closing + 1;
            continue;
        }

        if pandoc_grid_table_at(source, i, end_line) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            push_markdown_format_node(
                source,
                &mut doc,
                state,
                MarkdownNodeKind::PandocTable,
                Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                MarkdownBlockFormatKind::PandocTable,
                options,
            )?;
            continue;
        }

        if pandoc_multiline_table_at(source, i, end_line) {
            let start = i;
            i = pandoc_multiline_table_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            push_markdown_format_node(
                source,
                &mut doc,
                state,
                MarkdownNodeKind::PandocTable,
                Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                MarkdownBlockFormatKind::PandocTable,
                options,
            )?;
            continue;
        }

        if pandoc_table_at(source, i, end_line) {
            let start = i;
            i = pandoc_simple_table_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            push_markdown_format_node(
                source,
                &mut doc,
                state,
                MarkdownNodeKind::PandocTable,
                Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                MarkdownBlockFormatKind::PandocTable,
                options,
            )?;
            continue;
        }

        if definition_list_at(source, i, end_line) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            let supported = markdown_block_emit_supported(
                source,
                span,
                doc.state(state),
                options,
                MarkdownBlockFormatKind::DefinitionList,
                definition_list_supported(source, start, i),
            );
            validate_markdown_format_target(source, &doc, state, span, supported)?;
            let emit = if supported {
                markdown_format_emit_plan(MarkdownBlockFormatKind::DefinitionList)
            } else {
                EmitPlan::Copy
            };
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::DefinitionList),
                span,
                state,
                emit,
            });
            continue;
        }

        if paired_html_block_start(text).is_some() {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            validate_markdown_format_target(source, &doc, state, span, false)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Raw),
                span,
                state,
                emit: EmitPlan::Copy,
            });
            continue;
        }

        if let Some((content, depth)) = setext_heading_at(source, i, end_line) {
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(source.lines[i].full.start(), source.lines[i + 1].full.end());
            let supported = !contains_markdown_template_span(
                source.slice(span),
                &doc.state(state).template_delimiters,
            );
            validate_markdown_format_target(source, &doc, state, span, supported)?;
            let emit = plan_markdown_setext_heading(source, span, content, depth, doc.state(state));
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::SetextHeading),
                span,
                state,
                emit,
            });
            i += 2;
            continue;
        }

        if let Some((marker, content)) = heading_ranges(source, i) {
            let state = engine.state_for_node(&mut doc, true);
            let supported = !contains_markdown_template_span(
                source.slice(line.full),
                &doc.state(state).template_delimiters,
            );
            validate_markdown_format_target(source, &doc, state, line.full.into(), supported)?;
            let emit =
                plan_markdown_heading(source, line.full.into(), marker, content, doc.state(state));
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Heading),
                span: line.full.into(),
                state,
                emit,
            });
            i += 1;
            continue;
        }

        if thematic_break_at(text) {
            let state = engine.state_for_node(&mut doc, true);
            validate_markdown_format_target(source, &doc, state, line.full.into(), true)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::ThematicBreak),
                span: line.full.into(),
                state,
                emit: plan_markdown_thematic_break(),
            });
            i += 1;
            continue;
        }

        if pipe_table_at(source, i, end_line) {
            let start = i;
            i += 2;
            while i < end_line {
                let candidate = source.line_text(i);
                if !pipe_table_body_row(candidate) {
                    break;
                }
                i += 1;
            }
            let state = engine.state_for_node(&mut doc, true);
            push_markdown_format_node(
                source,
                &mut doc,
                state,
                MarkdownNodeKind::GfmPipeTable,
                Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                MarkdownBlockFormatKind::Table,
                options,
            )?;
            continue;
        }

        if list_item_at(text) {
            let start = i;
            i = list_block_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            let supported = markdown_block_emit_supported(
                source,
                span,
                doc.state(state),
                options,
                MarkdownBlockFormatKind::List,
                list_block_supported(source, start, i),
            );
            validate_markdown_format_target(source, &doc, state, span, supported)?;
            let emit = if supported {
                markdown_format_emit_plan(MarkdownBlockFormatKind::List)
            } else {
                EmitPlan::Copy
            };
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::List),
                span,
                state,
                emit,
            });
            continue;
        }

        if blockquote_at(text) {
            let start = i;
            i += 1;
            while i < end_line && blockquote_at(source.line_text(i)) {
                i += 1;
            }
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            let supported = markdown_block_emit_supported(
                source,
                span,
                doc.state(state),
                options,
                MarkdownBlockFormatKind::Blockquote,
                blockquote_block_supported(source, start, i),
            );
            validate_markdown_format_target(source, &doc, state, span, supported)?;
            let emit = if supported {
                markdown_format_emit_plan(MarkdownBlockFormatKind::Blockquote)
            } else {
                EmitPlan::Copy
            };
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Blockquote),
                span,
                state,
                emit,
            });
            continue;
        }

        if footnote_definition_at(text) {
            let start = i;
            i = footnote_block_end(source, i, end_line);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            let state = engine.state_for_node(&mut doc, true);
            push_markdown_format_node(
                source,
                &mut doc,
                state,
                MarkdownNodeKind::FootnoteDefinition,
                span,
                MarkdownBlockFormatKind::Paragraph,
                options,
            )?;
            continue;
        }

        if html_comment_at(text) {
            let start = i;
            i = html_comment_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, false);
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::HtmlComment),
                span: Span::new(
                    source.lines[start].full.start(),
                    source.lines[i - 1].full.end(),
                ),
                state,
                emit: EmitPlan::Copy,
            });
            continue;
        }

        if shortcode_block_at(text) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            validate_markdown_format_target(source, &doc, state, span, true)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Shortcode),
                span,
                state,
                emit: EmitPlan::MarkdownOpaque,
            });
            continue;
        }

        if display_math_block_at(text) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            validate_markdown_format_target(source, &doc, state, span, true)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::DisplayMath),
                span,
                state,
                emit: EmitPlan::MarkdownOpaque,
            });
            continue;
        }

        if link_definition_start(text.trim_start()) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            validate_markdown_format_target(source, &doc, state, span, true)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::ReferenceDefinition),
                span,
                state,
                emit: EmitPlan::MarkdownOpaque,
            });
            continue;
        }

        if raw_sensitive_at(text) {
            let start = i;
            i = raw_sensitive_end(source, i, end_line);
            let state = engine.state_for_node(&mut doc, true);
            let span = Span::new(
                source.lines[start].full.start(),
                source.lines[i - 1].full.end(),
            );
            validate_markdown_format_target(source, &doc, state, span, false)?;
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Raw),
                span,
                state,
                emit: EmitPlan::Copy,
            });
            continue;
        }

        if standalone_template_line_at(text, &config.markdown_standalone_template_delimiters) {
            let state = engine.state_for_node(&mut doc, true);
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::Raw),
                span: line.full.into(),
                state,
                emit: EmitPlan::Copy,
            });
            i += 1;
            continue;
        }

        if let Some(fence) = code_fence_at(text) {
            let Some(closing) = find_code_fence_closing(source, i + 1, end_line, fence) else {
                let state = engine.state_for_node(&mut doc, true);
                let span = Span::new(line.full.start(), source.lines[end_line - 1].full.end());
                validate_markdown_format_target(source, &doc, state, span, false)?;
                doc.push_node(Node {
                    kind: NodeKind::Markdown(MarkdownNodeKind::Raw),
                    span,
                    state,
                    emit: EmitPlan::Copy,
                });
                break;
            };
            let opening = line.full;
            let closing_span = source.lines[closing].full;
            let content = Span::new(opening.end(), source.lines[closing].full.start());
            let language = code_fence_language(text);
            let raw_format = code_fence_raw_format(text);
            let code_cell_without_language = code_fence_code_cell_without_language(text);
            let local_skip =
                code_fence_local_skip(text) || quarto_fence_skip(source, i + 1, closing);
            let language_has_formatter = language.as_deref().is_some_and(|language| {
                crate::plugins::PluginRegistry::is_known_formatter(config, language)
            });
            let supported_opaque_language = language
                .as_deref()
                .is_some_and(code_fence_language_supported_opaque);
            let no_language = code_fence_info(text).is_some_and(|info| info.is_empty());
            let supported_language_behavior = matches!(
                language.as_deref(),
                Some("yaml" | "yml" | "markdown" | "md")
            ) || no_language
                || language_has_formatter
                || supported_opaque_language
                || raw_format
                || code_cell_without_language;
            let state = engine.state_for_node(&mut doc, true);
            let state_value = doc.state(state).clone();
            let nested_options = state_value.markdown_options(options);
            let normalized_opening = (!local_skip)
                .then(|| {
                    normalized_code_fence_opening(
                        source.line_full(i),
                        supported_language_behavior,
                        nested_options,
                    )
                    .map(String::into_boxed_str)
                })
                .flatten();
            let span = Span::new(opening.start(), closing_span.end());
            validate_markdown_format_target(
                source,
                &doc,
                state,
                span,
                local_skip || supported_language_behavior,
            )?;
            let nested_config = config_for_directive_state(config, &state_value);
            let nested = if local_skip {
                None
            } else {
                match language.as_deref() {
                    Some("yaml" | "yml") => Some(doc.push_nested(parse_nested_yaml(
                        source,
                        content,
                        nested_options,
                        &nested_config,
                        mode,
                    )?)),
                    Some("markdown" | "md") => Some(doc.push_nested(parse_markdown_with_mode(
                        source,
                        content,
                        nested_options,
                        &nested_config,
                        mode,
                    )?)),
                    _ => None,
                }
            };
            let formatter_body = if code_fence_is_code_cell(text) {
                code_cell_formatter_body(source, i + 1, closing).unwrap_or(content)
            } else {
                content
            };
            let emit = if nested.is_none() && !local_skip && language_has_formatter {
                EmitPlan::ExternalPlugin {
                    name: language.unwrap().into_boxed_str(),
                    body: formatter_body,
                    string_indent: None,
                    normalized_opening,
                    fence_safety: Some(fence.safety()),
                }
            } else {
                EmitPlan::MarkdownCodeFence {
                    opening: opening.into(),
                    normalized_opening,
                    closing: closing_span.into(),
                    nested,
                    safety: fence.safety(),
                    supported: !local_skip && supported_language_behavior,
                }
            };
            doc.push_node(Node {
                kind: NodeKind::Markdown(MarkdownNodeKind::CodeFence),
                span,
                state,
                emit,
            });
            i = closing + 1;
            continue;
        }

        let start = i;
        i += 1;
        while i < end_line
            && !paragraph_should_end_before(
                source,
                i,
                &config.markdown_standalone_template_delimiters,
            )
        {
            i += 1;
        }
        let span = Span::new(
            source.lines[start].full.start(),
            source.lines[i - 1].full.end(),
        );
        let state = engine.state_for_node(&mut doc, true);
        push_markdown_format_node(
            source,
            &mut doc,
            state,
            MarkdownNodeKind::Paragraph,
            span,
            MarkdownBlockFormatKind::Paragraph,
            options,
        )?;
    }

    if let Some(message) = engine.pending_target_error() {
        return Err(markdown_error_at(
            source,
            range.end.saturating_sub(1),
            message,
        ));
    }

    Ok(doc)
}

fn parse_nested_yaml<'src>(
    source: &'src SourceBuffer,
    range: Span,
    options: FormatOptions,
    config: &Config,
    mode: MarkdownParseMode,
) -> Result<Document<'src>> {
    match mode {
        MarkdownParseMode::Concrete => {
            crate::core::yaml::parse_yaml(source, range, options, config)
        }
        MarkdownParseMode::SemanticOnly => {
            crate::core::yaml::parse_yaml_for_formatting(source, range, options, config)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkdownBlockFormatKind {
    Paragraph,
    Table,
    PandocTable,
    List,
    DefinitionList,
    Blockquote,
}

pub(crate) fn apply_file_scope_delta_to_markdown_document<'src>(
    source: &'src SourceBuffer,
    doc: &mut Document<'src>,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    let owned_source = doc.source.take();
    if let Some(owned_source) = owned_source {
        let placeholder = Document::new(doc.kind, doc.range);
        let mut owned_doc = std::mem::replace(doc, placeholder).retag_source_lifetime();
        owned_doc.source = None;
        let (owned_doc, result) =
            replan_markdown_document_with_delta(&owned_source, owned_doc, delta, options, config);
        let mut output_doc = owned_doc.retag_source_lifetime();
        output_doc.source = Some(owned_source);
        *doc = output_doc;
        return result;
    }

    let placeholder = Document::new(doc.kind, doc.range);
    let doc_value = std::mem::replace(doc, placeholder);
    let (doc_value, result) =
        replan_markdown_document_with_delta(source, doc_value, delta, options, config);
    *doc = doc_value;
    result
}

fn replan_markdown_document_with_delta<'src>(
    source: &'src SourceBuffer,
    mut doc: Document<'src>,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> (Document<'src>, Result<()>) {
    doc.patch_all_states(delta.clone());
    let result =
        patch_nested_documents_after_file_scope_delta(source, &mut doc, delta, options, config);
    (doc, result)
}

fn patch_nested_documents_after_file_scope_delta<'src>(
    source: &'src SourceBuffer,
    doc: &mut Document<'src>,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    for index in 0..doc.nodes.len() {
        let state = doc.state(doc.nodes[index].state).clone();
        match doc.nodes[index].emit.clone() {
            EmitPlan::MarkdownFrontMatter { nested, .. } => {
                let nested_config = config_for_directive_state(config, &state);
                apply_file_scope_delta_to_nested_document(
                    source,
                    doc,
                    nested,
                    delta,
                    state.markdown_options(options),
                    &nested_config,
                )?;
            }
            EmitPlan::MarkdownCodeFence {
                nested: Some(nested),
                ..
            } => {
                let nested_config = config_for_directive_state(config, &state);
                apply_file_scope_delta_to_nested_document(
                    source,
                    doc,
                    nested,
                    delta,
                    state.markdown_options(options),
                    &nested_config,
                )?;
            }
            EmitPlan::MarkdownDiv { nested, .. } => {
                let nested_config = config_for_directive_state(config, &state);
                apply_file_scope_delta_to_nested_document(
                    source,
                    doc,
                    nested,
                    delta,
                    state.markdown_options(options),
                    &nested_config,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn apply_file_scope_delta_to_nested_document<'src>(
    source: &'src SourceBuffer,
    doc: &mut Document<'src>,
    nested: usize,
    delta: &DirectiveDelta,
    options: FormatOptions,
    config: &Config,
) -> Result<()> {
    match doc.nested[nested].kind {
        DocumentKind::Markdown => apply_file_scope_delta_to_markdown_document(
            source,
            &mut doc.nested[nested],
            delta,
            options,
            config,
        )?,
        DocumentKind::Yaml => crate::core::yaml::apply_file_scope_delta_to_yaml_document(
            source,
            &mut doc.nested[nested],
            delta,
            options,
            config,
        )?,
        DocumentKind::Python | DocumentKind::R => return Ok(()),
    };
    Ok(())
}

fn config_for_directive_state(config: &Config, state: &DirectiveState) -> Config {
    let mut config = config.clone();
    config.template_delimiters = state.template_delimiters.clone();
    config
}

fn plan_markdown_heading<'src>(
    source: &'src SourceBuffer,
    span: Span,
    marker: Span,
    content: Span,
    state: &DirectiveState,
) -> EmitPlan<'src> {
    if contains_markdown_template_span(source.slice(span), &state.template_delimiters) {
        return EmitPlan::Copy;
    }
    EmitPlan::MarkdownHeading { marker, content }
}

fn plan_markdown_setext_heading<'src>(
    source: &'src SourceBuffer,
    span: Span,
    content: Span,
    depth: usize,
    state: &DirectiveState,
) -> EmitPlan<'src> {
    if contains_markdown_template_span(source.slice(span), &state.template_delimiters) {
        return EmitPlan::Copy;
    }
    EmitPlan::MarkdownSetextHeading { content, depth }
}

fn plan_markdown_thematic_break<'src>() -> EmitPlan<'src> {
    EmitPlan::MarkdownThematicBreak
}

fn push_markdown_format_node(
    source: &SourceBuffer,
    doc: &mut Document,
    state: StateId,
    kind: MarkdownNodeKind,
    span: Span,
    format_kind: MarkdownBlockFormatKind,
    options: FormatOptions,
) -> Result<()> {
    let supported =
        markdown_block_emit_supported(source, span, doc.state(state), options, format_kind, true);
    validate_markdown_format_target(source, doc, state, span, supported)?;
    let emit = if supported {
        markdown_format_emit_plan(format_kind)
    } else {
        EmitPlan::Copy
    };
    doc.push_node(Node {
        kind: NodeKind::Markdown(kind),
        span,
        state,
        emit,
    });
    Ok(())
}

fn markdown_block_emit_supported(
    source: &SourceBuffer,
    span: Span,
    state: &DirectiveState,
    options: FormatOptions,
    kind: MarkdownBlockFormatKind,
    structurally_supported: bool,
) -> bool {
    structurally_supported
        && (!state.markdown_target
            || markdown_block_format_supported(source, span, state, options, kind))
}

fn markdown_block_format_supported(
    source: &SourceBuffer,
    span: Span,
    state: &DirectiveState,
    options: FormatOptions,
    kind: MarkdownBlockFormatKind,
) -> bool {
    let input = source.slice(span);
    if contains_markdown_template_span(input, &state.template_delimiters)
        && !matches!(
            kind,
            MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable
        )
    {
        return false;
    }
    match kind {
        MarkdownBlockFormatKind::Paragraph => {
            crate::core::wrap::markdown_paragraph_format_supported(
                input,
                state.markdown_options(options),
            )
        }
        MarkdownBlockFormatKind::Table | MarkdownBlockFormatKind::PandocTable => true,
        MarkdownBlockFormatKind::List => crate::core::wrap::markdown_list_format_supported(
            input,
            state.markdown_options(options),
        ),
        MarkdownBlockFormatKind::DefinitionList => {
            crate::core::wrap::markdown_definition_list_format_supported(
                input,
                state.markdown_options(options),
            )
        }
        MarkdownBlockFormatKind::Blockquote => {
            crate::core::wrap::markdown_blockquote_format_supported(
                input,
                state.markdown_options(options),
            )
        }
    }
}

fn validate_markdown_format_target(
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

fn markdown_format_emit_plan<'src>(kind: MarkdownBlockFormatKind) -> EmitPlan<'src> {
    match kind {
        MarkdownBlockFormatKind::Paragraph => EmitPlan::MarkdownParagraph,
        MarkdownBlockFormatKind::Table => EmitPlan::MarkdownTable,
        MarkdownBlockFormatKind::PandocTable => EmitPlan::MarkdownPandocTable,
        MarkdownBlockFormatKind::List => EmitPlan::MarkdownList,
        MarkdownBlockFormatKind::DefinitionList => EmitPlan::MarkdownDefinitionList,
        MarkdownBlockFormatKind::Blockquote => EmitPlan::MarkdownBlockquote,
    }
}

pub(crate) fn render_markdown_heading(
    source: &SourceBuffer,
    span: Span,
    marker: Span,
    content: Span,
    options: FormatOptions,
) -> String {
    let mut text = source.slice(content).trim().to_owned();
    if options.markdown_canonical {
        text = crate::core::wrap::canonicalize_inline(&text);
    }
    text = crate::core::wrap::normalize_heading_content(&text);

    let mut output = String::new();
    output.push_str(source.slice(Span::new(span.start, marker.start)));
    output.push_str(source.slice(marker));
    if !text.is_empty() {
        output.push(' ');
        output.push_str(&text);
    }
    output.push_str(line_ending_for_span(source, span));
    output
}

pub(crate) fn render_markdown_setext_heading(
    source: &SourceBuffer,
    span: Span,
    content: Span,
    depth: usize,
    options: FormatOptions,
) -> String {
    let mut text = source.slice(content).trim().to_owned();
    if options.markdown_canonical {
        text = crate::core::wrap::canonicalize_inline(&text);
    }
    text = crate::core::wrap::normalize_heading_content(&text);

    let mut output = String::new();
    output.push_str(source.slice(Span::new(span.start, content.start)));
    output.push_str(&"#".repeat(depth));
    if !text.is_empty() {
        output.push(' ');
        output.push_str(&text);
    }
    output.push_str(line_ending_for_span(source, span));
    output
}

pub(crate) fn render_markdown_thematic_break(
    source: &SourceBuffer,
    span: Span,
    options: FormatOptions,
) -> String {
    let mut output = options.markdown_horizontal_rule.to_owned();
    output.push_str(line_ending_for_span(source, span));
    output
}

pub(crate) fn render_markdown_format(
    source: &SourceBuffer,
    span: Span,
    options: FormatOptions,
    kind: MarkdownBlockFormatKind,
) -> String {
    let input = source.slice(span);
    match kind {
        MarkdownBlockFormatKind::Paragraph => {
            crate::core::wrap::format_markdown_paragraph(input, options)
        }
        MarkdownBlockFormatKind::Table => crate::core::wrap::format_markdown_table(input, options),
        MarkdownBlockFormatKind::PandocTable => {
            crate::core::wrap::format_markdown_pandoc_table(input, options)
        }
        MarkdownBlockFormatKind::List => crate::core::wrap::format_markdown_list(input, options),
        MarkdownBlockFormatKind::DefinitionList => {
            crate::core::wrap::format_markdown_definition_list(input, options)
        }
        MarkdownBlockFormatKind::Blockquote => {
            crate::core::wrap::format_markdown_blockquote(input, options)
        }
    }
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

fn markdown_error_at(
    source: &SourceBuffer,
    byte: usize,
    message: impl Into<String>,
) -> crate::diagnostic::YamarkError {
    let (line, column) = source.line_column_at_byte(byte);
    crate::diagnostic::YamarkError::at(message, line, column)
}

fn infer_markdown_directive_scope(
    source: &SourceBuffer,
    start: usize,
    line: usize,
    end: usize,
    directive: Directive,
) -> std::result::Result<Directive, &'static str> {
    let Directive::Template { scope, delimiter } = directive else {
        return Ok(directive);
    };
    if scope != Scope::Next || source.line_text(line).contains("scope=") {
        return Ok(Directive::Template { scope, delimiter });
    }
    if directive_is_isolated(source, start, line, end) {
        Ok(Directive::Template {
            scope: Scope::FromHere,
            delimiter,
        })
    } else if line + 1 < end && markdown_line_starts_target(source, line + 1, end) {
        Ok(Directive::Template { scope, delimiter })
    } else {
        Err("fmt: template.delimiters needs explicit scope")
    }
}

fn directive_is_isolated(source: &SourceBuffer, start: usize, line: usize, end: usize) -> bool {
    let before_blank = line == start || source.line_text(line - 1).trim().is_empty();
    let after_blank = line + 1 >= end || source.line_text(line + 1).trim().is_empty();
    before_blank && after_blank
}

fn markdown_line_starts_target(source: &SourceBuffer, line: usize, end: usize) -> bool {
    let text = source.line_text(line);
    !text.trim().is_empty()
        && parse_markdown_html_directive(text).is_none()
        && paired_html_block_start(text).is_none()
        && (pandoc_table_at(source, line, end)
            || definition_list_at(source, line, end)
            || setext_heading_at(source, line, end).is_some()
            || heading_ranges(source, line).is_some()
            || thematic_break_at(text)
            || quarto_div_opening(text).is_some()
            || pandoc_grid_table_at(source, line, end)
            || pandoc_multiline_table_at(source, line, end)
            || pipe_table_at(source, line, end)
            || list_item_at(text)
            || blockquote_at(text)
            || footnote_definition_at(text)
            || code_fence_at(text).is_some()
            || !raw_sensitive_at(text))
}

fn first_line_index(source: &SourceBuffer, range: Span) -> usize {
    source
        .lines
        .partition_point(|line| line.full.end() <= range.start)
}

fn end_line_index(source: &SourceBuffer, range: Span) -> usize {
    source
        .lines
        .partition_point(|line| line.full.start() < range.end)
}

fn front_matter_opening(text: &str) -> bool {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    front_matter_marker_line(text)
}

fn find_front_matter_closing(source: &SourceBuffer, mut i: usize, end: usize) -> Option<usize> {
    while i < end {
        if front_matter_marker_line(source.line_text(i)) {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn front_matter_marker_line(text: &str) -> bool {
    text.trim_end_matches([' ', '\t']) == "---"
}

fn markdown_on_directive_line(text: &str) -> bool {
    matches!(parse_markdown_html_directive(text), Some(Directive::On))
}

fn markdown_disabled_region_end(source: &SourceBuffer, mut i: usize, end: usize) -> usize {
    while i < end && !markdown_on_directive_line(source.line_text(i)) {
        i += 1;
    }
    i
}

fn paragraph_should_end_before(
    source: &SourceBuffer,
    line: usize,
    standalone_template_delimiters: &[TemplateDelimiter],
) -> bool {
    let text = source.line_text(line);
    if !line_may_start_markdown_block(source, line, text) {
        return false;
    }
    text.trim().is_empty()
        || parse_markdown_html_directive(text).is_some()
        || pandoc_table_at(source, line, source.lines.len())
        || definition_list_at(source, line, source.lines.len())
        || paired_html_block_start(text).is_some()
        || setext_heading_at(source, line, source.lines.len()).is_some()
        || heading_ranges(source, line).is_some()
        || thematic_break_at(text)
        || quarto_div_opening(text).is_some()
        || pandoc_grid_table_at(source, line, source.lines.len())
        || pandoc_multiline_table_at(source, line, source.lines.len())
        || pipe_table_at(source, line, source.lines.len())
        || list_item_at(text)
        || blockquote_at(text)
        || raw_sensitive_at(text)
        || standalone_template_line_at(text, standalone_template_delimiters)
        || code_fence_at(text).is_some()
}

fn standalone_template_line_at(text: &str, delimiters: &[TemplateDelimiter]) -> bool {
    let trimmed = text.trim();
    delimiters
        .iter()
        .any(|delimiter| standalone_template_line_matches(trimmed, delimiter))
}

fn standalone_template_line_matches(text: &str, delimiter: &TemplateDelimiter) -> bool {
    if delimiter.open == "{" && delimiter.close == "}" {
        return python_f_string_replacement_field_line(text);
    }
    text.starts_with(&delimiter.open)
        && text.ends_with(&delimiter.close)
        && text.len() > delimiter.open.len() + delimiter.close.len()
}

fn python_f_string_replacement_field_line(text: &str) -> bool {
    if !text.starts_with('{') || text.starts_with("{{") || !text.ends_with('}') {
        return false;
    }
    balanced_brace_end(text) == Some(text.len())
}

fn balanced_brace_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < text.len() {
        let ch = text[index..].chars().next()?;
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

fn line_may_start_markdown_block(source: &SourceBuffer, line: usize, text: &str) -> bool {
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    let indent = text.len() - text.trim_start().len();
    let Some(first) = trimmed.as_bytes().first().copied() else {
        return true;
    };
    if indent >= 4 || text.trim().is_empty() || list_item_at(text) {
        return true;
    }
    if trimmed.starts_with("Table:") {
        return true;
    }
    if source
        .lines
        .get(line + 1)
        .is_some_and(|_| following_line_can_promote_paragraph(source.line_text(line + 1)))
    {
        return true;
    }
    matches!(
        first,
        b'<' | b'#'
            | b'-'
            | b'='
            | b'*'
            | b'_'
            | b':'
            | b'+'
            | b'|'
            | b'>'
            | b'`'
            | b'~'
            | b'['
            | b'\\'
            | b'$'
            | b'{'
    )
}

fn following_line_can_promote_paragraph(text: &str) -> bool {
    let trimmed = text.trim();
    let marker = text.trim_start();
    pandoc_table_separator(trimmed)
        || (!trimmed.is_empty() && trimmed.chars().all(|ch| ch == '=' || ch == '-'))
        || matches!(marker.as_bytes().first(), Some(b':' | b'~'))
}

fn setext_heading_at(
    source: &SourceBuffer,
    line_index: usize,
    end: usize,
) -> Option<(Span, usize)> {
    if line_index + 1 >= end {
        return None;
    }
    let text = source.line_text(line_index);
    if text.trim().is_empty()
        || parse_markdown_html_directive(text).is_some()
        || heading_ranges(source, line_index).is_some()
        || code_fence_at(text).is_some()
        || list_item_at(text)
    {
        return None;
    }
    let underline = source.line_text(line_index + 1).trim();
    let depth = if !underline.is_empty() && underline.chars().all(|ch| ch == '=') {
        1
    } else if !underline.is_empty() && underline.chars().all(|ch| ch == '-') {
        2
    } else {
        return None;
    };
    let line = source.lines[line_index];
    let body = source.line_text(line_index);
    let bom = if body.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let start = bom
        + body[bom..]
            .bytes()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count();
    let end = body.trim_end().len();
    Some((
        Span::new(line.text.start() + start, line.text.start() + end),
        depth,
    ))
}

fn heading_ranges(source: &SourceBuffer, line_index: usize) -> Option<(Span, Span)> {
    let line = source.lines[line_index];
    let text = source.line_text(line_index);
    let bytes = text.as_bytes();
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let indent = bom
        + bytes[bom..]
            .iter()
            .take_while(|byte| **byte == b' ')
            .count();
    if indent > 3 || indent >= bytes.len() || bytes[indent] != b'#' {
        return None;
    }
    let mut depth = 0usize;
    while indent + depth < bytes.len() && bytes[indent + depth] == b'#' {
        depth += 1;
    }
    if depth == 0 || depth > 6 {
        return None;
    }
    if indent + depth < bytes.len() && !matches!(bytes[indent + depth], b' ' | b'\t') {
        return None;
    }
    let mut content_start = indent + depth;
    while content_start < bytes.len() && matches!(bytes[content_start], b' ' | b'\t') {
        content_start += 1;
    }
    let mut content_end = bytes.len();
    while content_end > content_start && matches!(bytes[content_end - 1], b' ' | b'\t') {
        content_end -= 1;
    }
    let mut hash_start = content_end;
    while hash_start > content_start && bytes[hash_start - 1] == b'#' {
        hash_start -= 1;
    }
    if hash_start < content_end {
        if hash_start == content_start {
            content_end = content_start;
        } else if matches!(bytes[hash_start - 1], b' ' | b'\t') {
            content_end = hash_start - 1;
            while content_end > content_start && matches!(bytes[content_end - 1], b' ' | b'\t') {
                content_end -= 1;
            }
        }
    }
    Some((
        Span::new(
            line.text.start() + indent,
            line.text.start() + indent + depth,
        ),
        Span::new(
            line.text.start() + content_start,
            line.text.start() + content_end,
        ),
    ))
}

#[derive(Debug, Clone, Copy)]
struct Fence {
    marker: u8,
    len: usize,
}

impl Fence {
    fn safety(self) -> CodeFenceSafety {
        CodeFenceSafety {
            marker: self.marker as char,
            min_len: self.len,
        }
    }
}

fn code_fence_at(text: &str) -> Option<Fence> {
    let bytes = text.as_bytes();
    let indent = bytes.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 || indent >= bytes.len() || !matches!(bytes[indent], b'`' | b'~') {
        return None;
    }
    let marker = bytes[indent];
    let mut len = 0usize;
    while indent + len < bytes.len() && bytes[indent + len] == marker {
        len += 1;
    }
    (len >= 3).then_some(Fence { marker, len })
}

fn find_code_fence_closing(
    source: &SourceBuffer,
    mut i: usize,
    end: usize,
    fence: Fence,
) -> Option<usize> {
    while i < end {
        if let Some(candidate) = code_fence_at(source.line_text(i)) {
            let rest = &source.line_text(i).trim_start()[candidate.len..];
            if candidate.marker == fence.marker
                && candidate.len >= fence.len
                && rest.trim().is_empty()
            {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn code_fence_language(text: &str) -> Option<String> {
    code_fence_info(text).and_then(code_fence_language_from_info)
}

fn code_fence_info(text: &str) -> Option<&str> {
    let fence = code_fence_at(text)?;
    Some(text.trim_start()[fence.len..].trim())
}

fn code_fence_language_from_info(info: &str) -> Option<String> {
    let info = info.trim();
    if info.is_empty() || braced_code_fence_raw_format(info) {
        return None;
    }
    if let Some(language) = code_cell_language_from_info(info) {
        return Some(language);
    }
    if let Some(inner) = braced_code_fence_inner(info) {
        return pandoc_attribute_language(inner);
    }
    let first = info.split_whitespace().next()?;
    code_fence_language_token(first)
}

fn code_cell_language_from_info(info: &str) -> Option<String> {
    let rest = info.strip_prefix("{code-cell}")?.trim_start();
    code_fence_language_token(rest.split_whitespace().next()?)
}

fn pandoc_attribute_language(inner: &str) -> Option<String> {
    let mut bare_language = None::<String>;
    for token in inner.split_whitespace() {
        if token.starts_with('#') || token.contains('=') {
            continue;
        }
        if let Some(language) = token.strip_prefix('.').and_then(code_fence_language_token) {
            return Some(language);
        }
        if bare_language.is_none() {
            bare_language = code_fence_language_token(token);
        }
    }
    bare_language
}

fn code_fence_language_token(token: &str) -> Option<String> {
    let token = token
        .trim_start_matches('.')
        .split([',', '}'])
        .next()
        .unwrap_or(token);
    if token.is_empty() || token.starts_with('#') || token.contains('=') {
        return None;
    }
    let language = match token.to_ascii_lowercase().as_str() {
        "ipython3" => "python".to_owned(),
        other => other.to_owned(),
    };
    Some(language)
}

fn code_fence_raw_format(text: &str) -> bool {
    code_fence_info(text).is_some_and(braced_code_fence_raw_format)
}

fn braced_code_fence_raw_format(info: &str) -> bool {
    let Some(inner) = braced_code_fence_inner(info) else {
        return false;
    };
    let Some(format) = inner.trim().strip_prefix('=') else {
        return false;
    };
    !format.is_empty()
        && format
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

fn code_fence_is_code_cell(text: &str) -> bool {
    code_fence_info(text).is_some_and(|info| info.trim().starts_with("{code-cell}"))
}

fn code_fence_code_cell_without_language(text: &str) -> bool {
    code_fence_info(text).is_some_and(|info| {
        let info = info.trim();
        info == "{code-cell}"
            || info
                .strip_prefix("{code-cell}")
                .is_some_and(|rest| rest.trim().is_empty())
    })
}

fn code_fence_language_supported_opaque(language: &str) -> bool {
    matches!(
        language,
        "bash"
            | "sh"
            | "shell"
            | "zsh"
            | "ojs"
            | "text"
            | "console"
            | "rust"
            | "toml"
            | "lua"
            | "mermaid"
            | "ini"
            | "julia"
            | "sql"
            | "java"
            | "c"
            | "tex"
            | "latex"
            | "output"
            | "powershell"
            | "cmd"
    )
}

fn code_cell_formatter_body(
    source: &SourceBuffer,
    mut line: usize,
    closing: usize,
) -> Option<Span> {
    while line < closing && source.line_text(line).trim_start().starts_with(':') {
        line += 1;
    }
    Some(Span::new(
        source.lines.get(line)?.full.start(),
        source.lines[closing].full.start(),
    ))
}

fn normalized_code_fence_opening(
    line: &str,
    normalize_generic_info: bool,
    options: FormatOptions,
) -> Option<String> {
    let (body, newline) = split_line_ending(line);
    let fence = code_fence_at(body)?;
    let indent = body.bytes().take_while(|byte| *byte == b' ').count();
    let marker_end = indent + fence.len;
    let after_marker = &body[marker_end..];
    let info = after_marker.trim();
    if info.is_empty() {
        return None;
    }
    let promote_quarto_options = !matches!(options.markdown_wrap, MarkdownWrap::None)
        && body.chars().count() > options.markdown_wrap_at_column.max(1);
    let normalized_info = normalize_code_fence_info(
        info,
        newline,
        normalize_generic_info,
        promote_quarto_options,
    )
    .or_else(|| {
        (braced_code_fence_info_supported(info) && after_marker.starts_with(char::is_whitespace))
            .then(|| info.to_owned())
    })?;
    let separator = if normalized_info.starts_with('{') {
        ""
    } else if after_marker.starts_with(char::is_whitespace) {
        " "
    } else {
        ""
    };
    let normalized = format!(
        "{}{}{}{}{}",
        &body[..indent],
        &body[indent..marker_end],
        separator,
        normalized_info,
        newline
    );
    (normalized != line).then_some(normalized)
}

fn braced_code_fence_info_starts_with_language(info: &str) -> bool {
    let Some(inner) = braced_code_fence_inner(info) else {
        return false;
    };
    let Some(first) = inner.split_whitespace().next() else {
        return false;
    };
    let first = first.trim_start_matches('.');
    !first.is_empty() && !first.starts_with('#') && !first.contains('=')
}

fn braced_code_fence_info_supported(info: &str) -> bool {
    braced_code_fence_raw_format(info)
        || braced_code_fence_info_starts_with_language(info)
        || code_fence_language_from_info(info).is_some()
}

fn braced_code_fence_inner(info: &str) -> Option<&str> {
    info.strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
}

fn normalize_code_fence_info(
    info: &str,
    newline: &str,
    normalize_generic_info: bool,
    promote_quarto_options: bool,
) -> Option<String> {
    let Some(inner) = braced_code_fence_inner(info) else {
        if let Some(normalized) = normalize_bare_language_attribute_info(info) {
            return Some(normalized);
        }
        if !normalize_generic_info {
            return None;
        }
        let normalized = info.split_whitespace().collect::<Vec<_>>().join(" ");
        return (normalized != info).then_some(normalized);
    };
    if promote_quarto_options && let Some(promoted) = promote_quarto_chunk_header(inner, newline) {
        return Some(promoted);
    }
    if inner.contains('"') || inner.contains('\'') {
        return None;
    }
    let mut seen_classes = Vec::<String>::new();
    let mut tokens = Vec::<&str>::new();
    for token in inner.split_whitespace() {
        if let Some(class) = token.strip_prefix('.') {
            let class_key = class.to_ascii_lowercase();
            if seen_classes.iter().any(|seen| seen == &class_key) {
                continue;
            }
            seen_classes.push(class_key);
        } else if tokens.is_empty() && !token.starts_with('#') && !token.contains('=') {
            seen_classes.push(token.trim_start_matches('.').to_ascii_lowercase());
        } else if !token.starts_with('#')
            && !token.contains('=')
            && seen_classes
                .iter()
                .any(|seen| seen == &token.trim_start_matches('.').to_ascii_lowercase())
        {
            continue;
        }
        tokens.push(token);
    }
    if tokens.is_empty() {
        return None;
    }
    let normalized = format!("{{{}}}", tokens.join(" "));
    (normalized != info).then_some(normalized)
}

fn normalize_bare_language_attribute_info(info: &str) -> Option<String> {
    let (language, rest) = info.split_once(char::is_whitespace)?;
    if language.is_empty()
        || language.starts_with('{')
        || language.starts_with('#')
        || language.contains('=')
    {
        return None;
    }
    let rest = rest.trim();
    let inner = rest
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))?;
    if inner.contains('"') || inner.contains('\'') {
        return None;
    }

    let language_key = language.trim_start_matches('.').to_ascii_lowercase();
    let mut changed = false;
    let mut tokens = Vec::new();
    for token in inner.split_whitespace() {
        if let Some(class) = token.strip_prefix('.')
            && class.eq_ignore_ascii_case(&language_key)
        {
            changed = true;
            continue;
        }
        tokens.push(token);
    }
    if !changed {
        return None;
    }
    if tokens.is_empty() {
        Some(language.to_owned())
    } else {
        Some(format!("{} {{{}}}", language, tokens.join(" ")))
    }
}

fn promote_quarto_chunk_header(inner: &str, newline: &str) -> Option<String> {
    if newline.is_empty() {
        return None;
    }
    if !inner.contains(',') || inner.contains(['"', '\'']) {
        return None;
    }
    let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
    let (language, options) = parts.split_first()?;
    let language = language.trim_start_matches('.');
    if language.is_empty()
        || language.contains(char::is_whitespace)
        || language.starts_with('#')
        || language.contains('=')
        || options.is_empty()
    {
        return None;
    }

    let mut lines = Vec::with_capacity(options.len());
    for option in options {
        let (key, value) = option.split_once('=')?;
        let key = key.trim();
        let value = normalize_quarto_option_value(value.trim())?;
        if key.is_empty()
            || value.is_empty()
            || !key
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        {
            return None;
        }
        lines.push(format!("#| {key}: {value}"));
    }
    Some(format!("{{{language}}}{newline}{}", lines.join(newline)))
}

fn normalize_quarto_option_value(value: &str) -> Option<String> {
    if value.contains(char::is_whitespace) || value.contains(['{', '}', '[', ']']) {
        return None;
    }
    Some(match value {
        "TRUE" | "True" => "true".to_owned(),
        "FALSE" | "False" => "false".to_owned(),
        _ => value.to_owned(),
    })
}

fn split_line_ending(text: &str) -> (&str, &str) {
    if let Some(body) = text.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = text.strip_suffix('\n') {
        (body, "\n")
    } else if let Some(body) = text.strip_suffix('\r') {
        (body, "\r")
    } else {
        (text, "")
    }
}

fn thematic_break_at(text: &str) -> bool {
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    let mut marker = None;
    let mut count = 0usize;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            continue;
        }
        if !matches!(ch, '-' | '*' | '_') {
            return false;
        }
        if let Some(marker) = marker {
            if marker != ch {
                return false;
            }
        } else {
            marker = Some(ch);
        }
        count += 1;
    }
    count >= 3
}

fn quarto_div_opening(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    if indent > 3 || !trimmed.starts_with(":::") {
        return None;
    }
    let marker_len = trimmed.bytes().take_while(|byte| *byte == b':').count();
    let rest = trimmed[marker_len..].trim();
    if rest.chars().all(|ch| ch == ':') {
        return None;
    }
    Some(marker_len)
}

fn find_quarto_div_closing(
    source: &SourceBuffer,
    mut line: usize,
    end: usize,
    marker_len: usize,
) -> Option<usize> {
    let mut depth = 1usize;
    while line < end {
        let trimmed = source.line_text(line).trim();
        if let Some(opening_len) = quarto_div_opening(source.line_text(line))
            && opening_len >= marker_len
        {
            depth += 1;
            line += 1;
            continue;
        }
        if trimmed.len() >= marker_len && trimmed.chars().all(|ch| ch == ':') {
            depth -= 1;
            if depth == 0 {
                return Some(line);
            }
        }
        line += 1;
    }
    None
}

fn pipe_table_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    line + 1 < end
        && source.line_text(line).trim_start().starts_with('|')
        && pipe_table_delimiter(source.line_text(line + 1))
}

fn pandoc_grid_table_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    let Some(columns) = grid_table_border(source.line_text(line).trim()) else {
        return false;
    };
    pandoc_grid_table_next_border(source, line + 1, end, columns).is_some()
}

fn pandoc_grid_table_next_border(
    source: &SourceBuffer,
    mut line: usize,
    end: usize,
    columns: usize,
) -> Option<usize> {
    let mut saw_row = false;
    while line < end {
        let trimmed = source.line_text(line).trim();
        if grid_table_border(trimmed).is_some_and(|found| found == columns) {
            return saw_row.then_some(line);
        }
        if !source.line_text(line).trim_start().starts_with('|') {
            return None;
        }
        saw_row = true;
        line += 1;
    }
    None
}

fn pandoc_multiline_table_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    if line + 3 >= end {
        return false;
    }
    let first = source.line_text(line).trim();
    if pandoc_multiline_outer_separator(first) && !pandoc_table_separator(first) {
        return pandoc_continuous_bound_multiline_table_at(source, line, end);
    }
    if !pandoc_table_separator(first) {
        return false;
    }
    let mut separator_count = 1usize;
    let mut i = line + 1;
    while i < end {
        let trimmed = source.line_text(i).trim();
        if pandoc_table_separator(trimmed) {
            separator_count += 1;
            if separator_count == 3 {
                return true;
            }
        }
        if trimmed.is_empty() && separator_count < 2 {
            return false;
        }
        i += 1;
    }
    false
}

fn pandoc_continuous_bound_multiline_table_at(
    source: &SourceBuffer,
    line: usize,
    end: usize,
) -> bool {
    let mut saw_header = false;
    let mut saw_inner_separator = false;
    let mut saw_body = false;
    let mut i = line + 1;
    while i < end {
        let trimmed = source.line_text(i).trim();
        if !saw_inner_separator {
            if trimmed.is_empty() {
                return false;
            }
            if pandoc_table_separator(trimmed) {
                if !saw_header {
                    return false;
                }
                saw_inner_separator = true;
            } else if pandoc_multiline_outer_separator(trimmed) {
                return false;
            } else {
                saw_header = true;
            }
        } else if pandoc_multiline_outer_separator(trimmed) {
            return saw_body;
        } else if !trimmed.is_empty() {
            saw_body = true;
        }
        i += 1;
    }
    false
}

fn pandoc_multiline_table_end(source: &SourceBuffer, line: usize, end: usize) -> usize {
    if pandoc_multiline_outer_separator(source.line_text(line).trim())
        && !pandoc_table_separator(source.line_text(line).trim())
    {
        let mut saw_inner_separator = false;
        let mut i = line + 1;
        while i < end {
            let trimmed = source.line_text(i).trim();
            if saw_inner_separator && pandoc_multiline_outer_separator(trimmed) {
                return i + 1;
            }
            if pandoc_table_separator(trimmed) {
                saw_inner_separator = true;
            }
            i += 1;
        }
        return end;
    }
    let mut separator_count = 0usize;
    let mut i = line;
    while i < end {
        if pandoc_table_separator(source.line_text(i).trim()) {
            separator_count += 1;
            if separator_count == 3 {
                return i + 1;
            }
        }
        i += 1;
    }
    end
}

fn pandoc_multiline_outer_separator(text: &str) -> bool {
    text.len() >= 3 && text.chars().all(|ch| matches!(ch, '-' | '='))
}

fn grid_table_border(text: &str) -> Option<usize> {
    let trimmed = text.trim();
    if !trimmed.starts_with('+') || !trimmed.ends_with('+') {
        return None;
    }
    let mut columns = 0usize;
    for part in trimmed.split('+').skip(1) {
        if part.is_empty() {
            continue;
        }
        if !part.chars().all(|ch| matches!(ch, '-' | '=')) {
            return None;
        }
        columns += 1;
    }
    (columns > 0).then_some(columns)
}

fn pipe_table_delimiter(text: &str) -> bool {
    let trimmed = trim_optional_pipe_edges(text.trim());
    if trimmed.is_empty() {
        return false;
    }
    let mut saw_dash = false;
    for ch in trimmed.chars() {
        match ch {
            '-' => saw_dash = true,
            '|' | ':' | ' ' | '\t' => {}
            _ => return false,
        }
    }
    saw_dash
}

fn pipe_table_body_row(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty() && trimmed.contains('|')
}

fn trim_optional_pipe_edges(mut text: &str) -> &str {
    if let Some(rest) = text.strip_prefix('|') {
        text = rest;
    }
    if let Some(rest) = text.strip_suffix('|') {
        text = rest;
    }
    text.trim()
}

fn list_item_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    if indent > 3 {
        return false;
    }
    markdown_list_marker_len(trimmed).is_some()
}

fn list_block_end(source: &SourceBuffer, start: usize, end: usize) -> usize {
    let first = source.line_text(start);
    let base_indent = first.len() - first.trim_start().len();
    let mut item_content_indent = list_item_content_indent(first);
    let mut line = start + 1;
    while line < end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            let Some(next) = next_nonblank_line(source, line + 1, end) else {
                return end;
            };
            if list_item_at(source.line_text(next)) {
                break;
            }
            if blank_line_continues_list_item(source.line_text(next), item_content_indent) {
                line += 1;
                continue;
            }
            break;
        }
        if list_item_at(text) {
            item_content_indent = list_item_content_indent(text);
            line += 1;
            continue;
        }
        let indent = text.len() - text.trim_start().len();
        if indent > base_indent {
            line += 1;
            continue;
        }
        break;
    }
    line
}

fn next_nonblank_line(source: &SourceBuffer, mut line: usize, end: usize) -> Option<usize> {
    while line < end {
        if !source.line_text(line).trim().is_empty() {
            return Some(line);
        }
        line += 1;
    }
    None
}

fn blank_line_continues_list_item(text: &str, item_content_indent: Option<usize>) -> bool {
    let Some(item_content_indent) = item_content_indent else {
        return false;
    };
    let indent = text.len() - text.trim_start().len();
    indent >= item_content_indent
}

fn list_block_supported(source: &SourceBuffer, start: usize, end: usize) -> bool {
    let first = source.line_text(start);
    let base_indent = first.len() - first.trim_start().len();
    let mut item_content_indent = list_item_content_indent(first);
    let mut task_continuation = task_list_continuation_range(first);
    for line in start + 1..end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            continue;
        }
        if list_item_at(text) {
            item_content_indent = list_item_content_indent(text);
            task_continuation = task_list_continuation_range(text);
            continue;
        }
        let indent = text.len() - text.trim_start().len();
        if blockquote_at(text) {
            let Some(body) = blockquote_plain_body(text) else {
                return false;
            };
            if list_item_at(body) || raw_sensitive_at(body) || code_fence_at(body).is_some() {
                return false;
            }
            continue;
        }
        if list_continuation_matches_item_content(text, indent, item_content_indent) {
            continue;
        }
        if let Some((min, max)) = task_continuation
            && indent >= min
            && indent < max
        {
            continue;
        }
        if indent >= base_indent + 4 || raw_sensitive_at(text) || code_fence_at(text).is_some() {
            if list_rich_child_line(text, item_content_indent) {
                continue;
            }
            return false;
        }
    }
    true
}

fn list_rich_child_line(text: &str, item_content_indent: Option<usize>) -> bool {
    let Some(item_content_indent) = item_content_indent else {
        return false;
    };
    let indent = text.len() - text.trim_start().len();
    if indent < item_content_indent {
        return false;
    }
    let content = &text[indent..];
    code_fence_at(content).is_some() || raw_sensitive_at(content) || rich_child_block_start(content)
}

fn rich_child_block_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('|')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || trimmed.starts_with("$$")
        || trimmed.starts_with("\\[")
        || trimmed.starts_with("\\begin{")
        || trimmed.starts_with(":::")
        || trimmed.starts_with("{{<")
        || trimmed.starts_with("{{%")
        || blockquote_at(trimmed)
        || list_item_at(trimmed)
}

fn list_item_content_indent(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    let marker_len = markdown_list_marker_len(trimmed)?;
    let after_marker = &trimmed[marker_len..];
    let marker_gap = after_marker.len() - after_marker.trim_start().len();
    let mut content_indent = indent + marker_len + marker_gap;
    let rest = &after_marker[marker_gap..];
    if let Some(checkbox) = task_list_checkbox(rest) {
        let after_checkbox = &rest[checkbox.len()..];
        let checkbox_gap = after_checkbox.len() - after_checkbox.trim_start().len();
        content_indent += checkbox.len() + checkbox_gap;
    }
    Some(content_indent)
}

fn list_continuation_matches_item_content(
    text: &str,
    indent: usize,
    item_content_indent: Option<usize>,
) -> bool {
    if item_content_indent != Some(indent) {
        return false;
    }
    let content = &text[indent..];
    !list_item_at(content) && !raw_sensitive_at(content) && code_fence_at(content).is_none()
}

fn task_list_continuation_range(text: &str) -> Option<(usize, usize)> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    let marker_len = markdown_list_marker_len(trimmed)?;
    let rest = trimmed[marker_len..].trim_start();
    let checkbox = task_list_checkbox(rest)?;
    let continuation = indent + marker_len + 1 + checkbox.len() + 1;
    Some((continuation, continuation + 4))
}

fn task_list_checkbox(body: &str) -> Option<&str> {
    let bytes = body.as_bytes();
    if bytes.len() < 3
        || bytes[0] != b'['
        || !matches!(bytes[1], b' ' | b'x' | b'X')
        || bytes[2] != b']'
    {
        return None;
    }
    if bytes.get(3).is_some_and(|byte| !byte.is_ascii_whitespace()) {
        return None;
    }
    Some(&body[..3])
}

fn blockquote_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    text.len() - trimmed.len() <= 3 && trimmed.starts_with('>')
}

fn blockquote_block_supported(source: &SourceBuffer, start: usize, end: usize) -> bool {
    for line in start..end {
        if blockquote_plain_body(source.line_text(line)).is_none() {
            return false;
        };
    }
    true
}

fn blockquote_plain_body(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    if text.len() - trimmed.len() > 3 {
        return None;
    }
    let mut body = trimmed.strip_prefix('>')?.trim_start();
    while let Some(rest) = body.strip_prefix('>') {
        body = rest.trim_start();
    }
    Some(body)
}

fn footnote_definition_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    text.len() - trimmed.len() <= 3 && trimmed.starts_with("[^") && trimmed.contains("]:")
}

fn footnote_block_end(source: &SourceBuffer, start: usize, end: usize) -> usize {
    let mut line = start + 1;
    while line < end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            let Some(next) = next_nonblank_line(source, line + 1, end) else {
                break;
            };
            let indent = source.line_text(next).len() - source.line_text(next).trim_start().len();
            if indent >= 4 {
                line = next + 1;
                continue;
            }
            break;
        }
        let indent = text.len() - text.trim_start().len();
        if indent >= 2 {
            line += 1;
            continue;
        }
        break;
    }
    line
}

fn html_comment_at(text: &str) -> bool {
    text.trim_start().starts_with("<!--")
}

fn html_comment_end(source: &SourceBuffer, line: usize, end: usize) -> usize {
    if source.line_text(line).contains("-->") {
        line + 1
    } else {
        find_until_contains(source, line + 1, end, "-->")
    }
}

fn raw_sensitive_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    indent >= 4
        || trimmed.starts_with('\t')
        || display_math_delimiter(trimmed)
        || trimmed.starts_with("\\[")
        || trimmed.starts_with("\\begin{")
        || trimmed.starts_with("{{<")
        || trimmed.starts_with("{{%")
        || trimmed.starts_with('|')
        || pandoc_block_attribute_line(trimmed)
        || trimmed.starts_with("Table:")
        || trimmed.starts_with(':')
        || trimmed.starts_with("<!--")
        || trimmed.starts_with("<div")
        || trimmed.starts_with("</div")
        || paired_html_block_start(text).is_some()
        || link_definition_start(trimmed)
        || trimmed.starts_with("#.")
        || trimmed.starts_with('+')
        || grid_table_separator(trimmed)
}

fn shortcode_block_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    indent <= 3 && (trimmed.starts_with("{{<") || trimmed.starts_with("{{%"))
}

fn display_math_block_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    indent <= 3
        && (display_math_delimiter(trimmed)
            || trimmed.starts_with("\\[")
            || trimmed.starts_with("\\begin{"))
}

fn pandoc_block_attribute_line(trimmed: &str) -> bool {
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    let inner = trimmed[1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return false;
    }
    let Some(tokens) = pandoc_attribute_tokens(inner) else {
        return false;
    };
    !tokens.is_empty() && tokens.iter().all(|token| pandoc_attribute_token(token))
}

fn pandoc_attribute_tokens(inner: &str) -> Option<Vec<&str>> {
    let mut tokens = Vec::new();
    let mut start = None::<usize>;
    let mut in_double_quote = false;
    for (index, ch) in inner.char_indices() {
        match ch {
            '"' => {
                if start.is_none() {
                    start = Some(index);
                }
                in_double_quote = !in_double_quote;
            }
            ch if ch.is_whitespace() && !in_double_quote => {
                if let Some(token_start) = start.take() {
                    tokens.push(&inner[token_start..index]);
                }
            }
            _ if start.is_none() => start = Some(index),
            _ => {}
        }
    }
    if in_double_quote {
        return None;
    }
    if let Some(token_start) = start {
        tokens.push(&inner[token_start..]);
    }
    Some(tokens)
}

fn pandoc_attribute_token(token: &str) -> bool {
    if token == "-" {
        return true;
    }
    if let Some(value) = token.strip_prefix('#').or_else(|| token.strip_prefix('.')) {
        return !value.is_empty() && !value.contains(['{', '}', '"', '\'']);
    }
    let Some((key, value)) = token.split_once('=') else {
        return false;
    };
    !key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
        && !value.is_empty()
        && !value.contains(['{', '}', '\''])
}

fn raw_sensitive_end(source: &SourceBuffer, line: usize, end: usize) -> usize {
    let trimmed = source.line_text(line).trim_start();
    if pandoc_table_at(source, line, end) || definition_list_at(source, line, end) {
        let mut i = line + 1;
        while i < end && !source.line_text(i).trim().is_empty() {
            i += 1;
        }
        return i;
    }
    if let Some(tag) = paired_html_block_start(source.line_text(line)) {
        return find_until_html_closing_tag(source, line + 1, end, &tag);
    }
    if let Some(shortcode) = hugo_shortcode_opening(trimmed)
        && let Some(close) = find_hugo_shortcode_close(source, line + 1, end, shortcode)
    {
        return close;
    }
    if display_math_delimiter(trimmed) {
        if trimmed[2..].contains("$$") {
            return line + 1;
        }
        return find_until_display_math_close(source, line + 1, end);
    }
    if trimmed.starts_with("\\[") {
        return find_until_contains(source, line, end, "\\]");
    }
    if let Some(name) = trimmed
        .strip_prefix("\\begin{")
        .and_then(|rest| rest.split_once('}').map(|(name, _)| name.to_owned()))
    {
        let closing = format!("\\end{{{name}}}");
        if trimmed.contains(&closing) {
            return line + 1;
        }
        return find_until_contains(source, line + 1, end, &closing);
    }
    if trimmed.starts_with("<!--") && !trimmed.contains("-->") {
        return find_until_contains(source, line + 1, end, "-->");
    }
    let mut i = line + 1;
    while i < end {
        let text = source.line_text(i);
        if text.trim().is_empty() {
            break;
        }
        if raw_continuation(source.line_text(line), text) {
            i += 1;
        } else {
            break;
        }
    }
    i.max(line + 1)
}

fn display_math_delimiter(trimmed: &str) -> bool {
    trimmed.starts_with("$$")
}

fn find_until_display_math_close(source: &SourceBuffer, mut line: usize, end: usize) -> usize {
    while line < end {
        if display_math_delimiter(source.line_text(line).trim_start()) {
            return line + 1;
        }
        line += 1;
    }
    end
}

fn raw_continuation(first: &str, candidate: &str) -> bool {
    let first_indent = first.len() - first.trim_start().len();
    let candidate_indent = candidate.len() - candidate.trim_start().len();
    let first_trimmed = first.trim_start();
    let candidate_trimmed = candidate.trim_start();
    if first_trimmed.starts_with('|') {
        return candidate_trimmed.starts_with('|');
    }
    if first_trimmed.starts_with('+') || grid_table_separator(first_trimmed) {
        return candidate_trimmed.starts_with('+')
            || candidate_trimmed.starts_with('|')
            || grid_table_separator(candidate_trimmed);
    }
    if first_trimmed.starts_with("Table:") || first_trimmed.starts_with(':') {
        return candidate.starts_with(' ') || candidate.starts_with('\t');
    }
    if link_definition_start(first_trimmed) {
        return candidate.starts_with(' ') || candidate.starts_with('\t');
    }
    if pandoc_table_separator(first_trimmed) {
        return !candidate_trimmed.is_empty();
    }
    first_indent >= 4 && candidate_indent >= 4
}

#[derive(Debug, Clone, Copy)]
struct HugoShortcodeOpening<'a> {
    delimiter: char,
    name: &'a str,
}

fn hugo_shortcode_opening(trimmed: &str) -> Option<HugoShortcodeOpening<'_>> {
    let (delimiter, rest) = if let Some(rest) = trimmed.strip_prefix("{{<") {
        ('>', rest)
    } else {
        ('%', trimmed.strip_prefix("{{%")?)
    };
    let rest = rest.trim_start();
    if rest.starts_with('/') {
        return None;
    }
    let name_end = rest
        .char_indices()
        .find_map(|(index, ch)| (ch.is_whitespace() || ch == delimiter).then_some(index))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    (!name.is_empty()).then_some(HugoShortcodeOpening { delimiter, name })
}

fn find_hugo_shortcode_close(
    source: &SourceBuffer,
    mut line: usize,
    end: usize,
    opening: HugoShortcodeOpening<'_>,
) -> Option<usize> {
    while line < end {
        if hugo_shortcode_closes(source.line_text(line).trim_start(), opening) {
            return Some(line + 1);
        }
        line += 1;
    }
    None
}

fn hugo_shortcode_closes(trimmed: &str, opening: HugoShortcodeOpening<'_>) -> bool {
    let rest = match opening.delimiter {
        '>' => trimmed.strip_prefix("{{<"),
        '%' => trimmed.strip_prefix("{{%"),
        _ => None,
    };
    let Some(rest) = rest else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix('/') else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(after_name) = rest.strip_prefix(opening.name) else {
        return false;
    };
    let after_name = after_name.trim_start();
    match opening.delimiter {
        '>' => after_name.starts_with(">}}"),
        '%' => after_name.starts_with("%}}"),
        _ => false,
    }
}

fn link_definition_start(trimmed: &str) -> bool {
    trimmed.starts_with('[') && trimmed.contains("]:")
}

fn pandoc_table_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    if line + 1 >= end {
        return false;
    }
    let current = source.line_text(line);
    let next = source.line_text(line + 1);
    if current.trim().is_empty() || table_caption_at(current) {
        return false;
    }
    pandoc_table_separator(next.trim())
}

fn pandoc_simple_table_end(source: &SourceBuffer, line: usize, end: usize) -> usize {
    let mut i = line + 2;
    while i < end {
        let text = source.line_text(i);
        if text.trim().is_empty() || table_caption_at(text) {
            break;
        }
        i += 1;
    }
    i
}

fn table_caption_at(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with("Table:") || trimmed.starts_with(':')
}

fn definition_list_at(source: &SourceBuffer, line: usize, end: usize) -> bool {
    if line + 1 >= end || source.line_text(line).trim().is_empty() {
        return false;
    }
    let marker = source.line_text(line + 1).trim_start();
    let indent = source.line_text(line + 1).len() - marker.len();
    indent <= 3
        && matches!(marker.as_bytes().first(), Some(b':' | b'~'))
        && marker
            .as_bytes()
            .get(1)
            .is_some_and(u8::is_ascii_whitespace)
}

fn definition_list_supported(source: &SourceBuffer, start: usize, end: usize) -> bool {
    for line in start..end {
        let text = source.line_text(line);
        if text.trim().is_empty() {
            continue;
        }
        if let Some(content) = definition_marker_content(text) {
            if raw_sensitive_at(content) || code_fence_at(content).is_some() {
                return false;
            }
            continue;
        }
        let trimmed = text.trim_start();
        let indent = text.len() - trimmed.len();
        if indent >= 4 {
            if raw_sensitive_at(trimmed) || code_fence_at(trimmed).is_some() {
                return false;
            }
            continue;
        }
        if raw_sensitive_at(text) || code_fence_at(text).is_some() {
            return false;
        }
    }
    true
}

fn definition_marker_content(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    let indent = text.len() - trimmed.len();
    if indent > 3 || !matches!(trimmed.as_bytes().first(), Some(b':' | b'~')) {
        return None;
    }
    if !trimmed
        .as_bytes()
        .get(1)
        .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    Some(trimmed[1..].trim_start())
}

fn pandoc_table_separator(text: &str) -> bool {
    let columns = text.split_whitespace().collect::<Vec<_>>();
    columns.len() >= 2
        && columns.iter().all(|column| {
            column.len() >= 3
                && column.chars().all(|ch| matches!(ch, '-' | '=' | ':'))
                && column.chars().any(|ch| matches!(ch, '-' | '='))
        })
}

fn paired_html_block_start(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('<')
        || trimmed.starts_with("</")
        || trimmed.starts_with("<!--")
        || trimmed.starts_with("<!")
        || trimmed.starts_with("<?")
        || crate::core::wrap::commonmark_autolink_span_end(trimmed, 0).is_some()
    {
        return None;
    }
    let mut chars = trimmed[1..].char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    let mut tag_end = 1 + first.len_utf8();
    for (offset, ch) in chars {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            tag_end = 1 + offset + ch.len_utf8();
            continue;
        }
        break;
    }
    let tag = &trimmed[1..tag_end];
    let rest = &trimmed[tag_end..];
    if !rest.contains('>')
        || rest.trim_end().ends_with("/>")
        || contains_html_closing_tag(trimmed, tag)
    {
        return None;
    }
    Some(tag.to_owned())
}

fn find_until_html_closing_tag(
    source: &SourceBuffer,
    mut line: usize,
    end: usize,
    tag: &str,
) -> usize {
    while line < end {
        if contains_html_closing_tag(source.line_text(line), tag) {
            return line + 1;
        }
        line += 1;
    }
    end
}

fn contains_html_closing_tag(text: &str, tag: &str) -> bool {
    let mut search = text;
    while let Some(relative) = search.find("</") {
        let candidate = &search[relative + 2..];
        let name_len = candidate
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
            .count();
        let name = &candidate[..name_len];
        let rest = candidate[name_len..].trim_start();
        if !name.is_empty() && name.eq_ignore_ascii_case(tag) && rest.starts_with('>') {
            return true;
        }
        search = &candidate[name_len..];
    }
    false
}

fn grid_table_separator(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() >= 3
        && trimmed
            .chars()
            .all(|ch| matches!(ch, '-' | '=' | ':' | ' ' | '\t'))
}

fn find_until_contains(source: &SourceBuffer, mut line: usize, end: usize, target: &str) -> usize {
    while line < end {
        if source.line_text(line).contains(target) {
            return line + 1;
        }
        line += 1;
    }
    end
}

fn code_fence_local_skip(text: &str) -> bool {
    let Some(fence) = code_fence_at(text) else {
        return false;
    };
    let info = text.trim_start()[fence.len..].trim();
    code_fence_info_has_exact_local_skip(info)
}

fn code_fence_info_has_exact_local_skip(info: &str) -> bool {
    let tokens = info
        .split_whitespace()
        .map(code_fence_info_token)
        .collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if *token == "fmt:skip" && local_skip_directive_ends(&tokens, index + 1) {
            return true;
        }
        if *token == "fmt:"
            && tokens.get(index + 1).copied() == Some("skip")
            && local_skip_directive_ends(&tokens, index + 2)
        {
            return true;
        }
    }
    false
}

fn code_fence_info_token(token: &str) -> &str {
    token
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim_end_matches(',')
}

fn local_skip_directive_ends(tokens: &[&str], start: usize) -> bool {
    start >= tokens.len()
        || (tokens.get(start).copied() == Some("scope=next") && start + 1 == tokens.len())
}

fn quarto_fence_skip(source: &SourceBuffer, mut line: usize, closing: usize) -> bool {
    while line < closing {
        let trimmed = source.line_text(line).trim_start();
        if trimmed.starts_with("#|") {
            let rest = trimmed.trim_start_matches("#|").trim();
            if rest == "fmt: skip" || rest == "fmt:skip" || rest == "fmt: skip scope=next" {
                return true;
            }
            line += 1;
            continue;
        }
        break;
    }
    false
}

fn front_matter_markdown_delta(
    source: &SourceBuffer,
    document: &Document,
) -> Option<DirectiveDelta> {
    let ast = document.yaml.as_ref()?;
    if let Some(editor_options) =
        front_matter_markdown_delta_at_path(source, ast, &["editor_options", "markdown"])
    {
        return directive_delta_has_markdown_options(&editor_options).then_some(editor_options);
    }
    let delta = front_matter_markdown_delta_at_path(source, ast, &["editor", "markdown"])?;
    directive_delta_has_markdown_options(&delta).then_some(delta)
}

fn front_matter_markdown_delta_at_path(
    source: &SourceBuffer,
    ast: &YamlDocumentAst<'_>,
    path: &[&str],
) -> Option<DirectiveDelta> {
    let root = ast.roots.iter().find_map(|root| root.node)?;
    let node = yaml_mapping_value_at_path(source, ast, root, path)?;
    let mut delta = DirectiveDelta::default();
    for (key, value) in yaml_mapping_scalar_pairs(source, ast, node) {
        match key.as_str() {
            "wrap" => apply_front_matter_wrap(&mut delta, &value),
            "canonical" => delta.markdown_canonical = parse_front_matter_bool(&value),
            "footnotes" => {
                delta.markdown_format_footnotes = match value.as_str() {
                    "wrap" | "format" | "true" | "yes" | "1" => Some(true),
                    "preserve" | "none" | "false" | "no" | "0" => Some(false),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    Some(delta)
}

fn yaml_mapping_value_at_path(
    source: &SourceBuffer,
    ast: &YamlDocumentAst<'_>,
    mut node: YamlNodeId,
    path: &[&str],
) -> Option<YamlNodeId> {
    for part in path {
        node = yaml_mapping_value_for_key(source, ast, node, part)?;
    }
    Some(node)
}

fn yaml_mapping_value_for_key(
    source: &SourceBuffer,
    ast: &YamlDocumentAst<'_>,
    node: YamlNodeId,
    key: &str,
) -> Option<YamlNodeId> {
    match &ast.node(node).kind {
        YamlAstKind::Mapping(mapping) => {
            mapping
                .pairs
                .iter()
                .rev()
                .find(|pair| yaml_block_mapping_key(source, ast, pair).as_deref() == Some(key))?
                .value
        }
        YamlAstKind::FlowMapping(mapping) => {
            mapping
                .pairs
                .iter()
                .rev()
                .find(|pair| yaml_node_scalar_text(source, ast, pair.key).as_deref() == Some(key))?
                .value
        }
        _ => None,
    }
}

fn yaml_mapping_scalar_pairs(
    source: &SourceBuffer,
    ast: &YamlDocumentAst<'_>,
    node: YamlNodeId,
) -> Vec<(String, String)> {
    match &ast.node(node).kind {
        YamlAstKind::Mapping(mapping) => mapping
            .pairs
            .iter()
            .filter_map(|pair| {
                let key = yaml_block_mapping_key(source, ast, pair)?;
                let value = pair.value?;
                let value = yaml_node_scalar_text(source, ast, value)?;
                Some((key, value))
            })
            .collect(),
        YamlAstKind::FlowMapping(mapping) => mapping
            .pairs
            .iter()
            .filter_map(|pair| {
                let key = yaml_node_scalar_text(source, ast, pair.key)?;
                let value = pair.value?;
                let value = yaml_node_scalar_text(source, ast, value)?;
                Some((key, value))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn yaml_block_mapping_key<'a>(
    source: &'a SourceBuffer,
    ast: &'a YamlDocumentAst,
    pair: &crate::core::yaml_model::YamlMappingPair,
) -> Option<String> {
    if let Some(key) = pair.key_node {
        return yaml_node_scalar_text(source, ast, key);
    }
    Some(front_matter_unquoted_text(source.slice(pair.key).trim()).to_owned())
}

fn yaml_node_scalar_text(
    source: &SourceBuffer,
    ast: &YamlDocumentAst<'_>,
    node: YamlNodeId,
) -> Option<String> {
    let YamlAstKind::Scalar(scalar) = &ast.node(node).kind else {
        return None;
    };
    Some(front_matter_scalar_value(source, scalar))
}

fn front_matter_scalar_value(source: &SourceBuffer, scalar: &YamlScalar<'_>) -> String {
    let value = source.slice(scalar.value).trim();
    front_matter_unquoted_text(value).trim().to_owned()
}

fn front_matter_unquoted_text(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn apply_front_matter_wrap(delta: &mut DirectiveDelta, value: &str) {
    match value {
        "none" => delta.markdown_wrap = Some(crate::core::document::MarkdownWrap::None),
        "paragraph" => {
            delta.markdown_wrap = Some(crate::core::document::MarkdownWrap::Paragraph);
        }
        "sentence" => delta.markdown_wrap = Some(crate::core::document::MarkdownWrap::Sentence),
        value => {
            if let Ok(width) = value.parse::<usize>()
                && width > 0
            {
                delta.markdown_wrap = Some(crate::core::document::MarkdownWrap::Column);
                delta.markdown_wrap_at_column = Some(width);
            }
        }
    }
}

fn parse_front_matter_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn directive_delta_has_markdown_options(delta: &DirectiveDelta) -> bool {
    delta.markdown_wrap.is_some()
        || delta.markdown_wrap_at_column.is_some()
        || delta.markdown_canonical.is_some()
        || delta.markdown_format_footnotes.is_some()
}
