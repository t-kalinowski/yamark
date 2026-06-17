"""Run via `uv run external-tests/run.py --suite cli/test_workspace.py`."""

from __future__ import annotations

from _support import run_cli_case


def test_duplicate_explicit_files_are_not_counted_twice() -> None:
    run_cli_case(
        "yamark format config.yaml config.yaml",
        files={"config.yaml": "items: [a,b]\n"},
        expected_files={"config.yaml": "items: [a, b]\n"},
        stdout="1 files scanned, 1 formatted, 0 unchanged, 0 skipped, 0 failed\n",
    )
