use crate::config::Config;
use crate::core::document::{Document, DocumentKind, FileKind, FormatOptions};
use crate::core::emit::{emit_document, emit_markdown_document};
use crate::core::source::{MAX_SOURCE_SPAN_OFFSET, SourceBuffer, Span};
use crate::diagnostic::{Result, YamarkError};
use crate::plugins::PluginRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatTrace {
    pub source_scans: usize,
    pub parse_passes: usize,
    pub source_lines: usize,
    pub yaml_scanned_lines: usize,
    pub yaml_semantic_nodes: usize,
    pub planned_rendered_scalars: usize,
    pub planned_rendered_flow_collections: usize,
    pub planned_rendered_block_flow_collections: usize,
    pub emitted_bytes: usize,
    pub emitted_nodes: usize,
}

#[derive(Debug, Clone)]
pub struct FormattedDocument {
    pub output: String,
    pub changed: bool,
    pub trace: Option<FormatTrace>,
    #[cfg(feature = "format-trace")]
    pub(crate) diagnostics: Vec<crate::diagnostic::Diagnostic>,
}

pub fn parse_source<'src>(
    source: &'src SourceBuffer,
    range: Span,
    kind: DocumentKind,
    options: FormatOptions,
    config: &Config,
) -> Result<Document<'src>> {
    validate_compact_source_range(range)?;
    match kind {
        DocumentKind::Markdown => {
            crate::core::markdown::parse_markdown(source, range, options, config)
        }
        DocumentKind::Yaml => crate::core::yaml::parse_yaml(source, range, options, config),
        DocumentKind::Python => crate::core::source_lang::parse_source_language(
            source,
            range,
            crate::core::source_lang::SourceLanguage::Python,
            options,
            config,
        ),
        DocumentKind::R => crate::core::source_lang::parse_source_language(
            source,
            range,
            crate::core::source_lang::SourceLanguage::R,
            options,
            config,
        ),
    }
}

fn parse_source_for_formatting<'src>(
    source: &'src SourceBuffer,
    range: Span,
    kind: DocumentKind,
    options: FormatOptions,
    config: &Config,
    collect_trace: bool,
) -> Result<Document<'src>> {
    validate_compact_source_range(range)?;
    match kind {
        DocumentKind::Markdown => {
            crate::core::markdown::parse_markdown_for_formatting(source, range, options, config)
        }
        DocumentKind::Yaml if collect_trace => {
            crate::core::yaml::parse_yaml_for_formatting_with_trace(source, range, options, config)
        }
        DocumentKind::Yaml => {
            crate::core::yaml::parse_yaml_for_formatting(source, range, options, config)
        }
        DocumentKind::Python => crate::core::source_lang::parse_source_language_for_formatting(
            source,
            range,
            crate::core::source_lang::SourceLanguage::Python,
            options,
            config,
        ),
        DocumentKind::R => crate::core::source_lang::parse_source_language_for_formatting(
            source,
            range,
            crate::core::source_lang::SourceLanguage::R,
            options,
            config,
        ),
    }
}

fn parse_source_for_validation<'src>(
    source: &'src SourceBuffer,
    range: Span,
    kind: DocumentKind,
    options: FormatOptions,
    config: &Config,
    yaml_node_capacity_hint: usize,
) -> Result<Document<'src>> {
    validate_compact_source_range(range)?;
    match kind {
        DocumentKind::Markdown => {
            crate::core::markdown::parse_markdown_for_validation(source, range, options, config)
        }
        DocumentKind::Yaml => crate::core::yaml::parse_yaml_for_validation(
            source,
            range,
            options,
            config,
            yaml_node_capacity_hint,
        ),
        DocumentKind::Python => crate::core::source_lang::parse_source_language_for_validation(
            source,
            range,
            crate::core::source_lang::SourceLanguage::Python,
            options,
            config,
        ),
        DocumentKind::R => crate::core::source_lang::parse_source_language_for_validation(
            source,
            range,
            crate::core::source_lang::SourceLanguage::R,
            options,
            config,
        ),
    }
}

pub(crate) fn validate_compact_source_range(range: Span) -> Result<()> {
    if range.end <= MAX_SOURCE_SPAN_OFFSET {
        return Ok(());
    }
    Err(YamarkError::new(format!(
        "source input exceeds supported maximum of {MAX_SOURCE_SPAN_OFFSET} bytes"
    )))
}

pub fn format_source(
    path_kind: FileKind,
    input: String,
    options: FormatOptions,
    config: &Config,
    plugins: &PluginRegistry,
) -> Result<String> {
    Ok(format_source_report(path_kind, input, options, config, plugins)?.output)
}

