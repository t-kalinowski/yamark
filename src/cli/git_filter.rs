//! Git clean/smudge commands and repository adoption lifecycle.

mod attributes;
mod process;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use clap::Subcommand;

use super::{parse_positive_usize, run_stdin};
use crate::core::document::{FormatOptions, MarkdownWrap};
use crate::workspace::{FormatExecutionOptions, format_source_for_path_with_overrides};
use attributes::{
    append_missing_attribute_lines, count_yamark_filter_attribute_lines,
    git_filter_attribute_patterns, line_has_yamark_filter_attribute,
    remove_yamark_filter_attribute_lines,
};
use process::{
    git_stdout, git_stdout_os, local_git_config_count, run_git_config, run_git_os,
    run_git_with_paths, unset_git_config_key,
};

#[derive(Debug, Subcommand)]
pub(super) enum GitFilterCommand {
    Clean {
        #[arg(long)]
        stdin_filename: PathBuf,
    },
    Smudge {
        #[arg(long)]
        stdin_filename: PathBuf,
        #[arg(long, default_value_t = 72, value_parser = parse_positive_usize)]
        markdown_wrap_at_column: usize,
    },
    #[command(about = "Adopt the yamark Git filter for a shared repository")]
    Adopt {
        #[arg(
            long,
            value_name = "PATH",
            help = "Repository path; defaults to the current repository"
        )]
        repo: Option<PathBuf>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Yamark executable stored in the filter command; defaults to this executable"
        )]
        yamark: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = 72,
            value_parser = parse_positive_usize,
            help = "Working-tree column width used by the smudge filter"
        )]
        markdown_wrap_at_column: usize,
        #[arg(
            value_name = "PATHS",
            help = "Specific Markdown paths to filter; defaults to Markdown file patterns"
        )]
        paths: Vec<PathBuf>,
    },
    #[command(about = "Join a repository that has already adopted the yamark Git filter")]
    Join {
        #[arg(
            long,
            value_name = "PATH",
            help = "Repository path; defaults to the current repository"
        )]
        repo: Option<PathBuf>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Yamark executable stored in the filter command; defaults to this executable"
        )]
        yamark: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = 72,
            value_parser = parse_positive_usize,
            help = "Working-tree column width used by the smudge filter"
        )]
        markdown_wrap_at_column: usize,
    },
    #[command(about = "Check committed yamark Git filter blobs round-trip safely")]
    Check {
        #[arg(
            long,
            value_name = "PATH",
            help = "Repository path; defaults to the current repository"
        )]
        repo: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = 72,
            value_parser = parse_positive_usize,
            help = "Working-tree column width used by the smudge filter"
        )]
        markdown_wrap_at_column: usize,
    },
    #[command(about = "Configure the yamark Git filter in a repository")]
    Setup {
        #[arg(
            long,
            value_name = "PATH",
            help = "Repository path; defaults to the current repository"
        )]
        repo: Option<PathBuf>,
        #[arg(
            long,
            value_name = "PATH",
            help = "Yamark executable stored in the filter command; defaults to this executable"
        )]
        yamark: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = 72,
            value_parser = parse_positive_usize,
            help = "Working-tree column width used by the smudge filter"
        )]
        markdown_wrap_at_column: usize,
        #[arg(
            value_name = "PATHS",
            help = "Specific Markdown paths to filter; defaults to Markdown file patterns"
        )]
        paths: Vec<PathBuf>,
    },
    #[command(about = "Remove the local yamark Git filter setup from a repository")]
    Teardown {
        #[arg(
            long,
            value_name = "PATH",
            help = "Repository path; defaults to the current repository"
        )]
        repo: Option<PathBuf>,
    },
}

pub(super) fn run(command: GitFilterCommand) -> ExitCode {
    match command {
        GitFilterCommand::Clean { stdin_filename } => {
            run_git_filter(stdin_filename, MarkdownWrap::Sentence)
        }
        GitFilterCommand::Smudge {
            stdin_filename,
            markdown_wrap_at_column,
        } => run_git_filter(
            stdin_filename,
            MarkdownWrap::Column(markdown_wrap_at_column),
        ),
        GitFilterCommand::Adopt {
            repo,
            yamark,
            markdown_wrap_at_column,
            paths,
        } => run_git_filter_adopt(repo, yamark, markdown_wrap_at_column, paths),
        GitFilterCommand::Join {
            repo,
            yamark,
            markdown_wrap_at_column,
        } => run_git_filter_join(repo, yamark, markdown_wrap_at_column),
        GitFilterCommand::Check {
            repo,
            markdown_wrap_at_column,
        } => run_git_filter_check(repo, markdown_wrap_at_column),
        GitFilterCommand::Setup {
            repo,
            yamark,
            markdown_wrap_at_column,
            paths,
        } => run_git_filter_setup(repo, yamark, markdown_wrap_at_column, paths),
        GitFilterCommand::Teardown { repo } => run_git_filter_teardown(repo),
    }
}

