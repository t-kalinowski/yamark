use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

use crate::core::document::{FormatOptions, MarkdownWrap};
use crate::workspace::{
    FormatMode, format_paths_with_trace, format_source_for_path, format_source_for_path_with_trace,
};

#[derive(Debug, Parser)]
#[command(name = "yamark")]
#[command(about = "An ultra-fast YAML and Markdown formatter")]
#[command(
    long_about = "An ultra-fast YAML and Markdown formatter.\n\nRun `yamark <COMMAND> --help` for command-level help."
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Format {
        #[arg(long)]
        check: bool,
        #[arg(long, conflicts_with = "check")]
        diff: bool,
        #[arg(long)]
        diagnostics: bool,
        #[arg(long, value_name = "PATH")]
        stdin_file_path: Option<PathBuf>,
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
        #[arg(long, value_parser = parse_wrap)]
        wrap: Option<WrapArg>,
        #[arg(long)]
        canonical: bool,
        #[arg(long)]
        preserve_footnotes: bool,
        #[arg(long, default_value_t = 80, value_parser = parse_positive_usize)]
        line_width: usize,
        #[arg(long, default_value_t = 72, value_parser = parse_positive_usize)]
        prose_width: usize,
        #[arg(long, default_value_t = 2, value_parser = parse_positive_usize)]
        indent_width: usize,
        #[arg(long)]
        compact: bool,
        #[arg(long)]
        skip_embedded_formatters: bool,
        #[arg(value_name = "PATHS")]
        paths: Vec<PathBuf>,
    },
    #[command(
        about = "Git clean/smudge filter helpers for Markdown files",
        long_about = "\
Git clean/smudge filter helpers for Markdown files.

These commands read Markdown from stdin and write formatted Markdown to stdout
for Git attributes filters.

Configure the filter driver with:
  yamark git-filter adopt
  yamark git-filter join
  yamark git-filter check
  yamark git-filter setup
  yamark git-filter teardown
  git config filter.yamark-md.clean \"yamark git-filter clean --stdin-filename %f\"
  git config filter.yamark-md.smudge \"yamark git-filter smudge --stdin-filename %f --markdown-wrap-at-column 72\"

Git only runs the filter for paths matched by attributes. Put these patterns in
.git/info/attributes for personal use or .gitattributes for a shared repo:
  *.md filter=yamark-md
  *.qmd filter=yamark-md
  *.Rmd filter=yamark-md
  *.rmd filter=yamark-md"
    )]
    GitFilter {
        #[command(subcommand)]
        command: GitFilterCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GitFilterCommand {
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

#[derive(Debug, Clone, Copy)]
enum WrapArg {
    None,
    Paragraph,
    Sentence,
    Column(usize),
}

pub fn run<I>(args: I) -> ExitCode
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if let Some(path) = default_format_path(&args) {
        return run_format(
            false,
            false,
            false,
            None,
            None,
            None,
            false,
            false,
            80,
            72,
            2,
            false,
            false,
            vec![path],
        );
    }
    match root_help_requested(&args) {
        Ok(Some(help_is_long)) => {
            let mut command = Args::command();
            let result = if help_is_long {
                command.print_long_help()
            } else {
                command.print_help()
            };
            if let Err(err) = result {
                eprintln!("error: failed to print help: {err}");
                return ExitCode::from(1);
            }
            println!();
            return ExitCode::SUCCESS;
        }
        Ok(None) => {}
        Err(arg) => {
            let mut command = Args::command();
            let err = command.error(
                ErrorKind::UnknownArgument,
                format!("unexpected argument '{}' found", arg.to_string_lossy()),
            );
            let _ = err.print();
            return ExitCode::from(2);
        }
    }

    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(err) => {
            let code = if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                0
            } else {
                2
            };
            let _ = err.print();
            return ExitCode::from(code);
        }
    };

    match args.command {
        Command::Format {
            check,
            diff,
            diagnostics,
            stdin_file_path,
            config,
            wrap,
            canonical,
            preserve_footnotes,
            line_width,
            prose_width,
            indent_width,
            compact,
            skip_embedded_formatters,
            paths,
        } => run_format(
            check,
            diff,
            diagnostics,
            stdin_file_path,
            config,
            wrap,
            canonical,
            preserve_footnotes,
            line_width,
            prose_width,
            indent_width,
            compact,
            skip_embedded_formatters,
            paths,
        ),
        Command::GitFilter { command } => match command {
            GitFilterCommand::Clean { stdin_filename } => {
                run_git_filter(stdin_filename, MarkdownWrap::Sentence, 80)
            }
            GitFilterCommand::Smudge {
                stdin_filename,
                markdown_wrap_at_column,
            } => run_git_filter(
                stdin_filename,
                MarkdownWrap::Column,
                markdown_wrap_at_column,
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
        },
    }
}

