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
    #[cfg(feature = "format-trace")]
    let diagnostics = if collect_trace {
        crate::core::format_trace::markdown_decision_diagnostics(&source, &document)
    } else {
        Vec::new()
    };
    let (output, yaml_emitted_nodes) = if kind == DocumentKind::Yaml {
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
    let trace = (kind == DocumentKind::Yaml && collect_trace).then_some(FormatTrace {
        source_scans: document.trace.source_scans,
        parse_passes: document.trace.parse_passes,
        source_lines: source.lines.len(),
        yaml_scanned_lines: document.trace.yaml_scanned_lines,
        yaml_semantic_nodes: document.trace.yaml_semantic_nodes,
        planned_rendered_scalars: document.trace.planned_rendered_scalars,
        planned_rendered_flow_collections: document.trace.planned_rendered_flow_collections,
        planned_rendered_block_flow_collections: document
            .trace
            .planned_rendered_block_flow_collections,
        emitted_bytes: output.len(),
        emitted_nodes: yaml_emitted_nodes,
    });
    let changed = output != source.as_str();
    Ok(FormattedDocument {
        output,
        changed,
        trace,
        #[cfg(feature = "format-trace")]
        diagnostics,
    })
}