fn run_git_filter(path: PathBuf, wrap: MarkdownWrap) -> ExitCode {
    if crate::core::document::FileKind::for_path(&path) != crate::core::document::FileKind::Markdown
    {
        eprintln!("{}:1:1: error: unsupported Git filter path", path.display());
        return ExitCode::from(1);
    }
    let options = FormatOptions {
        markdown_wrap: wrap,
        markdown_compact_tables: true,
        respect_frontmatter_markdown_options: false,
        ..FormatOptions::default()
    };
    run_stdin(
        path,
        None,
        options,
        FormatExecutionOptions {
            markdown_wrap_override: Some(wrap),
            ..FormatExecutionOptions::default()
        },
    )
}

fn run_git_filter_adopt(
    repo: Option<PathBuf>,
    yamark: Option<PathBuf>,
    width: usize,
    paths: Vec<PathBuf>,
) -> ExitCode {
    match adopt_git_filter(repo, yamark, width, paths) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run_git_filter_join(repo: Option<PathBuf>, yamark: Option<PathBuf>, width: usize) -> ExitCode {
    match join_git_filter(repo, yamark, width) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run_git_filter_check(repo: Option<PathBuf>, width: usize) -> ExitCode {
    match check_git_filter(repo, width) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run_git_filter_setup(
    repo: Option<PathBuf>,
    yamark: Option<PathBuf>,
    width: usize,
    paths: Vec<PathBuf>,
) -> ExitCode {
    match setup_git_filter(repo, yamark, width, paths) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run_git_filter_teardown(repo: Option<PathBuf>) -> ExitCode {
    match teardown_git_filter(repo) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn setup_git_filter(
    repo: Option<PathBuf>,
    yamark: Option<PathBuf>,
    width: usize,
    paths: Vec<PathBuf>,
) -> Result<String, String> {
    let (repo_root, attributes_path) = git_filter_repo_paths(repo)?;
    configure_git_filter(&repo_root, yamark, width)?;

    let patterns = git_filter_attribute_patterns(&repo_root, paths)?;
    let added = append_missing_attribute_lines(&attributes_path, &patterns)?;
    Ok(format!(
        "Configured yamark Git filter in {}\nUpdated {} with {added} new attribute pattern(s)\n\nNext step:\n  git add --renormalize .\n\nUndo:\n  Temporarily stage one file exactly as it appears in the working tree:\n    git -c filter.yamark-md.clean=cat add <path>\n  Permanently remove this local setup:\n    yamark git-filter teardown\n\nPer-file control:\n  Add later Git attribute lines to .git/info/attributes or .gitattributes:\n    NEWS.md -filter\n  Then use the normal formatter for that file, for example:\n    yamark format --wrap {width} NEWS.md\n  Use <!-- fmt: skip file --> inside a Markdown file to leave its contents untouched.",
        repo_root.display(),
        attributes_path.display(),
    ))
}

fn adopt_git_filter(
    repo: Option<PathBuf>,
    yamark: Option<PathBuf>,
    width: usize,
    paths: Vec<PathBuf>,
) -> Result<String, String> {
    let (repo_root, info_attributes_path) = git_filter_repo_paths(repo)?;
    ensure_clean_worktree(&repo_root)?;
    ensure_no_local_yamark_filter_setup(&repo_root, &info_attributes_path)?;

    let attributes_path = repo_root.join(".gitattributes");
    let previous_attributes = read_optional_file(&attributes_path)?;
    let result = adopt_git_filter_inner(&repo_root, yamark, width, paths, &attributes_path);
    if let Err(err) = result {
        let rollback = rollback_adopt(&repo_root, &attributes_path, previous_attributes);
        return match rollback {
            Ok(()) => Err(format!("{err}\nRolled back yamark Git filter adoption.")),
            Err(rollback_err) => Err(format!(
                "{err}\nRollback failed: {rollback_err}\nReview the repository before continuing."
            )),
        };
    }
    result
}

fn adopt_git_filter_inner(
    repo_root: &Path,
    yamark: Option<PathBuf>,
    width: usize,
    paths: Vec<PathBuf>,
    attributes_path: &Path,
) -> Result<String, String> {
    configure_git_filter(repo_root, yamark, width)?;
    let patterns = git_filter_attribute_patterns(repo_root, paths)?;
    let added = append_missing_attribute_lines(attributes_path, &patterns)?;
    let targets = tracked_yamark_filter_paths(repo_root)?;
    if targets.is_empty() {
        return Err("no tracked Markdown files match filter=yamark-md".to_owned());
    }

    run_git_os(
        repo_root,
        &[
            OsString::from("add"),
            OsString::from("--"),
            OsString::from(".gitattributes"),
        ],
    )?;
    run_git_with_paths(repo_root, &["add", "--renormalize", "--"], &targets)?;
    verify_index_roundtrip(repo_root, &targets, width)?;
    write_smudged_worktree(repo_root, &targets)?;
    ensure_no_unstaged_changes(repo_root)?;

    Ok(format!(
        "Adopted yamark Git filter in {}\nUpdated .gitattributes with {added} new attribute pattern(s)\nStaged .gitattributes and {} normalized Markdown file(s).\n\nReview and commit with:\n  git diff --cached\n  git commit -m \"Normalize Markdown with yamark Git filter\"",
        repo_root.display(),
        targets.len(),
    ))
}

fn join_git_filter(
    repo: Option<PathBuf>,
    yamark: Option<PathBuf>,
    width: usize,
) -> Result<String, String> {
    let (repo_root, _) = git_filter_repo_paths(repo)?;
    ensure_clean_worktree(&repo_root)?;
    ensure_not_behind_upstream(&repo_root)?;
    ensure_shared_yamark_attributes(&repo_root)?;
    configure_git_filter(&repo_root, yamark, width)?;

    let targets = tracked_yamark_filter_paths(&repo_root)?;
    if targets.is_empty() {
        return Err("no tracked Markdown files match filter=yamark-md".to_owned());
    }
    verify_head_roundtrip(&repo_root, &targets, width)?;
    write_smudged_worktree(&repo_root, &targets)?;
    ensure_clean_worktree(&repo_root)?;

    Ok(format!(
        "Joined yamark Git filter in {}\nConfigured local filter driver and checked {} committed Markdown file(s).",
        repo_root.display(),
        targets.len(),
    ))
}

fn check_git_filter(repo: Option<PathBuf>, width: usize) -> Result<String, String> {
    let (repo_root, _) = git_filter_repo_paths(repo)?;
    ensure_shared_yamark_attributes(&repo_root)?;
    let targets = tracked_yamark_filter_paths(&repo_root)?;
    if targets.is_empty() {
        return Err("no tracked Markdown files match filter=yamark-md".to_owned());
    }
    verify_head_roundtrip(&repo_root, &targets, width)?;
    Ok(format!(
        "yamark Git filter check passed for {} Markdown file(s).",
        targets.len()
    ))
}

fn teardown_git_filter(repo: Option<PathBuf>) -> Result<String, String> {
    let (repo_root, attributes_path) = git_filter_repo_paths(repo)?;
    let unset = unset_git_filter_config(&repo_root)?;
    let removed = remove_yamark_filter_attribute_lines(&attributes_path)?;
    Ok(format!(
        "Removed yamark Git filter setup from {}\nUnset {unset} local Git config value(s)\nRemoved {removed} yamark attribute pattern(s) from {}\n\nTracked files keep their current index content until staged again. Run:\n  git add --renormalize .\nif you want to restage them without the filter.",
        repo_root.display(),
        attributes_path.display(),
    ))
}

#[derive(Clone, Copy)]
enum BlobSource {
    Head,
    Index,
}

fn configure_git_filter(
    repo_root: &Path,
    yamark: Option<PathBuf>,
    width: usize,
) -> Result<(), String> {
    let yamark = match yamark {
        Some(path) => path,
        None => env::current_exe()
            .map_err(|err| format!("failed to locate current yamark executable: {err}"))?,
    };
    let yamark = shell_quote_path(&yamark)?;

    run_git_config(
        repo_root,
        "filter.yamark-md.clean",
        &format!("{yamark} git-filter clean --stdin-filename %f"),
    )?;
    run_git_config(
        repo_root,
        "filter.yamark-md.smudge",
        &format!(
            "{yamark} git-filter smudge --stdin-filename %f --markdown-wrap-at-column {width}"
        ),
    )?;
    run_git_config(repo_root, "filter.yamark-md.required", "true")?;
    run_git_config(repo_root, "merge.renormalize", "true")
}

fn ensure_clean_worktree(repo_root: &Path) -> Result<(), String> {
    let status = git_stdout(repo_root, ["status", "--porcelain"])?;
    if status.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "working tree is not clean; commit or stash changes first\n{status}"
        ))
    }
}

fn ensure_no_unstaged_changes(repo_root: &Path) -> Result<(), String> {
    let diff = git_stdout(repo_root, ["diff", "--name-only"])?;
    if diff.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "checkout form is not clean after smudge\nunstaged files:\n{diff}"
        ))
    }
}