fn default_format_path(args: &[OsString]) -> Option<PathBuf> {
    let [_, command, path] = args else {
        return None;
    };
    if command != OsStr::new("format") || path.as_os_str().to_string_lossy().starts_with('-') {
        return None;
    }
    Some(PathBuf::from(path))
}

fn root_help_requested(args: &[OsString]) -> Result<Option<bool>, OsString> {
    let mut short_help = false;
    let mut long_help = false;
    let mut unknown_flag = None;

    for arg in args.iter().skip(1) {
        if arg == OsStr::new("--help") {
            long_help = true;
            continue;
        }
        if arg == OsStr::new("-h") {
            short_help = true;
            continue;
        }
        if arg.as_os_str().to_string_lossy().starts_with('-') {
            if unknown_flag.is_none() {
                unknown_flag = Some(arg.clone());
            }
            continue;
        }
        return Ok(None);
    }

    if let Some(arg) = unknown_flag
        && (short_help || long_help)
    {
        return Err(arg);
    }

    Ok(match (long_help, short_help) {
        (true, _) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_format(
    check: bool,
    diff: bool,
    diagnostics: bool,
    stdin_file_path: Option<PathBuf>,
    config: Option<PathBuf>,
    wrap: Option<WrapArg>,
    canonical: bool,
    preserve_footnotes: bool,
    line_width: usize,
    prose_width: usize,
    indent_width: usize,
    compact: bool,
    skip_embedded_formatters: bool,
    paths: Vec<PathBuf>,
) -> ExitCode {
    if stdin_file_path.is_some() && !paths.is_empty() {
        eprintln!("error: --stdin-file-path cannot be used with PATHS");
        return ExitCode::from(2);
    }
    if stdin_file_path.is_some() && (check || diff) {
        eprintln!("error: --stdin-file-path cannot be used with --check or --diff");
        return ExitCode::from(2);
    }

    let mut options = FormatOptions {
        line_width,
        prose_width,
        indent_width,
        yaml_compact: compact,
        markdown_canonical: canonical,
        markdown_format_footnotes: !preserve_footnotes,
        markdown_preserve_footnotes: preserve_footnotes,
        skip_embedded_formatters,
        ..FormatOptions::default()
    };
    apply_wrap(wrap, &mut options);

    if let Some(path) = stdin_file_path {
        return run_stdin(path, config, options, diagnostics);
    }

    let mode = if diff {
        FormatMode::Diff
    } else if check {
        FormatMode::Check
    } else {
        FormatMode::Write
    };
    match format_paths_with_trace(paths, options, mode, config, diagnostics) {
        Ok(run) => {
            if mode == FormatMode::Diff {
                for diff in &run.diffs {
                    print!("{diff}");
                }
            }
            let summary = run.summary.render();
            if mode == FormatMode::Write {
                println!("{summary}");
            } else {
                eprintln!("{summary}");
            }
            if diagnostics || run.summary.failed > 0 {
                for diagnostic in &run.diagnostics {
                    eprintln!("{}", diagnostic.render());
                }
            }
            if run.summary.failed > 0
                || matches!(mode, FormatMode::Check | FormatMode::Diff) && run.summary.formatted > 0
            {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("{}", err.diagnostic.render());
            ExitCode::from(1)
        }
    }
}

fn run_stdin(
    path: PathBuf,
    config: Option<PathBuf>,
    options: FormatOptions,
    diagnostics: bool,
) -> ExitCode {
    let input = match read_stdin_utf8() {
        Ok(input) => input,
        Err(message) => {
            eprintln!("{}:1:1: error: {message}", path.display());
            return ExitCode::from(1);
        }
    };
    let formatted = if diagnostics {
        format_source_for_path_with_trace(&path, input, options, config.as_deref())
    } else {
        format_source_for_path(&path, input, options, config.as_deref())
    };
    match formatted {
        Ok(formatted) => {
            if diagnostics {
                for diagnostic in &formatted.diagnostics {
                    eprintln!("{}", diagnostic.render());
                }
            }
            let mut stdout = io::stdout().lock();
            if let Err(err) = stdout.write_all(formatted.output.as_bytes()) {
                eprintln!(
                    "{}:1:1: error: failed to write stdout: {err}",
                    path.display()
                );
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.diagnostic.render());
            ExitCode::from(1)
        }
    }
}

fn run_git_filter(path: PathBuf, wrap: MarkdownWrap, width: usize) -> ExitCode {
    if crate::core::document::FileKind::for_path(&path) != crate::core::document::FileKind::Markdown
    {
        eprintln!("{}:1:1: error: unsupported Git filter path", path.display());
        return ExitCode::from(1);
    }
    let options = FormatOptions {
        markdown_wrap: wrap,
        markdown_wrap_at_column: width,
        markdown_compact_tables: true,
        respect_frontmatter_markdown_options: false,
        ..FormatOptions::default()
    };
    run_stdin(path, None, options, false)
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
    write_smudged_worktree(repo_root, &targets, BlobSource::Index, width)?;
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
    write_smudged_worktree(&repo_root, &targets, BlobSource::Head, width)?;
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
    let tracked = git_stdout(repo_root, ["ls-files"])?;
    let mut paths = Vec::new();
    for path in tracked.lines() {
        if crate::core::document::FileKind::for_path(Path::new(path))
            != crate::core::document::FileKind::Markdown
        {
            continue;
        }
        let output = git_stdout_os(
            repo_root,
            &[
                OsString::from("check-attr"),
                OsString::from("filter"),
                OsString::from("--"),
                OsString::from(path),
            ],
        )?;
        if output.trim_end().ends_with(": filter: yamark-md") {
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
        let smudged = format_git_filter_text(path, blob.clone(), MarkdownWrap::Column, width)?;
        let cleaned = format_git_filter_text(path, smudged, MarkdownWrap::Sentence, 80)?;
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

fn write_smudged_worktree(
    repo_root: &Path,
    paths: &[String],
    source: BlobSource,
    width: usize,
) -> Result<(), String> {
    for path in paths {
        let blob = git_blob(repo_root, source, path)?;
        let smudged = format_git_filter_text(path, blob, MarkdownWrap::Column, width)?;
        let worktree_path = repo_root.join(path);
        fs::write(&worktree_path, smudged).map_err(|err| {
            format!(
                "{}: failed to write smudged working-tree file: {err}",
                worktree_path.display()
            )
        })?;
    }
    Ok(())
}

fn git_blob(repo_root: &Path, source: BlobSource, path: &str) -> Result<String, String> {
    let spec = match source {
        BlobSource::Head => format!("HEAD:{path}"),
        BlobSource::Index => format!(":{path}"),
    };
    git_stdout_os(repo_root, &[OsString::from("show"), OsString::from(spec)])
}

fn format_git_filter_text(
    path: &str,
    input: String,
    wrap: MarkdownWrap,
    width: usize,
) -> Result<String, String> {
    let options = FormatOptions {
        markdown_wrap: wrap,
        markdown_wrap_at_column: width,
        markdown_compact_tables: true,
        respect_frontmatter_markdown_options: false,
        ..FormatOptions::default()
    };
    format_source_for_path(Path::new(path), input, options, None)
        .map(|formatted| formatted.output)
        .map_err(|err| err.diagnostic.render())
}

fn run_git_with_paths(repo_root: &Path, prefix: &[&str], paths: &[String]) -> Result<(), String> {
    let mut args = prefix
        .iter()
        .map(|arg| OsString::from(*arg))
        .collect::<Vec<_>>();
    args.extend(paths.iter().map(OsString::from));
    run_git_os(repo_root, &args)
}

fn run_git_os(repo_root: &Path, args: &[OsString]) -> Result<(), String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format_git_error("git", output))
    }
}

fn git_stdout_os(repo_root: &Path, args: &[OsString]) -> Result<String, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if !output.status.success() {
        return Err(format_git_error("git", output));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("git output was not UTF-8: {err}"))
}

fn local_git_config_count(repo_root: &Path, key: &str) -> Result<usize, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("config")
        .arg("--local")
        .arg("--get-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stderr.is_empty() {
            return Ok(0);
        }
        return Err(format_git_error("git config", output));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| format!("git config output was not UTF-8: {err}"))
        .map(|output| output.lines().count())
}

fn count_yamark_filter_attribute_lines(path: &Path) -> Result<usize, String> {
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    Ok(existing
        .lines()
        .filter(|line| is_yamark_filter_attribute_setup_line(line))
        .count())
}

fn line_has_yamark_filter_attribute(line: &str) -> bool {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return false;
    }
    line.split_whitespace()
        .skip(1)
        .any(|field| field == "filter=yamark-md")
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

fn git_filter_attribute_patterns(
    repo_root: &Path,
    paths: Vec<PathBuf>,
) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Ok(vec![
            "*.md filter=yamark-md".to_owned(),
            "*.qmd filter=yamark-md".to_owned(),
            "*.Rmd filter=yamark-md".to_owned(),
            "*.rmd filter=yamark-md".to_owned(),
        ]);
    }

    paths
        .into_iter()
        .map(|path| {
            if crate::core::document::FileKind::for_path(&path)
                != crate::core::document::FileKind::Markdown
            {
                return Err(format!("{}: unsupported Git filter path", path.display()));
            }
            let relative = repo_relative_existing_path(repo_root, &path)?;
            let pattern = git_attribute_path_pattern(&relative)?;
            Ok(format!("{pattern} filter=yamark-md"))
        })
        .collect()
}

