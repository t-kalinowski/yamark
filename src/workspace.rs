use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use ignore::WalkBuilder;

use crate::config::{Config, discover_config_path};
use crate::core::document::{FileKind, FormatOptions};
use crate::core::parser::{format_source_report, format_source_report_with_trace};
use crate::diagnostic::{Diagnostic, Result, YamarkError};
use crate::plugins::PluginRegistry;

#[derive(Debug, Clone)]
pub struct FormattedSource {
    pub output: String,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    Write,
    Check,
    Diff,
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

#[derive(Debug, Clone)]
struct FormatCandidate {
    path: PathBuf,
    kind: FileKind,
    config: Option<Config>,
}

#[derive(Debug)]
struct IndexedOutcome {
    index: usize,
    path: PathBuf,
    result: FileFormatResult,
}

#[derive(Debug)]
struct FormatJob {
    index: usize,
    candidate: FormatCandidate,
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
        output: String,
        diff_input: Option<String>,
    },
}

pub fn format_source_for_path(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
) -> Result<FormattedSource> {
    let config = load_config_for_formatted_path(path, config_path)?;
    format_source_with_config(path, input, options, &config, false)
}

pub fn format_source_for_path_with_trace(
    path: &Path,
    input: String,
    options: FormatOptions,
    config_path: Option<&Path>,
) -> Result<FormattedSource> {
    let config = load_config_for_formatted_path(path, config_path)?;
    format_source_with_config(path, input, options, &config, true)
}

fn format_source_with_config(
    path: &Path,
    input: String,
    options: FormatOptions,
    config: &Config,
    collect_trace: bool,
) -> Result<FormattedSource> {
    if input.as_bytes().starts_with(&[0xff, 0xfe]) || input.as_bytes().starts_with(&[0xfe, 0xff]) {
        return Err(YamarkError::new("unsupported encoding: UTF-16 BOM").with_path(path));
    }
    let kind = FileKind::for_path(path);
    if !kind.is_supported() {
        return Err(YamarkError::new("unsupported file type").with_path(path));
    }
    let options = apply_config_options(options, config);
    let plugins = PluginRegistry::from_config(config).with_source_path(path);
    let formatted = if collect_trace {
        format_source_report_with_trace(kind, input, options, config, &plugins)
    } else {
        format_source_report(kind, input, options, config, &plugins)
    }
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
    if let Some(result) =
        format_single_explicit_file(&paths, options, mode, config_path.as_deref(), collect_trace)
    {
        let outcome = result?;
        let mut run = FormatRun::default();
        run.summary.scanned = 1;
        apply_format_outcome(&mut run, outcome, mode)?;
        sort_diagnostics(&mut run.diagnostics);
        return Ok(run);
    }

    let (mut outcomes, scanned) =
        format_streaming_candidates(paths, options, mode, config_path.as_deref(), collect_trace)?;
    outcomes.sort_by_key(|outcome| outcome.index);

    let mut run = FormatRun::default();
    run.summary.scanned = scanned;

    for outcome in outcomes {
        apply_format_outcome(&mut run, outcome, mode)?;
    }
    sort_diagnostics(&mut run.diagnostics);
    Ok(run)
}

