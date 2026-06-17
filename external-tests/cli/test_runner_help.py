"""Run via `uv run external-tests/run.py --suite cli/test_runner_help.py`."""

from __future__ import annotations

import subprocess
from pathlib import Path


def test_external_runner_help_describes_suite_boundaries() -> None:
    repo = Path(__file__).resolve().parents[2]
    result = subprocess.run(
        [
            "uv",
            "run",
            "--no-project",
            "--script",
            "external-tests/run.py",
            "--help",
        ],
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )

    assert result.returncode == 0
    assert "Public suite directories: cli, corpus, smoke, snapshots." in result.stdout
    assert "--suite PATH" in result.stdout