fn repo_relative_existing_path(repo_root: &Path, path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|err| format!("failed to read current directory: {err}"))?
            .join(path)
    };
    let absolute = fs::canonicalize(&absolute)
        .map_err(|err| format!("{}: failed to resolve path: {err}", path.display()))?;
    let repo_root = fs::canonicalize(repo_root).map_err(|err| {
        format!(
            "{}: failed to resolve repository root: {err}",
            repo_root.display()
        )
    })?;
    absolute
        .strip_prefix(&repo_root)
        .map(Path::to_path_buf)
        .map_err(|_| {
            format!(
                "{}: path is outside {}",
                path.display(),
                repo_root.display()
            )
        })
}

fn git_attribute_path_pattern(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for part in path {
        let Some(part) = part.to_str() else {
            return Err(format!("{}: path is not valid UTF-8", path.display()));
        };
        if part.chars().any(char::is_whitespace) {
            return Err(format!(
                "{}: path contains whitespace; add this Git attribute manually",
                path.display()
            ));
        }
        parts.push(part);
    }
    if parts.is_empty() {
        return Err("cannot configure a Git filter for the repository root".to_owned());
    }
    Ok(parts.join("/"))
}

fn append_missing_attribute_lines(path: &Path, lines: &[String]) -> Result<usize, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("{}: failed to create directory: {err}", parent.display()))?;
    }
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    let mut updated = existing.clone();
    let existing_lines = existing.lines().collect::<Vec<_>>();
    let mut added = 0;
    for line in lines {
        if existing_lines.iter().any(|existing| existing == line) {
            continue;
        }
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(line);
        updated.push('\n');
        added += 1;
    }
    if added > 0 {
        fs::write(path, updated)
            .map_err(|err| format!("{}: failed to write attributes: {err}", path.display()))?;
    }
    Ok(added)
}