fn apply_format_outcome(
    run: &mut FormatRun,
    outcome: IndexedOutcome,
    mode: FormatMode,
) -> Result<()> {
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
            output,
            diff_input,
        } => {
            run.diagnostics.extend(diagnostics);
            run.summary.formatted += 1;
            match mode {
                FormatMode::Write => fs::write(&outcome.path, output).map_err(|err| {
                    YamarkError::from(
                        Diagnostic::error(format!("failed to write file: {err}"))
                            .with_path(&outcome.path),
                    )
                })?,
                FormatMode::Check => {}
                FormatMode::Diff => {
                    let input = diff_input
                        .as_deref()
                        .expect("diff mode keeps original input");
                    run.diffs.push(simple_diff(&outcome.path, input, &output));
                }
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

fn format_worker_count() -> usize {
    std::thread::available_parallelism().map_or(1, |count| count.get())
}

fn format_worker_count_for_roots(paths: &[PathBuf]) -> usize {
    let max_workers = format_worker_count();
    if paths.is_empty() || !paths.iter().all(|path| path_is_file_like(path)) {
        return max_workers;
    }
    max_workers.min(paths.len()).max(1)
}

fn path_is_file_like(path: &Path) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return true;
    };
    let file_type = metadata.file_type();
    if file_type.is_file() {
        return true;
    }
    if file_type.is_symlink() {
        return fs::metadata(path).is_ok_and(|target| target.is_file());
    }
    false
}

fn format_single_explicit_file(
    paths: &[PathBuf],
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<&Path>,
    collect_trace: bool,
) -> Option<Result<IndexedOutcome>> {
    let [path] = paths else {
        return None;
    };
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => {
            return Some(Err(YamarkError::from(
                Diagnostic::error("path does not exist").with_path(path),
            )));
        }
    };
    let file_type = metadata.file_type();
    let is_file = if file_type.is_symlink() {
        match fs::metadata(path) {
            Ok(target) if target.is_file() => true,
            Ok(target) if target.is_dir() => return None,
            Ok(_) => false,
            Err(_) => {
                return Some(Err(YamarkError::from(
                    Diagnostic::error("path does not exist").with_path(path),
                )));
            }
        }
    } else if file_type.is_file() {
        true
    } else if file_type.is_dir() {
        return None;
    } else {
        false
    };
    if !is_file {
        return Some(Err(YamarkError::from(
            Diagnostic::error("path is not a file or directory").with_path(path),
        )));
    }

    let kind = FileKind::for_path(path);
    let result = (|| {
        let config = if kind.is_supported() {
            Some(load_config_for_formatted_path(path, config_path)?)
        } else {
            None
        };
        let candidate = FormatCandidate {
            path: path.clone(),
            kind,
            config,
        };
        Ok(format_candidate(
            0,
            &candidate,
            options,
            mode,
            collect_trace,
        ))
    })();
    Some(result)
}

