"""Run via `uv run external-tests/run.py --suite cli/test_check_diff.py`."""

from __future__ import annotations

from textwrap import dedent

from _support import run_cli_case


def test_check_reports_would_change_without_writing() -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        "yamark format --check config.yaml",
        files={"config.yaml": input_text},
        expected_files={"config.yaml": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="1 formatted",
    )


def test_check_succeeds_when_files_are_unchanged() -> None:
    run_cli_case(
        "yamark format --check config.yaml",
        files={"config.yaml": "items: [a, b]\n"},
        stdout="",
        stderr=None,
        stderr_contains="1 unchanged",
    )


def test_diff_prints_unified_diff_without_writing() -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        "yamark format --diff config.yaml",
        files={"config.yaml": input_text},
        expected_files={"config.yaml": input_text},
        status=1,
        stdout_contains=[
            "--- config.yaml\n+++ config.yaml\n",
            "-items: [a,b]\n+items: [a, b]\n",
        ],
        stderr=None,
        stderr_contains="1 formatted",
    )


def test_diff_succeeds_without_output_when_files_are_unchanged() -> None:
    input_text = "items: [a, b]\n"

    run_cli_case(
        "yamark format --diff config.yaml",
        files={"config.yaml": input_text},
        expected_files={"config.yaml": input_text},
        stdout="",
        stderr=None,
        stderr_contains="1 unchanged",
    )


def test_diff_treats_yaml_decline_as_unchanged_without_writing() -> None:
    input_text = "key: [\n"

    run_cli_case(
        "yamark format --diff bad.yaml",
        files={"bad.yaml": input_text},
        expected_files={"bad.yaml": input_text},
        stdout="",
        stderr=None,
        stderr_contains=["1 unchanged", "0 failed"],
    )


def test_diff_prints_successful_diffs_when_another_file_is_unchanged() -> None:
    changed_input = "items: [a,b]\n"
    bad_input = "key: [\n"

    run_cli_case(
        "yamark format --diff changed.yaml bad.yaml",
        files={
            "changed.yaml": changed_input,
            "bad.yaml": bad_input,
        },
        expected_files={
            "changed.yaml": changed_input,
            "bad.yaml": bad_input,
        },
        status=1,
        stdout_contains=[
            "--- changed.yaml\n+++ changed.yaml\n",
            "-items: [a,b]\n+items: [a, b]\n",
        ],
        stderr=None,
        stderr_contains=[
            "2 files scanned",
            "1 formatted",
            "1 unchanged",
            "0 failed",
        ],
    )


def test_yaml_check_and_diff_modes_do_not_write() -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        "yamark format --check check.yaml",
        files={"check.yaml": input_text},
        expected_files={"check.yaml": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="1 formatted",
    )
    run_cli_case(
        "yamark format --diff diff.yaml",
        files={"diff.yaml": input_text},
        expected_files={"diff.yaml": input_text},
        status=1,
        stdout_contains="-items: [a,b]\n+items: [a, b]\n",
        stderr=None,
        stderr_contains="1 formatted",
    )


def test_markdown_check_does_not_write() -> None:
    input_text = "#   Title\n"

    run_cli_case(
        "yamark format --check post.md",
        files={"post.md": input_text},
        expected_files={"post.md": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="1 formatted",
    )


def test_markdown_diff_accepts_explicit_config_without_writing() -> None:
    input_text = "#   Title\n"

    run_cli_case(
        "yamark format --diff --config {root}/yamark.toml post.md",
        files={
            "yamark.toml": "[format]\nmarkdown_horizontal_rule = \"***\"\n",
            "post.md": input_text,
        },
        expected_files={"post.md": input_text},
        status=1,
        stdout_contains="-#   Title\n+# Title\n",
        stderr=None,
        stderr_contains="1 formatted",
    )


def test_markdown_frontmatter_check_and_diff_use_yaml() -> None:
    input_text = dedent(
        """\
        ---
        tags: [quarto,markdown]
        editor_options:
          markdown:
            wrap: 32
        ---

        Body
        """
    )

    run_cli_case(
        "yamark format --check check.md",
        files={"check.md": input_text},
        expected_files={"check.md": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="1 formatted",
    )
    run_cli_case(
        "yamark format --diff diff.md",
        files={"diff.md": input_text},
        expected_files={"diff.md": input_text},
        status=1,
        stdout_contains="-tags: [quarto,markdown]\n+tags: [quarto, markdown]\n",
        stderr=None,
        stderr_contains="1 formatted",
    )