fn ensure_no_local_yamark_filter_setup(
    repo_root: &Path,
    info_attributes_path: &Path,
) -> Result<(), String> {
    for key in [
        "filter.yamark-md.clean",
        "filter.yamark-md.smudge",
        "filter.yamark-md.required",
        "merge.renormalize",
    ] {
        if local_git_config_count(repo_root, key)? > 0 {
            return Err(format!(
                "existing local Git filter config {key} is set; run yamark git-filter teardown first"
            ));
        }
    }
    let local_attribute_lines = count_yamark_filter_attribute_lines(info_attributes_path)?;
    if local_attribute_lines > 0 {
        return Err(format!(
            "{} already contains yamark filter attributes; run yamark git-filter teardown first",
            info_attributes_path.display()
        ));
    }
    Ok(())
}

fn ensure_not_behind_upstream(repo_root: &Path) -> Result<(), String> {
    let upstream = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("--symbolic-full-name")
        .arg("@{upstream}")
        .output()
        .map_err(|err| format!("failed to run git rev-parse: {err}"))?;
    if !upstream.status.success() {
        return Ok(());
    }

    let behind = git_stdout(repo_root, ["rev-list", "--count", "HEAD..@{upstream}"])?;
    let behind = behind
        .trim()
        .parse::<usize>()
        .map_err(|err| format!("git rev-list output was not a count: {err}"))?;
    if behind > 0 {
        return Err("branch is behind upstream. Run git pull --ff-only first, then run yamark git-filter join.".to_owned());
    }
    Ok(())
}