fn format_streaming_candidates(
    paths: Vec<PathBuf>,
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<&Path>,
    collect_trace: bool,
) -> Result<(Vec<IndexedOutcome>, usize)> {
    let worker_count = format_worker_count_for_roots(&paths);
    let queue_capacity = worker_count.saturating_mul(2).max(1);
    let (job_sender, job_receiver) = mpsc::sync_channel::<FormatJob>(queue_capacity);
    let (outcome_sender, outcome_receiver) = mpsc::channel::<IndexedOutcome>();
    let job_receiver = Arc::new(Mutex::new(job_receiver));

    let mut stream_result = Ok(0usize);
    thread::scope(|scope| {
        for _ in 0..worker_count {
            let job_receiver = Arc::clone(&job_receiver);
            let outcome_sender = outcome_sender.clone();
            scope.spawn(move || {
                loop {
                    let job = {
                        let receiver = job_receiver
                            .lock()
                            .expect("formatter work queue lock was poisoned");
                        receiver.recv()
                    };
                    let Ok(job) = job else {
                        break;
                    };
                    let outcome =
                        format_candidate(job.index, &job.candidate, options, mode, collect_trace);
                    if outcome_sender.send(outcome).is_err() {
                        break;
                    }
                }
            });
        }
        drop(outcome_sender);

        stream_result = match CandidateProducer::new(config_path, job_sender) {
            Ok(mut producer) => match producer.stream(paths) {
                Ok(()) => Ok(producer.scanned()),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        };
    });

    let scanned = stream_result?;
    Ok((outcome_receiver.into_iter().collect(), scanned))
}

struct CandidateProducer {
    explicit_config: Option<Config>,
    config_cache: BTreeMap<PathBuf, Config>,
    seen: BTreeSet<PathBuf>,
    sender: mpsc::SyncSender<FormatJob>,
    next_index: usize,
}

impl CandidateProducer {
    fn new(explicit: Option<&Path>, sender: mpsc::SyncSender<FormatJob>) -> Result<Self> {
        Ok(Self {
            explicit_config: explicit.map(Config::from_path).transpose()?,
            config_cache: BTreeMap::new(),
            seen: BTreeSet::new(),
            sender,
            next_index: 0,
        })
    }

    fn stream(&mut self, paths: Vec<PathBuf>) -> Result<()> {
        let roots = if paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            paths
        };
        for root in roots {
            self.stream_root(root)?;
        }
        Ok(())
    }

    fn stream_root(&mut self, root: PathBuf) -> Result<()> {
        let metadata = fs::symlink_metadata(&root).map_err(|_| {
            YamarkError::from(Diagnostic::error("path does not exist").with_path(&root))
        })?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            let target = fs::metadata(&root).map_err(|_| {
                YamarkError::from(Diagnostic::error("path does not exist").with_path(&root))
            })?;
            if target.is_file() {
                self.push_candidate(root)?;
            } else if target.is_dir() {
                self.stream_directory(&root)?;
            }
        } else if file_type.is_file() {
            self.push_candidate(root)?;
        } else if file_type.is_dir() {
            self.stream_directory(&root)?;
        } else {
            return Err(YamarkError::from(
                Diagnostic::error("path is not a file or directory").with_path(&root),
            ));
        }
        Ok(())
    }

    fn stream_directory(&mut self, root: &Path) -> Result<()> {
        let mut builder = WalkBuilder::new(root);
        builder.follow_links(false);
        builder.sort_by_file_name(|left, right| left.cmp(right));
        for entry in builder.build() {
            let entry = entry
                .map_err(|err| YamarkError::new(format!("failed to walk directory: {err}")))?;
            if entry.file_type().is_some_and(|ty| ty.is_file()) {
                self.push_candidate(entry.path().to_path_buf())?;
            }
        }
        Ok(())
    }

    fn push_candidate(&mut self, path: PathBuf) -> Result<()> {
        let key = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !self.seen.insert(key) {
            return Ok(());
        }
        let kind = FileKind::for_path(&path);
        let config = if kind.is_supported() {
            Some(self.config_for_path(&path)?)
        } else {
            None
        };
        let index = self.next_index;
        self.next_index += 1;
        self.sender
            .send(FormatJob {
                index,
                candidate: FormatCandidate { path, kind, config },
            })
            .map_err(|_| YamarkError::new("formatter worker queue closed"))?;
        Ok(())
    }

    fn config_for_path(&mut self, path: &Path) -> Result<Config> {
        if let Some(config) = &self.explicit_config {
            return Ok(config.for_formatted_path(path));
        }
        let Some(config_path) = discover_config_path(path) else {
            return Ok(Config::default().for_formatted_path(path));
        };
        if !self.config_cache.contains_key(&config_path) {
            self.config_cache
                .insert(config_path.clone(), Config::from_path(&config_path)?);
        }
        Ok(self
            .config_cache
            .get(&config_path)
            .expect("config was cached")
            .for_formatted_path(path))
    }

    fn scanned(&self) -> usize {
        self.next_index
    }
}

