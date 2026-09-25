"""Public CLI scaling controls for template rejection and lexical shortcodes.

Run via `uv run external-tests/run.py --suite corpus/test_scaling.py`.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

pytestmark = pytest.mark.skipif(
    not hasattr(os, "wait4"), reason="Child CPU accounting requires os.wait4"
)


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
        # The trailing expression now wraps as an opaque word after the
        # overwide bracket token. Templates inside labels remain unsupported.
        expected = (
            opening * depth + label + "]" * depth + "\n" + suffix.lstrip() + "\n"
            if suffix
            else text
        )
        assert formatted == expected
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


@pytest.mark.parametrize("opening", ["{", "*"])
def test_rejected_template_paragraphs_scale_with_input_size(
    tmp_path: Path, opening: str
) -> None:
    durations = []
    for count in [1000, 4000]:
        text = (
            "Before `{{ foo }}` "
            + " ".join(opening + "x" for _ in range(count))
            + " after.\n"
        )
        source = tmp_path / f"rejected-template-{count}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == text
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "rejected template paragraphs should scale with input size: "
        f"1000 openers used {small:.6f}s CPU, 4000 openers used {large:.6f}s CPU"
    )


def test_unpaired_shortcode_calls_scale_with_input_size(tmp_path: Path) -> None:
    durations = []
    for count in [2000, 8000]:
        tokens = "{{< meta title >}}\n" * count
        source = tmp_path / f"shortcodes-{count}.md"
        source.write_text(
            "{{% notice %}}\n" + tokens + "Following\nprose.\n{{% /notice %}}\n",
            encoding="utf-8",
        )
        cpu, formatted = measure_formatting_cpu(source)
        assert formatted == (
            "{{% notice %}}\n" + tokens + "Following prose.\n{{% /notice %}}\n"
        )
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "independent shortcode calls should scale with input size: "
        f"2000 calls used {small:.6f}s CPU, 8000 calls used {large:.6f}s CPU"
    )


@pytest.mark.parametrize("token", ["{{ keep   this END", "{#identifier}"])
@pytest.mark.parametrize("suffix", ["", "\n{# next line #}"])
def test_overlapping_delimiters_and_attributes_scale_with_input_size(
    tmp_path: Path, token: str, suffix: str
) -> None:
    durations = []
    for count in [2000, 8000]:
        text = (
            '<!-- fmt: template.delimiters "{{" "END" scope=file -->\n'
            '<!-- fmt: wrap=paragraph scope=file -->\n'
            + (token + " ") * count
            + "{{ last }}"
            + suffix
            + "\n"
        )
        source = tmp_path / f"template-boundaries-{count}.md"
        source.write_text(text, encoding="utf-8")
        cpu, formatted = measure_formatting_cpu(source)
        # Attribute openers remain ambiguous to the existing template guard
        # when a comment closer occurs on a later physical line.
        expected = (
            text
            if token == "{#identifier}"
            else text.replace("\n{# next line #}", " {# next line #}")
        )
        assert formatted == expected
        durations.append(cpu)

    small, large = durations
    assert small > 0, "formatter CPU time must be available"
    assert large <= small * 6, (
        "template boundaries should not repeatedly scan the remaining suffix: "
        f"2000 tokens used {small:.6f}s CPU, 8000 used {large:.6f}s CPU"
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
