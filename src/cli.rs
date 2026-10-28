//! Argument parsing and stdin/path command dispatch.

mod git_filter;

use git_filter::GitFilterCommand;

use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::builder::styling::{AnsiColor, Styles};
use clap::{
    CommandFactory, FromArgMatches, Parser, Subcommand, error::ErrorKind, parser::ValueSource,
};

use crate::core::document::{FormatOptions, MarkdownTableWidths, MarkdownWrap};
use crate::workspace::{
    FormatExecutionOptions, FormatMode, format_paths_with_trace_and_overrides,
    format_source_for_path_with_overrides, project_source_to_yaml,
};

const HELP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightBlue.on_default().bold())
    .usage(AnsiColor::BrightBlue.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::BrightBlack.on_default());
const HIDE_VERIFY_FROM_HELP: bool = !cfg!(debug_assertions);

#[derive(Debug, Parser)]
#[command(name = "yamark")]
#[command(about)]
#[command(
    long_about = concat!(
        env!("CARGO_PKG_DESCRIPTION"),
        "\n\nRun `yamark <COMMAND> --help` for command-level help."
    )
)]
#[command(styles = HELP_STYLES)]
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
        #[arg(
            long,
            hide = HIDE_VERIFY_FROM_HELP,
            help = "Reparse changed YAML and reject invalid or value-changing output"
        )]
        verify: bool,
        #[arg(long, value_name = "PATH")]
        stdin_file_path: Option<PathBuf>,
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
        #[arg(long, default_value = "72", value_parser = parse_wrap)]
        wrap: MarkdownWrap,
        #[arg(long, value_parser = parse_table_widths, help = "Table widths: fit contents or preserve source widths [default: fit]")]
        table_widths: Option<MarkdownTableWidths>,
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
    #[command(about = "Convert JSON-family input from stdin to formatted YAML")]
    ToYaml {
        #[arg(long, value_name = "PATH")]
        stdin_file_path: PathBuf,
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = 80, value_parser = parse_positive_usize)]
        line_width: usize,
        #[arg(long, default_value_t = 72, value_parser = parse_positive_usize)]
        prose_width: usize,
        #[arg(long, default_value_t = 2, value_parser = parse_positive_usize)]
        indent_width: usize,
        #[arg(long)]
        compact: bool,
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
            false,
            None,
            None,
            MarkdownWrap::Column(72),
            false,
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

    let matches = match Args::command().try_get_matches_from(args) {
        Ok(matches) => matches,
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
    let wrap_from_cli = matches
        .subcommand_matches("format")
        .is_some_and(|matches| matches.value_source("wrap") == Some(ValueSource::CommandLine));
    let args = match Args::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(err) => {
            let _ = err.print();
            return ExitCode::from(2);
        }
    };

    match args.command {
        Command::Format {
            check,
            diff,
            diagnostics,
            verify,
            stdin_file_path,
            config,
            wrap,
            table_widths,
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
            verify,
            stdin_file_path,
            config,
            wrap,
            wrap_from_cli,
            table_widths,
            canonical,
            preserve_footnotes,
            line_width,
            prose_width,
            indent_width,
            compact,
            skip_embedded_formatters,
            paths,
        ),
        Command::ToYaml {
            stdin_file_path,
            config,
            line_width,
            prose_width,
            indent_width,
            compact,
        } => run_to_yaml(
            stdin_file_path,
            config,
            line_width,
            prose_width,
            indent_width,
            compact,
        ),
        Command::GitFilter { command } => git_filter::run(command),
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
    verify: bool,
    stdin_file_path: Option<PathBuf>,
    config: Option<PathBuf>,
    wrap: MarkdownWrap,
    wrap_from_cli: bool,
    table_widths: Option<MarkdownTableWidths>,
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

    let options = FormatOptions {
        line_width,
        prose_width,
        indent_width,
        yaml_compact: compact,
        markdown_wrap: wrap,
        markdown_table_widths: table_widths.unwrap_or_default(),
        markdown_canonical: canonical,
        markdown_format_footnotes: !preserve_footnotes,
        markdown_preserve_footnotes: preserve_footnotes,
        skip_embedded_formatters,
        ..FormatOptions::default()
    };
    let markdown_wrap_override = wrap_from_cli.then_some(wrap);
    let execution = FormatExecutionOptions {
        collect_trace: diagnostics,
        verify_output: verify,
        markdown_wrap_override,
        markdown_table_widths_override: table_widths,
    };

    if let Some(path) = stdin_file_path {
        return run_stdin(path, config, options, execution);
    }

    let mode = if diff {
        FormatMode::Diff
    } else if check {
        FormatMode::Check
    } else {
        FormatMode::Write
    };
    match format_paths_with_trace_and_overrides(paths, options, mode, config, execution) {
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
    execution: FormatExecutionOptions,
) -> ExitCode {
    let input = match read_stdin_utf8() {
        Ok(input) => input,
        Err(message) => {
            eprintln!("{}:1:1: error: {message}", path.display());
            return ExitCode::from(1);
        }
    };
    let formatted =
        format_source_for_path_with_overrides(&path, input, options, config.as_deref(), execution);
    match formatted {
        Ok(formatted) => {
            if execution.collect_trace {
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

fn run_to_yaml(
    path: PathBuf,
    config: Option<PathBuf>,
    line_width: usize,
    prose_width: usize,
    indent_width: usize,
    compact: bool,
) -> ExitCode {
    let input = match read_stdin_utf8() {
        Ok(input) => input,
        Err(message) => {
            eprintln!("{}:1:1: error: {message}", path.display());
            return ExitCode::from(1);
        }
    };
    let options = FormatOptions {
        line_width,
        prose_width,
        indent_width,
        yaml_compact: compact,
        ..FormatOptions::default()
    };
    match project_source_to_yaml(&path, input, options, config.as_deref()) {
        Ok(projected) => {
            let mut stdout = io::stdout().lock();
            if let Err(err) = stdout.write_all(projected.output.as_bytes()) {
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

fn parse_wrap(value: &str) -> Result<MarkdownWrap, String> {
    MarkdownWrap::parse(value).map_err(str::to_owned)
}

fn parse_table_widths(value: &str) -> Result<MarkdownTableWidths, String> {
    MarkdownTableWidths::parse(value).map_err(str::to_owned)
}
