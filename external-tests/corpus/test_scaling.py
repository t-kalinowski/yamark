"""Run via `uv run external-tests/run.py --suite corpus/test_scaling.py`."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

pytestmark = pytest.mark.skipif(
    not hasattr(os, "wait4"), reason="Child CPU accounting requires os.wait4"
)


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
    cpu, formatted = measure_formatting_cpu(source)
    assert formatted.count("ports: [8000, 9000]") == items
    return cpu


def test_unmatched_backticks_scale_with_input_size(tmp_path: Path) -> None:
    durations = []
    for runs in [400, 800]:
        text = "Before " + " ".join("`" * size + "x" for size in range(1, runs + 1))
        text += " <% keep   this %>\n"
        source = tmp_path / f"backticks-{runs}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == text
        durations.append(cpu)

    # Doubling the number of runs quadruples the input size. Repeated suffix
    # searches instead do roughly eight times as much work.
    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "unmatched backtick scanning should scale with input size: "
        f"400 runs used {small:.6f}s CPU, 800 runs used {large:.6f}s CPU"
    )


@pytest.mark.parametrize(
    ("label", "suffix"),
    [
        ("label", " <% keep   this %>"),
        ("{{ foo }}", ""),
        ("`<% keep   this %>`", ""),
    ],
    ids=["template-after-label", "template-in-label", "template-in-code"],
)
@pytest.mark.parametrize("opening", ["[", "[outer "])
def test_nested_brackets_scale_with_input_size(
    tmp_path: Path, label: str, suffix: str, opening: str
) -> None:
    durations = []
    for depth in [2000, 8000]:
        text = opening * depth + label + "]" * depth + suffix + "\n"
        source = tmp_path / f"brackets-{depth}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == text
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "nested bracket scanning should scale with input size: "
        f"2000 pairs used {small:.6f}s CPU, 8000 pairs used {large:.6f}s CPU"
    )


def test_incomplete_html_before_templates_scales_with_input_size(
    tmp_path: Path,
) -> None:
    durations = []
    for count in [20_000, 80_000]:
        text = "Before " + "<!-- " * count + "{{ foo }}\n"
        source = tmp_path / f"incomplete-html-{count}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == text
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "incomplete HTML before templates should scale with input size: "
        f"20000 openers used {small:.6f}s CPU, 80000 openers used {large:.6f}s CPU"
    )


@pytest.mark.parametrize("opening", ["{ ", "{{ "])
def test_unmatched_braces_in_inline_html_scale_with_input_size(
    tmp_path: Path, opening: str
) -> None:
    durations = []
    for count in [2000, 8000]:
        text = "Before <span>" + opening * count + "</span>\n"
        source = tmp_path / f"html-braces-{count}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == "Before\n<span>" + opening * count + "</span>\n"
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "unmatched braces in inline HTML should scale with input size: "
        f"2000 openers used {small:.6f}s CPU, 8000 openers used {large:.6f}s CPU"
    )


def measure_formatting_cpu(source: Path) -> tuple[float, str]:
    log_path = source.with_suffix(".log")

    # Measure this child's CPU use, excluding scheduling and I/O waits. File
    # output lets wait4 reap the child without leaving unread pipes blocked.
    with log_path.open("wb") as log, subprocess.Popen(
        [os.environ["YAMARK_BIN"], "format", os.fspath(source)],
        cwd=source.parent,
        stdout=log,
        stderr=subprocess.STDOUT,
    ) as child:
        _, status, usage = os.wait4(child.pid, 0)
        child.returncode = os.waitstatus_to_exitcode(status)

    assert child.returncode == 0, (
        f"yamark failed with exit code {child.returncode}\n"
        f"{log_path.read_text(encoding='utf-8')}"
    )
    return usage.ru_utime + usage.ru_stime, source.read_text(encoding="utf-8")


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
