"""Public table-width controls, formatting, and scope precedence."""

from __future__ import annotations

import pytest
from _support import run_cli_case

SIMPLE = "A           B\n-----  ------------\nx           y\n"
SIMPLE_FIT = "A     B\n---  ---\nx     y\n"
GRID = "+----+-----+\n|Name|Value|\n+====+=====+\n| x  | y   |\n+----+-----+\n"
GRID_FIT = (
    "+------+-------+\n| Name | Value |\n+======+=======+\n"
    "| x    | y     |\n+------+-------+\n"
)
MULTILINE = "----------  ----------\nA           B\n----------  ----------\nx           y\n----------  ----------\n"
MULTILINE_FIT = "---  ---\nA    B\n---  ---\nx    y\n---  ---\n"
PIPE = "| A        |       B |\n| :------- | ------: |\n| x        |       y |\n"
PIPE_FIT = "| A    |    B |\n| :--- | ---: |\n| x    |    y |\n"
GRID_ALIGNED = (
    "+----------+------------+\n| Name     | Value      |\n"
    "+=========:+:==========:+\n| x        | y          |\n"
    "+----------+------------+\n"
)


@pytest.mark.parametrize(
    ("source", "fitted"),
    [
        (SIMPLE, SIMPLE_FIT),
        (GRID, GRID_FIT),
        (MULTILINE, MULTILINE_FIT),
        (PIPE, PIPE_FIT),
    ],
    ids=["simple", "grid", "multiline", "pipe"],
)
@pytest.mark.parametrize("mode", [None, "fit", "preserve"])
@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_table_widths(source: str, fitted: str, mode: str | None, newline: str) -> None:
    flags = f" --table-widths {mode}" if mode else ""
    command = f"yamark format{flags} --stdin-file-path input.qmd"
    expected = (source if mode == "preserve" else fitted).replace("\n", newline)
    for text in [source.replace("\n", newline), expected]:
        run_cli_case(command, stdin=text, stdout=expected)


@pytest.mark.parametrize("mode", ["fit", "preserve"])
def test_table_widths_precedence(mode: str) -> None:
    other = "preserve" if mode == "fit" else "fit"
    expected = SIMPLE if mode == "preserve" else SIMPLE_FIT
    config = {"yamark.toml": f'[format]\ntable_widths = "{mode}"\n'}
    run_cli_case(
        "yamark format input.qmd",
        files={**config, "input.qmd": SIMPLE},
        expected_files={"input.qmd": expected},
    )
    config = {"yamark.toml": f'[format]\ntable_widths = "{other}"\n'}
    run_cli_case(
        f"yamark format --table-widths {mode} --stdin-file-path input.qmd",
        files=config,
        stdin=SIMPLE,
        stdout=expected,
    )
    frontmatter = f"---\neditor_options:\n  markdown:\n    table-widths: {mode}\n---\n"
    run_cli_case(
        f"yamark format --table-widths {other} --stdin-file-path input.qmd",
        stdin=frontmatter + SIMPLE,
        stdout=frontmatter + expected,
    )
    directive = f"<!-- fmt: table-widths={other} scope=next -->\n"
    run_cli_case(
        f"yamark format --table-widths {mode} --stdin-file-path input.qmd",
        stdin=frontmatter + directive + SIMPLE,
        stdout=frontmatter
        + directive
        + (SIMPLE if other == "preserve" else SIMPLE_FIT),
    )


def test_table_widths_next_and_from_here_scopes() -> None:
    next_table = "<!-- fmt: table-widths=preserve scope=next -->\n"
    preserve = "<!-- fmt: table-widths=preserve scope=from-here -->\n"
    fit = "<!-- fmt: table-widths=fit scope=from-here -->\n"
    source = (
        next_table
        + SIMPLE
        + "\n"
        + SIMPLE
        + "\n"
        + preserve
        + SIMPLE
        + "\n"
        + SIMPLE
        + "\n"
        + fit
        + SIMPLE
    )
    expected = (
        next_table
        + SIMPLE
        + "\n"
        + SIMPLE_FIT
        + "\n"
        + preserve
        + SIMPLE
        + "\n"
        + SIMPLE
        + "\n"
        + fit
        + SIMPLE_FIT
    )
    run_cli_case(
        "yamark format --stdin-file-path input.qmd", stdin=source, stdout=expected
    )


@pytest.mark.parametrize("scope", ["", " scope=file"])
def test_table_widths_file_scope_applies_before_directive(scope: str) -> None:
    source = SIMPLE + f"\n<!-- fmt: table-widths=preserve{scope} -->\n\n" + SIMPLE
    run_cli_case(
        "yamark format --stdin-file-path input.qmd", stdin=source, stdout=source
    )


def test_table_widths_nested_scope_stays_in_fence() -> None:
    directive = "<!-- fmt: table-widths=preserve scope=file -->\n"
    source = "```markdown\n" + directive + SIMPLE + "```\n\n" + SIMPLE
    expected = "```markdown\n" + directive + SIMPLE + "```\n\n" + SIMPLE_FIT
    run_cli_case(
        "yamark format --stdin-file-path input.qmd", stdin=source, stdout=expected
    )


def test_table_widths_preserve_still_formats_contents() -> None:
    source = SIMPLE.replace("x", "_x_").replace("y", "_y_")
    expected = "A           B\n-----  ------------\n*x*        *y*\n"
    run_cli_case(
        "yamark format --canonical --table-widths preserve --stdin-file-path input.qmd",
        stdin=source,
        stdout=expected,
    )


def test_table_widths_fit_grid_alignment() -> None:
    expected = (
        "+------+-------+\n| Name | Value |\n+=====:+:=====:+\n"
        "| x    | y     |\n+------+-------+\n"
    )
    for source in [GRID_ALIGNED, expected]:
        run_cli_case(
            "yamark format --stdin-file-path input.qmd", stdin=source, stdout=expected
        )


@pytest.mark.parametrize("suffix", ["yaml", "py", "R"])
def test_table_widths_in_marked_markdown(suffix: str) -> None:
    directive = "# fmt: markdown table-widths=preserve\n"
    if suffix == "yaml":
        source = (
            directive
            + "text: !markdown |\n"
            + "".join("  " + line for line in SIMPLE.splitlines(keepends=True))
        )
    elif suffix == "py":
        source = directive + 'text = """\n' + SIMPLE + '"""\n'
    else:
        source = directive + 'text <- r"(\n' + SIMPLE + ')"\n'
    run_cli_case(
        f"yamark format --stdin-file-path input.{suffix}", stdin=source, stdout=source
    )


@pytest.mark.parametrize(
    ("command", "files", "source", "status", "diagnostic"),
    [
        (
            "yamark format --table-widths bogus input.qmd",
            {},
            SIMPLE,
            2,
            "fit or preserve",
        ),
        (
            "yamark format input.qmd",
            {"yamark.toml": '[format]\ntable_widths = "bogus"\n'},
            SIMPLE,
            1,
            "fit or preserve",
        ),
        (
            "yamark format input.qmd",
            {},
            "<!-- fmt: table-widths=bogus -->\n" + SIMPLE,
            1,
            "fit or preserve",
        ),
    ],
)
def test_invalid_table_widths_do_not_write(
    command: str, files: dict[str, str], source: str, status: int, diagnostic: str
) -> None:
    run_cli_case(
        command,
        files={**files, "input.qmd": source},
        expected_files={"input.qmd": source},
        status=status,
        stderr=None,
        stderr_contains=diagnostic,
    )