pub fn format_source_report(
    path_kind: FileKind,
    input: String,
    options: FormatOptions,
    config: &Config,
    plugins: &PluginRegistry,
) -> Result<FormattedDocument> {
    format_source_report_impl(path_kind, input, options, config, plugins, false)
}

pub fn format_source_report_with_trace(
    path_kind: FileKind,
    input: String,
    options: FormatOptions,
    config: &Config,
    plugins: &PluginRegistry,
) -> Result<FormattedDocument> {
    format_source_report_impl(path_kind, input, options, config, plugins, true)
}

fn format_source_report_impl(
    path_kind: FileKind,
    input: String,
    options: FormatOptions,
    config: &Config,
    plugins: &PluginRegistry,
    collect_trace: bool,
) -> Result<FormattedDocument> {
    let Some(kind) = DocumentKind::from_file_kind(path_kind) else {
        return Err(YamarkError::new("unsupported file type"));
    };
    validate_compact_source_range(Span::new(0, input.len()))?;
    let source = SourceBuffer::new(input);
    let range = Span::new(0, source.as_str().len());
    let document =
        parse_source_for_formatting(&source, range, kind, options, config, collect_trace)?;
    let input_trace = document.trace;
    let source_lines = source.lines.len();
    #[cfg(feature = "format-trace")]
    let diagnostics = if collect_trace {
        crate::core::format_trace::markdown_decision_diagnostics(&source, &document)
    } else {
        Vec::new()
    };
    let (mut output, yaml_emitted_nodes) = if kind == DocumentKind::Yaml {
        if document.skip_file {
            (source.slice(document.range).to_owned(), 0)
        } else {
            let mut emit_options = options;
            if !matches!(
                source.dominant_line_ending,
                crate::core::source::LineEnding::None
            ) {
                emit_options.default_line_ending = source.dominant_line_ending.as_str();
            }
            let (output, stats) = crate::core::yaml::emit_yaml_document_with_stats(
                &source,
                &document,
                emit_options,
                plugins,
            )?;
            (output, stats.emitted_nodes)
        }
    } else if kind == DocumentKind::Markdown {
        (
            emit_markdown_document(&source, &document, options, plugins)?,
            0,
        )
    } else {
        (emit_document(&source, &document, options, plugins)?, 0)
    };
    let changed = output != source.as_str();
    let mut output_parse_passes = 0;
    if changed && output_requires_yaml_validation(kind, &document, &output) {
        let document = document.retag_source_lifetime();
        let before = crate::core::yaml_equivalence::capture_yaml_validation_snapshot(
            source.into_string(),
            document,
        );
        let yaml_node_capacity_hint = before.node_capacity_hint_for_root_yaml().unwrap_or(0);
        let output_source = SourceBuffer::new(output);
        {
            let output_range = Span::new(0, output_source.as_str().len());
            let output_document = parse_source_for_validation(
                &output_source,
                output_range,
                kind,
                options,
                config,
                yaml_node_capacity_hint,
            )?;
            crate::core::yaml_equivalence::validate_yaml_documents_equivalent(
                &before,
                &output_source,
                &output_document,
            )?;
            output_parse_passes = output_document.trace.parse_passes;
        }
        output = output_source.into_string();
    }
    let trace = (kind == DocumentKind::Yaml && collect_trace).then_some(FormatTrace {
        source_scans: input_trace.source_scans,
        parse_passes: input_trace.parse_passes + output_parse_passes,
        source_lines,
        yaml_scanned_lines: input_trace.yaml_scanned_lines,
        yaml_semantic_nodes: input_trace.yaml_semantic_nodes,
        planned_rendered_scalars: input_trace.planned_rendered_scalars,
        planned_rendered_flow_collections: input_trace.planned_rendered_flow_collections,
        planned_rendered_block_flow_collections: input_trace
            .planned_rendered_block_flow_collections,
        emitted_bytes: output.len(),
        emitted_nodes: yaml_emitted_nodes,
    });
    Ok(FormattedDocument {
        output,
        changed,
        trace,
        #[cfg(feature = "format-trace")]
        diagnostics,
    })
}

fn document_contains_yaml(document: &Document<'_>) -> bool {
    document.kind == DocumentKind::Yaml || document.nested.iter().any(document_contains_yaml)
}

fn output_requires_yaml_validation(
    kind: DocumentKind,
    input_document: &Document<'_>,
    output: &str,
) -> bool {
    document_contains_yaml(input_document)
        || kind == DocumentKind::Markdown && output_starts_with_front_matter(output)
}

fn output_starts_with_front_matter(output: &str) -> bool {
    let output = output.strip_prefix('\u{feff}').unwrap_or(output);
    let line_end = output.find(['\r', '\n']).unwrap_or(output.len());
    output[..line_end].trim_end_matches([' ', '\t']) == "---"
}