fn ensure_shared_yamark_attributes(repo_root: &Path) -> Result<(), String> {
    let attributes_path = repo_root.join(".gitattributes");
    let attributes = fs::read_to_string(&attributes_path).map_err(|err| {
        format!(
            "{}: failed to read shared Git attributes: {err}",
            attributes_path.display()
        )
    })?;
    if attributes.lines().any(line_has_yamark_filter_attribute) {
        Ok(())
    } else {
        Err(format!(
            "{} does not contain filter=yamark-md; run yamark git-filter adopt in the maintainer checkout first",
            attributes_path.display()
        ))
    }
}

fn tracked_yamark_filter_paths(repo_root: &Path) -> Result<Vec<String>, String> {
    // NUL records preserve Unicode and embedded line breaks without Git's
    // presentation quoting. Each check-attr response is path, name, then value.
    let tracked = git_stdout(repo_root, ["ls-files", "-z"])?;
    let mut paths = Vec::new();
    for path in tracked.split_terminator('\0') {
        if crate::core::document::FileKind::for_path(Path::new(path))
            != crate::core::document::FileKind::Markdown
        {
            continue;
        }
        let output = git_stdout_os(
            repo_root,
            &[
                OsString::from("check-attr"),
                OsString::from("-z"),
                OsString::from("filter"),
                OsString::from("--"),
                OsString::from(path),
            ],
        )?;
        if output.split_terminator('\0').nth(2) == Some("yamark-md") {
            paths.push(path.to_owned());
        }
    }
    Ok(paths)
}

fn verify_head_roundtrip(repo_root: &Path, paths: &[String], width: usize) -> Result<(), String> {
    verify_roundtrip(repo_root, paths, BlobSource::Head, width)
}

fn verify_index_roundtrip(repo_root: &Path, paths: &[String], width: usize) -> Result<(), String> {
    verify_roundtrip(repo_root, paths, BlobSource::Index, width)
}