fn format_candidate(
    index: usize,
    candidate: &FormatCandidate,
    options: FormatOptions,
    mode: FormatMode,
    collect_trace: bool,
) -> IndexedOutcome {
    let path = candidate.path.clone();
    let result = if !candidate.kind.is_supported() {
        FileFormatResult::Skipped
    } else {
        match read_utf8_file(&candidate.path) {
            Ok(input) => {
                let diff_input = (mode == FormatMode::Diff).then(|| input.clone());
                let config = candidate
                    .config
                    .as_ref()
                    .expect("supported candidates have a resolved config");
                match format_source_with_config(
                    &candidate.path,
                    input,
                    options,
                    config,
                    collect_trace,
                ) {
                    Ok(formatted) if formatted.changed => FileFormatResult::Changed {
                        diagnostics: formatted.diagnostics,
                        output: formatted.output,
                        diff_input,
                    },
                    Ok(formatted) => FileFormatResult::Unchanged {
                        diagnostics: formatted.diagnostics,
                    },
                    Err(err) => FileFormatResult::Failed(err.diagnostic),
                }
            }
            Err(err) => FileFormatResult::Failed(
                Diagnostic::error(format!("failed to read file: {err}")).with_path(&candidate.path),
            ),
        }
    };
    IndexedOutcome {
        index,
        path,
        result,
    }
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

fn apply_config_options(mut options: FormatOptions, config: &Config) -> FormatOptions {
    if let Some(compact) = config.format.compact
        && (compact || !options.yaml_compact)
    {
        options.yaml_compact = compact;
    }
    if let Some(marker) = config.format.markdown_horizontal_rule {
        options.markdown_horizontal_rule = marker;
    }
    options
}

fn read_utf8_file(path: &Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported encoding: UTF-16 BOM",
        ));
    }
    String::from_utf8(bytes).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid UTF-8: {err}"),
        )
    })
}

fn severity_order(severity: &crate::diagnostic::Severity) -> u8 {
    match severity {
        crate::diagnostic::Severity::Error => 0,
        crate::diagnostic::Severity::Note => 1,
    }
}

fn simple_diff(path: &Path, before: &str, after: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("--- {}\n+++ {}\n", path.display(), path.display()));
    let before_lines = diff_lines(before);
    let after_lines = diff_lines(after);
    let ops = diff_ops(&before_lines, &after_lines);
    for (start, end) in diff_hunks(&ops, 3) {
        let (old_start, old_count, new_start, new_count) = hunk_ranges(&ops, start, end);
        out.push_str(&format!(
            "@@ -{} +{} @@\n",
            unified_range(old_start, old_count),
            unified_range(new_start, new_count)
        ));
        for op in &ops[start..end] {
            match op {
                DiffOp::Equal(line) => push_diff_line(&mut out, ' ', *line),
                DiffOp::Remove(line) => push_diff_line(&mut out, '-', *line),
                DiffOp::Add(line) => push_diff_line(&mut out, '+', *line),
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DiffLine<'a> {
    text: &'a str,
    has_line_ending: bool,
}

#[derive(Debug, Clone, Copy)]
enum DiffOp<'a> {
    Equal(DiffLine<'a>),
    Remove(DiffLine<'a>),
    Add(DiffLine<'a>),
}

fn diff_lines(input: &str) -> Vec<DiffLine<'_>> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(DiffLine {
                    text: &input[start..i],
                    has_line_ending: true,
                });
                i += 1;
                start = i;
            }
            b'\r' => {
                lines.push(DiffLine {
                    text: &input[start..i],
                    has_line_ending: true,
                });
                i += if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    2
                } else {
                    1
                };
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < input.len() {
        lines.push(DiffLine {
            text: &input[start..],
            has_line_ending: false,
        });
    }
    lines
}

fn push_diff_line(out: &mut String, prefix: char, line: DiffLine<'_>) {
    out.push(prefix);
    out.push_str(line.text);
    out.push('\n');
    if !line.has_line_ending {
        out.push_str("\\ No newline at end of file\n");
    }
}