fn remove_yamark_filter_attribute_lines(path: &Path) -> Result<usize, String> {
    let existing = match fs::read_to_string(path) {
        Ok(existing) => existing,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(format!(
                "{}: failed to read attributes: {err}",
                path.display()
            ));
        }
    };
    let mut removed = 0;
    let mut kept = Vec::new();
    for line in existing.lines() {
        if is_yamark_filter_attribute_setup_line(line) {
            removed += 1;
        } else {
            kept.push(line);
        }
    }
    if removed == 0 {
        return Ok(0);
    }
    let mut updated = kept.join("\n");
    if !updated.is_empty() && existing.ends_with('\n') {
        updated.push('\n');
    }
    fs::write(path, updated)
        .map_err(|err| format!("{}: failed to write attributes: {err}", path.display()))?;
    Ok(removed)
}

fn is_yamark_filter_attribute_setup_line(line: &str) -> bool {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return false;
    }
    let mut fields = line.split_whitespace();
    if fields.next().is_none() {
        return false;
    }
    let Some(attribute) = fields.next() else {
        return false;
    };
    attribute == "filter=yamark-md" && fields.next().is_none()
}

fn repo_relative_git_path(repo_root: &Path, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn run_git_config(repo: &Path, key: &str, value: &str) -> Result<(), String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg(key)
        .arg(value)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format_git_error("git config", output))
}

