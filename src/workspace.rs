//! Source formatting and ordered application of workspace outcomes.

mod diff;
mod paths;

use paths::{format_single_explicit_file, format_streaming_candidates};

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Config, discover_config_path};
use crate::core::document::{FileKind, FormatOptions, MarkdownTableWidths, MarkdownWrap};
use crate::core::parser::format_source_report_with_policy;
use crate::diagnostic::{Diagnostic, Result, YamarkError};
use crate::json_to_yaml::{JsonSourceKind, json_to_yaml_source};
use crate::plugins::PluginRegistry;

#[derive(Debug, Clone)]
pub struct FormattedSource {
    pub output: String,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct ProjectedSource {
    pub output: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    Write,
    Check,
    Diff,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FormatExecutionOptions {
    pub(crate) collect_trace: bool,
    pub(crate) verify_output: bool,
    pub(crate) markdown_wrap_override: Option<MarkdownWrap>,
    pub(crate) markdown_table_widths_override: Option<MarkdownTableWidths>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatSummary {
    pub scanned: usize,
    pub formatted: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl FormatSummary {
    pub fn render(&self) -> String {
        format!(
            "{} files scanned, {} formatted, {} unchanged, {} skipped, {} failed",
            self.scanned, self.formatted, self.unchanged, self.skipped, self.failed
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct FormatRun {
    pub summary: FormatSummary,
    pub diffs: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
struct IndexedOutcome {
    index: usize,
    path: PathBuf,
    result: FileFormatResult,
}

#[derive(Debug)]
enum FileFormatResult {
    Skipped,
    Failed(Diagnostic),
    Unchanged {
        diagnostics: Vec<Diagnostic>,
    },
    Changed {
        diagnostics: Vec<Diagnostic>,
        change: FileChange,
    },
}

#[derive(Debug)]
enum FileChange {
    Check,
    Write(String),
    Diff(String),
}

pub fn format_source_for_path(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
) -> Result<FormattedSource> {
    format_source_for_path_with_overrides(
        path,
        input,
        options,
        config_path,
        FormatExecutionOptions::default(),
    )
}

pub fn format_source_for_path_with_trace(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
) -> Result<FormattedSource> {
    format_source_for_path_with_overrides(
        path,
        input,
        options,
        config_path,
        FormatExecutionOptions {
            collect_trace: true,
            ..FormatExecutionOptions::default()
        },
    )
}

pub fn project_source_to_yaml(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
) -> Result<ProjectedSource> {
    let Some(json_kind) = JsonSourceKind::for_path(path) else {
        return Err(YamarkError::new(
            "JSON-to-YAML projection requires a .json, .jsonc, .json5, .jsonl, or .ndjson source path",
        )
        .with_path(path));
    };
    let mut config = load_config_for_formatted_path(path, config_path)?;
    // JSON-family input cannot contain active YAML templates. Delimiter text
    // in the generated YAML is data and must not preserve a flow collection.
    config.template_delimiters.clear();
    let yaml_input = json_to_yaml_source(&input, json_kind).map_err(|err| err.with_path(path))?;
    drop(input);
    let mut options = options;
    options.skip_embedded_formatters = true;
    let formatted = format_source_with_config_as(
        path,
        yaml_input,
        FileKind::Yaml,
        options,
        &config,
        FormatExecutionOptions::default(),
    )?;
    Ok(ProjectedSource {
        output: formatted.output,
        diagnostics: formatted.diagnostics,
    })
}

pub(crate) fn format_source_for_path_with_overrides(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
    execution: FormatExecutionOptions,
) -> Result<FormattedSource> {
    let config = load_config_for_formatted_path(path, config_path)?;
    format_source_with_config(path, input, options, &config, execution)
}

fn format_source_with_config(
    path: &Path,
    input: String,
    options: FormatOptions,
    config: &Config,
    execution: FormatExecutionOptions,
) -> Result<FormattedSource> {
    let kind = FileKind::for_path(path);
    format_source_with_config_as(path, input, kind, options, config, execution)
}

fn format_source_with_config_as(
    path: &Path,
    input: String,
    kind: FileKind,
    options: FormatOptions,
    config: &Config,
    execution: FormatExecutionOptions,
) -> Result<FormattedSource> {
    if input.as_bytes().starts_with(&[0xff, 0xfe]) || input.as_bytes().starts_with(&[0xfe, 0xff]) {
        return Err(YamarkError::new("unsupported encoding: UTF-16 BOM").with_path(path));
    }
    if !kind.is_supported() {
        return Err(YamarkError::new("unsupported file type").with_path(path));
    }
    let options = apply_config_options(options, config, execution);
    let plugins = PluginRegistry::from_config(config).with_source_path(path);
    let formatted = format_source_report_with_policy(
        kind,
        input,
        options,
        config,
        &plugins,
        execution.collect_trace,
        execution.verify_output,
    )
    .map_err(|err| err.with_path(path))?;
    #[cfg(feature = "format-trace")]
    let trace_diagnostics = formatted.diagnostics;
    let changed = formatted.changed;
    let output = formatted.output;
    let mut diagnostics = plugins.diagnostics();
    #[cfg(feature = "format-trace")]
    diagnostics.extend(
        trace_diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_path(path)),
    );
    if let Some(trace) = formatted.trace {
        diagnostics.push(
            Diagnostic::note(format!(
                "yaml trace: source_scans={} parse_passes={} source_lines={} yaml_scanned_lines={} yaml_semantic_nodes={} planned_rendered_scalars={} planned_rendered_flow_collections={} planned_rendered_block_flow_collections={} emitted_bytes={} emitted_nodes={}",
                trace.source_scans,
                trace.parse_passes,
                trace.source_lines,
                trace.yaml_scanned_lines,
                trace.yaml_semantic_nodes,
                trace.planned_rendered_scalars,
                trace.planned_rendered_flow_collections,
                trace.planned_rendered_block_flow_collections,
                trace.emitted_bytes,
                trace.emitted_nodes
            ))
            .with_path(path),
        );
    }
    Ok(FormattedSource {
        changed,
        output,
        diagnostics,
    })
}

pub fn format_paths(
    paths: Vec<PathBuf>,
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<PathBuf>,
) -> Result<FormatRun> {
    format_paths_with_trace(paths, options, mode, config_path, false)
}

pub fn format_paths_with_trace(
    paths: Vec<PathBuf>,
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<PathBuf>,
    collect_trace: bool,
) -> Result<FormatRun> {
    format_paths_with_trace_and_overrides(
        paths,
        options,
        mode,
        config_path,
        FormatExecutionOptions {
            collect_trace,
            ..FormatExecutionOptions::default()
        },
    )
}

pub(crate) fn format_paths_with_trace_and_overrides(
    paths: Vec<PathBuf>,
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<PathBuf>,
    execution: FormatExecutionOptions,
) -> Result<FormatRun> {
    if let Some(result) =
        format_single_explicit_file(&paths, options, mode, config_path.as_deref(), execution)
    {
        let outcome = result?;
        let mut run = FormatRun::default();
        run.summary.scanned = 1;
        apply_format_outcome(&mut run, outcome)?;
        sort_diagnostics(&mut run.diagnostics);
        return Ok(run);
    }

    let (mut outcomes, scanned) =
        format_streaming_candidates(paths, options, mode, config_path.as_deref(), execution)?;
    outcomes.sort_by_key(|outcome| outcome.index);

    let mut run = FormatRun::default();
    run.summary.scanned = scanned;

    for outcome in outcomes {
        apply_format_outcome(&mut run, outcome)?;
    }
    sort_diagnostics(&mut run.diagnostics);
    Ok(run)
}

fn apply_format_outcome(run: &mut FormatRun, outcome: IndexedOutcome) -> Result<()> {
    match outcome.result {
        FileFormatResult::Skipped => run.summary.skipped += 1,
        FileFormatResult::Failed(diagnostic) => {
            run.summary.failed += 1;
            run.diagnostics.push(diagnostic);
        }
        FileFormatResult::Unchanged { diagnostics } => {
            run.diagnostics.extend(diagnostics);
            run.summary.unchanged += 1;
        }
        FileFormatResult::Changed {
            diagnostics,
            change,
        } => {
            run.diagnostics.extend(diagnostics);
            run.summary.formatted += 1;
            match change {
                FileChange::Write(output) => fs::write(&outcome.path, output).map_err(|err| {
                    YamarkError::from(
                        Diagnostic::error(format!("failed to write file: {err}"))
                            .with_path(&outcome.path),
                    )
                })?,
                FileChange::Check => {}
                FileChange::Diff(diff) => run.diffs.push(diff),
            }
        }
    }
    Ok(())
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            left.path.as_ref(),
            left.line,
            left.column,
            severity_order(&left.severity),
            &left.message,
        )
            .cmp(&(
                right.path.as_ref(),
                right.line,
                right.column,
                severity_order(&right.severity),
                &right.message,
            ))
    });
}

fn load_config_for_formatted_path(path: &Path, explicit: Option<&Path>) -> Result<Config> {
    if let Some(config_path) = explicit {
        return Config::from_path(config_path).map(|config| config.for_formatted_path(path));
    }
    if let Some(config_path) = discover_config_path(path) {
        return Config::from_path(&config_path).map(|config| config.for_formatted_path(path));
    }
    Ok(Config::default())
}

fn apply_config_options(
    mut options: FormatOptions,
    config: &Config,
    execution: FormatExecutionOptions,
) -> FormatOptions {
    if let Some(compact) = config.format.compact
        && (compact || !options.yaml_compact)
    {
        options.yaml_compact = compact;
    }
    if let Some(wrap) = config.format.markdown_wrap {
        options.markdown_wrap = wrap;
    }
    if let Some(marker) = config.format.markdown_horizontal_rule {
        options.markdown_horizontal_rule = marker;
    }
    if let Some(wrap) = execution.markdown_wrap_override {
        options.markdown_wrap = wrap;
    }
    if let Some(widths) = execution
        .markdown_table_widths_override
        .or(config.format.markdown_table_widths)
    {
        options.markdown_table_widths = widths;
    }
    options
}

fn severity_order(severity: &crate::diagnostic::Severity) -> u8 {
    match severity {
        crate::diagnostic::Severity::Error => 0,
        crate::diagnostic::Severity::Note => 1,
    }
}
