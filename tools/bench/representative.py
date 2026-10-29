"""Deterministic YAML workloads with varied scalar and Markdown structure."""

from __future__ import annotations

import json
import random


WORKLOADS = (
    "ordinary-config",
    "flow-heavy",
    "block-heavy",
    "plain-prose",
    "quoted-prose",
    "literal-scalars",
    "markdown-paragraphs",
    "markdown-rich",
)
WORDS = (
    "lorem ipsum dolor sit amet consectetur adipiscing elit sed eiusmod tempor "
    "incididunt labore dolore magna aliqua enim minim veniam quis nostrud "
    "exercitation ullamco laboris nisi aliquip commodo consequat"
).split()


def prose(seed: int, *, punctuation: bool = False) -> str:
    words = random.Random(seed).choices(WORDS, k=27 + seed % 19)
    if punctuation:
        for index in range(7, len(words), 11):
            words[index] += "."
    return " ".join(words)


def render_file(file_idx: int, items: int, workload: str) -> str:
    renderers = {
        "ordinary-config": ordinary_config,
        "plain-prose": plain_prose,
        "quoted-prose": quoted_prose,
        "literal-scalars": literal_scalars,
        "markdown-paragraphs": markdown_paragraphs,
        "markdown-rich": markdown_rich,
    }
    text = f"name:    lorem-{file_idx:04d}\n" + renderers[workload](file_idx, items)
    return text.replace("\n", "\r\n") if workload == "literal-scalars" else text


def ordinary_config(file_idx: int, items: int) -> str:
    lines = [
        "# lorem ipsum",
        "defaults: &lorem",
        "  enabled: true",
        "  retries: 3",
        "  timeout: 30s",
        "lorem:",
    ]
    for index in range(items):
        lines.extend([
            f"  ipsum_{index:03d}:",
            "    <<: *lorem",
            f"    count: {(file_idx + index) % 13}",
            "    optional: null",
            "    flags: [true,false]",
            "    paths: [lorem,ipsum,dolor]",
            f"    label: '{WORDS[(file_idx + index) % len(WORDS)]}' # sit amet",
        ])
    return "\n".join(lines) + "\n"


def plain_prose(file_idx: int, items: int) -> str:
    lines = ["lorem:"]
    for index in range(items):
        words = prose(file_idx * 1000 + index).split()
        split = 9 + index % 13
        lines.extend(["  - " + " ".join(words[:split]), "    " + " ".join(words[split:])])
    return "\n".join(lines) + "\n"


def quoted_prose(file_idx: int, items: int) -> str:
    lines = ["lorem:"]
    for index in range(items):
        text = "lórém ip'sum \"dolor\" \\ sit " + prose(file_idx * 1000 + index)
        quote = "'" if index % 2 == 0 else '"'
        raw = text.replace("'", "''") if quote == "'" else json.dumps(text, ensure_ascii=False)[1:-1]
        words = raw.split()
        split = 11 + index % 7
        lines.extend([
            "  - " + quote + " ".join(words[:split]),
            "    " + " ".join(words[split:]) + quote,
        ])
    return "\n".join(lines) + "\n"


def literal_scalars(file_idx: int, items: int) -> str:
    lines = ["lorem:"]
    for index in range(items):
        lines.extend([
            "  - |-",
            "    " + prose(file_idx * 1000 + index),
            "",
            "    " + prose(file_idx * 1000 + index + 1),
        ])
    return "\n".join(lines) + "\n"


def markdown_paragraphs(file_idx: int, items: int) -> str:
    lines = ["lorem:"]
    used = 0
    group = 0
    while used < items:
        count = min((1, 2, 8)[group % 3], items - used)
        lines.append("  - !md |")
        for paragraph in range(count):
            if paragraph:
                lines.append("")
            seed = file_idx * 1000 + used + paragraph
            lines.append("    " + prose(seed, punctuation=group % 2 == 0))
        used += count
        group += 1
    return "\n".join(lines) + "\n"


def markdown_rich(file_idx: int, items: int) -> str:
    lines = ["lorem:"]
    # A rich scalar has roughly eight blocks; keep its total work comparable
    # to the paragraph budget instead of multiplying that budget by eight.
    for index in range(max(1, (items + 7) // 8)):
        seed = file_idx * 1000 + index
        body = [
            "# Lorem ipsum",
            "",
            prose(seed),
            "",
            "- " + prose(seed + 1),
            "- lorem ipsum `dolor sit`",
            "",
            "i. lorem ipsum",
            "iv. dolor sit amet",
            "",
            "```yaml",
            "lorem: [ipsum,dolor]",
            "```",
            "",
            "[Lorem ipsum](https://example.invalid/lorem) " + prose(seed + 2),
            "",
            "Lorem {{ ipsum }} dolor sit amet.",
            "",
            "> " + prose(seed + 3, punctuation=True),
        ]
        lines.append("  - !md |")
        lines.extend("    " + line if line else "" for line in body)
    return "\n".join(lines) + "\n"
