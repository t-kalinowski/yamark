"""Run via `uv run external-tests/run.py --suite cli/test_stdin.py`."""

from __future__ import annotations

from textwrap import dedent

from _support import format_stdin_and_check, run_cli_case


def test_stdin_file_path_formats_yaml_to_stdout() -> None:
    format_stdin_and_check(
        "yamark format --stdin-file-path config.yaml",
        "items: [a,b]\n",
        "items: [a, b]\n",
        stdin_file_path="config.yaml",
    )


def test_yaml_stdin_matches_file_formatting() -> None:
    input_text = dedent(
        """\
        name: yamark
        tags: [yaml,markdown]
        table:
          - {name: alpha,value: 1}
          - {name: beta,value: 2}
        """
    )

    expected = dedent(
        """\
        name: yamark
        tags: [yaml, markdown]
        table:
          - {name: alpha, value: 1}
          - {name: beta, value: 2}
        """
    )

    format_stdin_and_check(
        "yamark format --stdin-file-path config.yaml",
        input_text,
        expected,
        stdin_file_path="config.yaml",
    )


def test_stdin_file_path_infers_markdown() -> None:
    format_stdin_and_check(
        "yamark format --stdin-file-path post.md",
        "#   Title\n",
        "# Title\n",
        stdin_file_path="post.md",
    )


def test_canonical_flag_applies_to_stdin_file_path() -> None:
    format_stdin_and_check(
        "yamark format --canonical --stdin-file-path post.md",
        "_Text_ stays semantic.\n",
        "*Text* stays semantic.\n",
        stdin_file_path="post.md",
    )


def test_stdin_file_path_rejects_additional_paths() -> None:
    run_cli_case(
        "yamark format --stdin-file-path config.yaml other.yaml",
        stdin="items: [a,b]\n",
        status=2,
        stdout="",
        stderr=None,
        stderr_contains="--stdin-file-path cannot be used with PATHS",
    )