fn unset_git_config_key(repo: &Path, key: &str) -> Result<usize, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg("--get-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stderr.is_empty() {
            return Ok(0);
        }
        return Err(format_git_error("git config", output));
    }
    let count = String::from_utf8(output.stdout)
        .map_err(|err| format!("git config output was not UTF-8: {err}"))?
        .lines()
        .count();
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .arg("config")
        .arg("--local")
        .arg("--unset-all")
        .arg(key)
        .output()
        .map_err(|err| format!("failed to run git config: {err}"))?;
    if output.status.success() {
        return Ok(count);
    }
    Err(format_git_error("git config", output))
}

fn git_stdout<const N: usize>(repo: &Path, args: [&str; N]) -> Result<String, String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if !output.status.success() {
        return Err(format_git_error("git", output));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("git output was not UTF-8: {err}"))
}

fn format_git_error(command: &str, output: std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("{command} failed with status {}", output.status)
    } else {
        format!("{command} failed: {stderr}")
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

fn read_stdin_utf8() -> std::result::Result<String, String> {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|err| format!("failed to read stdin: {err}"))?;
    if input.starts_with(&[0xff, 0xfe]) || input.starts_with(&[0xfe, 0xff]) {
        return Err("unsupported encoding: UTF-16 BOM".to_owned());
    }
    String::from_utf8(input).map_err(|err| format!("invalid UTF-8: {err}"))
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|err| format!("invalid positive integer: {err}"))?;
    if parsed == 0 {
        Err("value must be greater than 0".to_owned())
    } else {
        Ok(parsed)
    }
}

fn parse_wrap(value: &str) -> Result<WrapArg, String> {
    match value {
        "none" => Ok(WrapArg::None),
        "paragraph" => Ok(WrapArg::Paragraph),
        "sentence" => Ok(WrapArg::Sentence),
        value => parse_positive_usize(value).map(WrapArg::Column),
    }
}

fn apply_wrap(wrap: Option<WrapArg>, options: &mut FormatOptions) {
    match wrap {
        Some(WrapArg::None) => options.markdown_wrap = MarkdownWrap::None,
        Some(WrapArg::Paragraph) => options.markdown_wrap = MarkdownWrap::Paragraph,
        Some(WrapArg::Sentence) => options.markdown_wrap = MarkdownWrap::Sentence,
        Some(WrapArg::Column(width)) => {
            options.markdown_wrap = MarkdownWrap::Column;
            options.markdown_wrap_at_column = width;
        }
        None => {}
    }
}