fn diff_ops<'a>(before: &[DiffLine<'a>], after: &[DiffLine<'a>]) -> Vec<DiffOp<'a>> {
    const MAX_LCS_CELLS: usize = 4_000_000;
    if before
        .len()
        .checked_mul(after.len())
        .is_none_or(|cells| cells > MAX_LCS_CELLS)
    {
        return linear_diff_ops(before, after);
    }

    let mut lcs = vec![vec![0usize; after.len() + 1]; before.len() + 1];
    for i in (0..before.len()).rev() {
        for j in (0..after.len()).rev() {
            lcs[i][j] = if before[i] == after[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;
    while i < before.len() || j < after.len() {
        if i < before.len() && j < after.len() && before[i] == after[j] {
            ops.push(DiffOp::Equal(before[i]));
            i += 1;
            j += 1;
        } else if i < before.len() && (j == after.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
            ops.push(DiffOp::Remove(before[i]));
            i += 1;
        } else {
            ops.push(DiffOp::Add(after[j]));
            j += 1;
        }
    }
    ops
}

fn linear_diff_ops<'a>(before: &[DiffLine<'a>], after: &[DiffLine<'a>]) -> Vec<DiffOp<'a>> {
    let mut prefix = 0usize;
    while prefix < before.len() && prefix < after.len() && before[prefix] == after[prefix] {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < before.len().saturating_sub(prefix)
        && suffix < after.len().saturating_sub(prefix)
        && before[before.len() - 1 - suffix] == after[after.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let before_end = before.len() - suffix;
    let after_end = after.len() - suffix;
    let mut ops = Vec::with_capacity(before.len() + after.len());
    for line in &before[..prefix] {
        ops.push(DiffOp::Equal(*line));
    }

    let mut i = prefix;
    let mut j = prefix;
    let mut sync_index = None;
    while i < before_end || j < after_end {
        if i < before_end && j < after_end && before[i] == after[j] {
            ops.push(DiffOp::Equal(before[i]));
            i += 1;
            j += 1;
            continue;
        }

        if i == before_end {
            for line in &after[j..after_end] {
                ops.push(DiffOp::Add(*line));
            }
            break;
        }
        if j == after_end {
            for line in &before[i..before_end] {
                ops.push(DiffOp::Remove(*line));
            }
            break;
        }

        if let Some((remove_count, add_count)) =
            next_linear_diff_sync(before, after, i, j, before_end, after_end, &mut sync_index)
        {
            for line in &before[i..i + remove_count] {
                ops.push(DiffOp::Remove(*line));
            }
            for line in &after[j..j + add_count] {
                ops.push(DiffOp::Add(*line));
            }
            i += remove_count;
            j += add_count;
        } else {
            for line in &before[i..before_end] {
                ops.push(DiffOp::Remove(*line));
            }
            for line in &after[j..after_end] {
                ops.push(DiffOp::Add(*line));
            }
            break;
        }
    }

    for line in &before[before_end..] {
        ops.push(DiffOp::Equal(*line));
    }
    ops
}

fn next_linear_diff_sync<'a>(
    before: &[DiffLine<'a>],
    after: &[DiffLine<'a>],
    before_start: usize,
    after_start: usize,
    before_end: usize,
    after_end: usize,
    sync_index: &mut Option<LinearDiffSyncIndex<'a>>,
) -> Option<(usize, usize)> {
    const SYNC_WINDOW: usize = 64;
    let before = &before[before_start..before_end];
    let after = &after[after_start..after_end];
    if let Some(sync) = next_bounded_linear_diff_sync(before, after, SYNC_WINDOW) {
        return Some(sync);
    }

    let sync_index = sync_index.get_or_insert_with(|| LinearDiffSyncIndex::new(after, after_start));
    sync_index.next_sync(before, after_start)
}

struct LinearDiffSyncIndex<'a> {
    after_positions: HashMap<DiffLine<'a>, Vec<usize>>,
}

impl<'a> LinearDiffSyncIndex<'a> {
    fn new(after: &[DiffLine<'a>], after_start: usize) -> Self {
        let mut after_positions: HashMap<DiffLine<'a>, Vec<usize>> =
            HashMap::with_capacity(after.len());
        for (offset, line) in after.iter().copied().enumerate() {
            after_positions
                .entry(line)
                .or_default()
                .push(after_start + offset);
        }
        Self { after_positions }
    }

    fn next_sync(&self, before: &[DiffLine<'a>], after_start: usize) -> Option<(usize, usize)> {
        let mut best = None;
        for (remove_count, line) in before.iter().copied().enumerate() {
            if let Some((best_remove, best_add)) = best
                && remove_count > best_remove + best_add
            {
                break;
            }

            let Some(positions) = self.after_positions.get(&line) else {
                continue;
            };
            let index = positions.partition_point(|&position| position < after_start);
            let Some(&position) = positions.get(index) else {
                continue;
            };

            let add_count = position - after_start;
            let distance = remove_count + add_count;
            if distance == 0 {
                continue;
            }
            if best.is_none_or(|(best_remove, best_add)| {
                let best_distance = best_remove + best_add;
                distance < best_distance || distance == best_distance && remove_count < best_remove
            }) {
                best = Some((remove_count, add_count));
            }
        }
        best
    }
}

fn next_bounded_linear_diff_sync(
    before: &[DiffLine<'_>],
    after: &[DiffLine<'_>],
    window: usize,
) -> Option<(usize, usize)> {
    let max_before = before.len().saturating_sub(1).min(window);
    let max_after = after.len().saturating_sub(1).min(window);

    for distance in 1..=max_before + max_after {
        let remove_min = distance.saturating_sub(max_after);
        let remove_max = max_before.min(distance);
        for (remove_count, before_line) in before
            .iter()
            .enumerate()
            .take(remove_max + 1)
            .skip(remove_min)
        {
            let add_count = distance - remove_count;
            if add_count <= max_after && *before_line == after[add_count] {
                return Some((remove_count, add_count));
            }
        }
    }
    None
}

fn diff_hunks(ops: &[DiffOp<'_>], context: usize) -> Vec<(usize, usize)> {
    let mut hunks = Vec::new();
    let mut index = 0usize;
    while let Some(change) = next_change(ops, index) {
        let mut start = change;
        let mut context_before = 0usize;
        while start > 0 && matches!(ops[start - 1], DiffOp::Equal(_)) && context_before < context {
            start -= 1;
            context_before += 1;
        }

        let mut end = change + 1;
        let mut trailing_context = 0usize;
        while end < ops.len() {
            if matches!(ops[end], DiffOp::Equal(_)) {
                if trailing_context == context {
                    break;
                }
                trailing_context += 1;
            } else {
                trailing_context = 0;
            }
            end += 1;
        }
        hunks.push((start, end));
        index = end;
    }
    hunks
}

fn next_change(ops: &[DiffOp<'_>], start: usize) -> Option<usize> {
    ops.iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, op)| (!matches!(op, DiffOp::Equal(_))).then_some(index))
}

fn hunk_ranges(ops: &[DiffOp<'_>], start: usize, end: usize) -> (usize, usize, usize, usize) {
    let mut old_line = 1usize;
    let mut new_line = 1usize;
    for op in &ops[..start] {
        match op {
            DiffOp::Equal(_) => {
                old_line += 1;
                new_line += 1;
            }
            DiffOp::Remove(_) => old_line += 1,
            DiffOp::Add(_) => new_line += 1,
        }
    }

    let mut old_count = 0usize;
    let mut new_count = 0usize;
    for op in &ops[start..end] {
        match op {
            DiffOp::Equal(_) => {
                old_count += 1;
                new_count += 1;
            }
            DiffOp::Remove(_) => old_count += 1,
            DiffOp::Add(_) => new_count += 1,
        }
    }
    (old_line, old_count, new_line, new_count)
}

fn unified_range(start: usize, count: usize) -> String {
    match count {
        0 => format!("{start},0"),
        1 => start.to_string(),
        count => format!("{start},{count}"),
    }
}
