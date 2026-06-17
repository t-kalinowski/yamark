"""Run via `uv run external-tests/run.py --suite cli/test_help.py`."""

from __future__ import annotations

import pytest

from _support import decode_output, run_cli_case


def test_format_help_omits_hidden_compat_flag() -> None:
    result = run_cli_case("yamark format --help", stderr="")
    stdout = decode_output(result.stdout)

    assert "--double-check" not in stdout


def test_short_and_long_root_help_are_distinct() -> None:
    short = run_cli_case("yamark -h", stderr="")
    short_stdout = decode_output(short.stdout)
    assert "Usage:" in short_stdout
    assert "Run `yamark <COMMAND> --help` for command-level help." not in short_stdout

    long = run_cli_case("yamark --help", stderr="")
    long_stdout = decode_output(long.stdout)
    assert "Run `yamark <COMMAND> --help` for command-level help." in long_stdout


@pytest.mark.parametrize(
    "command",
    [
        "yamark --bogus --help",
        "yamark --bogus -h",
        "yamark -h --bogus",
    ],
)
def test_root_help_does_not_mask_unknown_root_flags(command: str) -> None:
    run_cli_case(
        command,
        status=2,
        stdout="",
        stderr=None,
        stderr_contains="--bogus",
    )
