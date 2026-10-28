//! File discovery, per-directory configuration caching, and worker scheduling.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use ignore::WalkBuilder;

use super::diff::simple_diff;
use super::{
    FileChange, FileFormatResult, FormatExecutionOptions, FormatMode, IndexedOutcome,
    format_source_with_config, load_config_for_formatted_path,
};
use crate::config::{Config, discover_config_path};
use crate::core::document::{FileKind, FormatOptions};
use crate::diagnostic::{Diagnostic, Result, YamarkError};

#[derive(Debug, Clone)]
struct FormatCandidate {
    path: PathBuf,
    kind: FileKind,
    config: Option<Config>,
}

#[derive(Debug)]
struct FormatJob {
    index: usize,
    candidate: FormatCandidate,
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

pub(super) fn format_single_explicit_file(
    paths: &[PathBuf],
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<&Path>,
    execution: FormatExecutionOptions,
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
        Ok(format_candidate(0, &candidate, options, mode, execution))
    })();
    Some(result)
}

pub(super) fn format_streaming_candidates(
    paths: Vec<PathBuf>,
    options: FormatOptions,
    mode: FormatMode,
    config_path: Option<&Path>,
    execution: FormatExecutionOptions,
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
                        format_candidate(job.index, &job.candidate, options, mode, execution);
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
    execution: FormatExecutionOptions,
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
                match format_source_with_config(&candidate.path, input, options, config, execution)
                {
                    Ok(formatted) if formatted.changed => FileFormatResult::Changed {
                        diagnostics: formatted.diagnostics,
                        change: match mode {
                            FormatMode::Check => FileChange::Check,
                            FormatMode::Write => FileChange::Write(formatted.output),
                            FormatMode::Diff => FileChange::Diff(simple_diff(
                                &candidate.path,
                                diff_input
                                    .as_deref()
                                    .expect("diff mode keeps original input"),
                                &formatted.output,
                            )),
                        },
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
