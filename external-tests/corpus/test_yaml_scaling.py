"""Run via `uv run external-tests/run.py --suite corpus/test_yaml_scaling.py`."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path


def test_flow_heavy_yaml_formatting_scales_near_linearly(tmp_path: Path) -> None:
    small = measure_flow_heavy_yaml(tmp_path, 400)
    large = measure_flow_heavy_yaml(tmp_path, 1600)

    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "flow-heavy YAML formatting should scale near-linearly: "
        f"400 items used {small:.6f}s CPU, 1600 items used {large:.6f}s CPU"
    )


def measure_flow_heavy_yaml(root: Path, items: int) -> float:
    source = root / f"flow-heavy-{items}.yaml"
    source.write_text(render_flow_heavy_yaml(items), encoding="utf-8")
    log_path = root / f"yamark-{items}.log"

    # Measure this child's CPU use, excluding scheduling and I/O waits. File
    # output lets wait4 reap the child without leaving unread pipes blocked.
    with log_path.open("wb") as log, subprocess.Popen(
        [os.environ["YAMARK_BIN"], "format", os.fspath(source)],
        cwd=root,
        stdout=log,
        stderr=subprocess.STDOUT,
    ) as child:
        _, status, usage = os.wait4(child.pid, 0)
        child.returncode = os.waitstatus_to_exitcode(status)

    assert child.returncode == 0, (
        f"yamark failed with exit code {child.returncode}\n"
        f"{log_path.read_text(encoding='utf-8')}"
    )
    formatted = source.read_text(encoding="utf-8")
    assert formatted.count("ports: [8000, 9000]") == items
    return usage.ru_utime + usage.ru_stime


def render_flow_heavy_yaml(items: int) -> str:
    lines = [
        "name:    flow-heavy\n",
        "enabled: true\n",
        "labels: {team: platform,region: us-0,tier: backend}\n",
        "settings:\n",
    ]
    for index in range(items):
        lines.append(
            f"  item_{index:04}: {{name: worker-{index:04},replicas: {1 + index % 9},"
            "ports: [8000,9000],env: {LOG_LEVEL: info,FEATURE_FLAG: false},"
            f"resources: {{cpu: {100 + index % 20}m,memory: {128 + index % 12 * 32}Mi}},"
            f"dependencies: [service-{(index + 1) % 50:04},service-{(index + 7) % 50:04}]}}\n"
        )
    return "".join(lines)
