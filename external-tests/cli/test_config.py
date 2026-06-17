"""Run via `uv run external-tests/run.py --suite cli/test_config.py`."""

from __future__ import annotations

import pytest

from _support import format_and_check, run_cli_case


def test_compact_can_be_enabled_from_config() -> None:
    format_and_check(
        "yamark format {path}.yaml",
        "package:\n  name: yamark\n  language: rust\n",
        "package: {name: yamark, language: rust}\n",
        files={"yamark.toml": "[format]\ncompact = true\n"},
    )


def test_compact_cli_flag_overrides_disabled_config() -> None:
    format_and_check(
        "yamark format --compact {path}.yaml",
        "items:\n  - alpha\n  - beta\n",
        "items: [alpha, beta]\n",
        files={"yamark.toml": "[format]\ncompact = false\n"},
    )


def test_non_spec_markdown_hard_break_space_config_is_rejected() -> None:
    input_text = "Hard break  \nNext line\n"

    run_cli_case(
        "yamark format post.md",
        files={
            "yamark.toml": "[format]\npreserve_markdown_hard_break_spaces = true\n",
            "post.md": input_text,
        },
        expected_files={"post.md": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains=(
            "unknown format config key: format.preserve_markdown_hard_break_spaces"
        ),
    )


@pytest.mark.parametrize(
    ("config", "diagnostic"),
    [
        ("[paths]\ndocs = true\n", "paths.docs must be a table"),
        (
            "[paths.\"docs\"]\nunknown = true\n",
            "unknown path config key: paths.docs.unknown",
        ),
        (
            "[paths.\"docs\".embedded_markdown]\ntemplate = true\n",
            "paths.docs.embedded_markdown.template must be a table",
        ),
        (
            "[paths.\"docs\".template]\nadd_delimiters = [{ open = \"{\", close = \"\" }]\n",
            "paths.docs.template.add_delimiters.close must not be empty",
        ),
    ],
)
def test_invalid_path_template_config_is_rejected_before_writing(
    config: str,
    diagnostic: str,
) -> None:
    input_text = "#   Report\n"

    run_cli_case(
        "yamark format docs/post.md",
        files={
            "yamark.toml": config,
            "docs/post.md": input_text,
        },
        expected_files={"docs/post.md": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains=diagnostic,
    )


def test_invalid_root_template_delimiter_config_is_rejected_before_writing() -> None:
    input_text = "#   Report\n"

    run_cli_case(
        "yamark format post.md",
        files={
            "yamark.toml": (
                "[template]\n"
                "add_delimiters = [{ open = \"{\", close = \"\" }]\n"
            ),
            "post.md": input_text,
        },
        expected_files={"post.md": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="template.add_delimiters.close must not be empty",
    )