fn verify_roundtrip(
    repo_root: &Path,
    paths: &[String],
    source: BlobSource,
    width: usize,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for path in paths {
        let blob = git_blob(repo_root, source, path)?;
        let smudged = format_git_filter_text(path, blob.clone(), MarkdownWrap::Column(width))?;
        let cleaned = format_git_filter_text(path, smudged, MarkdownWrap::Sentence)?;
        if cleaned != blob {
            failures.push(format!(
                "{path}: roundtrip failure: clean(smudge(blob)) != blob"
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn write_smudged_worktree(repo_root: &Path, paths: &[String]) -> Result<(), String> {
    // Let Git apply text/eol attributes and core.autocrlf before the configured
    // smudge filter. Writing formatted blob bytes directly bypasses that step.
    for path in paths {
        let smudged = git_stdout_os(
            repo_root,
            &[
                OsString::from("cat-file"),
                OsString::from("--filters"),
                OsString::from(format!(":{path}")),
            ],
        )?;
        let worktree_path = repo_root.join(path);
        fs::write(&worktree_path, smudged).map_err(|err| {
            format!(
                "{}: failed to write smudged working-tree file: {err}",
                worktree_path.display()
            )
        })?;
    }
    // Refresh Git's cached file metadata as part of checkout. A plain write can
    // leave a CRLF file marked dirty even when its cleaned blob is unchanged.
    run_git_with_paths(
        repo_root,
        &["checkout-index", "--force", "--index", "--"],
        paths,
    )
}

fn git_blob(repo_root: &Path, source: BlobSource, path: &str) -> Result<String, String> {
    let spec = match source {
        BlobSource::Head => format!("HEAD:{path}"),
        BlobSource::Index => format!(":{path}"),
    };
    git_stdout_os(repo_root, &[OsString::from("show"), OsString::from(spec)])
}

fn format_git_filter_text(path: &str, input: String, wrap: MarkdownWrap) -> Result<String, String> {
    let options = FormatOptions {
        markdown_wrap: wrap,
        markdown_compact_tables: true,
        respect_frontmatter_markdown_options: false,
        ..FormatOptions::default()
    };
    format_source_for_path_with_overrides(
        Path::new(path),
        input,
        options,
        None,
        FormatExecutionOptions {
            markdown_wrap_override: Some(wrap),
            ..FormatExecutionOptions::default()
        },
    )
    .map(|formatted| formatted.output)
    .map_err(|err| err.diagnostic.render())
}

fn read_optional_file(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("{}: failed to read file: {err}", path.display())),
    }
}

fn restore_optional_file(path: &Path, contents: Option<String>) -> Result<(), String> {
    match contents {
        Some(contents) => fs::write(path, contents)
            .map_err(|err| format!("{}: failed to restore file: {err}", path.display())),
        None => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!("{}: failed to remove file: {err}", path.display())),
        },
    }
}

fn rollback_adopt(
    repo_root: &Path,
    attributes_path: &Path,
    previous_attributes: Option<String>,
) -> Result<(), String> {
    let mut errors = Vec::new();
    for args in [
        vec![
            OsString::from("reset"),
            OsString::from("-q"),
            OsString::from("--mixed"),
            OsString::from("HEAD"),
        ],
        vec![
            OsString::from("checkout"),
            OsString::from("-q"),
            OsString::from("--"),
            OsString::from("."),
        ],
    ] {
        if let Err(err) = run_git_os(repo_root, &args) {
            errors.push(err);
        }
    }
    if let Err(err) = restore_optional_file(attributes_path, previous_attributes) {
        errors.push(err);
    }
    if let Err(err) = unset_git_filter_config(repo_root) {
        errors.push(err);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn unset_git_filter_config(repo_root: &Path) -> Result<usize, String> {
    [
        "filter.yamark-md.clean",
        "filter.yamark-md.smudge",
        "filter.yamark-md.required",
        "merge.renormalize",
    ]
    .into_iter()
    .map(|key| unset_git_config_key(repo_root, key))
    .try_fold(0, |sum, removed| removed.map(|removed| sum + removed))
}

fn git_filter_repo_paths(repo: Option<PathBuf>) -> Result<(PathBuf, PathBuf), String> {
    let start = match repo {
        Some(repo) => repo,
        None => {
            env::current_dir().map_err(|err| format!("failed to read current directory: {err}"))?
        }
    };
    let repo_root = git_stdout(&start, ["rev-parse", "--show-toplevel"])?;
    let repo_root = PathBuf::from(repo_root.trim_end());
    let attributes_path = git_stdout(&repo_root, ["rev-parse", "--git-path", "info/attributes"])?;
    let attributes_path = repo_relative_git_path(&repo_root, attributes_path.trim_end());
    Ok((repo_root, attributes_path))
}

fn repo_relative_git_path(repo_root: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn shell_quote_path(path: &Path) -> Result<String, String> {
    let Some(value) = path.to_str() else {
        return Err(format!("{}: path is not valid UTF-8", path.display()));
    };
    Ok(shell_quote(value))
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | '+'))
    {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
