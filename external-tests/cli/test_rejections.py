"""Run via `uv run external-tests/run.py --suite cli/test_rejections.py`."""

from __future__ import annotations

import pytest

from _support import run_cli_case


def test_double_check_flag_is_rejected_and_file_is_not_written() -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        "yamark format --double-check config.yaml",
        files={"config.yaml": input_text},
        expected_files={"config.yaml": input_text},
        status=2,
        stdout="",
        stderr=None,
        stderr_contains="unexpected argument '--double-check' found",
    )


@pytest.mark.parametrize(
    ("flag", "value"),
    [
        ("--markdown-wrap", "sentence"),
        ("--markdown-wrap-at-column", "72"),
    ],
)
def test_removed_markdown_wrap_flags_are_rejected(flag: str, value: str) -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        f"yamark format {flag} {value} config.yaml",
        files={"config.yaml": input_text},
        expected_files={"config.yaml": input_text},
        status=2,
        stdout="",
        stderr=None,
        stderr_contains=flag,
    )


def test_double_check_config_key_is_rejected_and_file_is_not_written() -> None:
    input_text = "items: [a,b]\n"

    run_cli_case(
        "yamark format config.yaml",
        files={
            "yamark.toml": "[format]\ndouble_check = true\n",
            "config.yaml": input_text,
        },
        expected_files={"config.yaml": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="unknown format config key: format.double_check",
    )


def test_yaml_rejects_double_check_config_key() -> None:
    run_cli_case(
        "yamark format --config yamark.toml --stdin-file-path config.yaml",
        files={"yamark.toml": "[format]\ndouble_check = true\n"},
        stdin="items: [a,b]\n",
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="unknown format config key: format.double_check",
    )


def test_compact_sequences_config_key_is_rejected() -> None:
    input_text = "items:\n  - alpha\n  - beta\n"

    run_cli_case(
        "yamark format config.yaml",
        files={
            "yamark.toml": "[format]\ncompact_sequences = true\n",
            "config.yaml": input_text,
        },
        expected_files={"config.yaml": input_text},
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="unknown format config key: format.compact_sequences",
    )
