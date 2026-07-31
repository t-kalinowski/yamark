"""Deterministic Markdown formatting checks against Quarto's Pandoc."""

from __future__ import annotations

import json
import os
import random
import shutil
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

import pytest


SEED = 20260731
WIDTHS = [20, 40, 72]
QUARTO_VERSION = "1.10.18"
KNOWN_PANDOC_TABLE_DIFFERENCES = [
    (
        "simple-table-alignment",
        "A             B\n-----  ------------\nx      y",
    ),
    (
        "grid-table-column-width",
        "+------+------+\n"
        "| A    | B    |\n"
        "+======+======+\n"
        "| x    | y    |\n"
        "+------+------+",
    ),
    (
        "multiline-table-column-width",
        "Before.\n\n"
        "----------  ----------\n"
        "A           B\n"
        "----------  ----------\n"
        "x           y\n"
        "----------  ----------",
    ),
]


def markdown_cases() -> list[tuple[str, str]]:
    cases: list[tuple[str, str]] = []

    unicode_whitespace = {
        "nel": "\u0085",
        "nbsp": "\u00a0",
        "ogham": "\u1680",
        "en-quad": "\u2000",
        "em-space": "\u2003",
        "figure-space": "\u2007",
        "thin-space": "\u2009",
        "narrow-nbsp": "\u202f",
        "medium-math-space": "\u205f",
        "ideographic-space": "\u3000",
    }
    for name, whitespace in unicode_whitespace.items():
        cases.extend(
            [
                (f"unicode-{name}", f"alpha{whitespace}beta gamma"),
                (f"unicode-leading-{name}", f"{whitespace}alpha beta"),
                (f"unicode-trailing-{name}", f"alpha beta{whitespace}"),
                (f"unicode-heading-{name}", f"# alpha{whitespace}"),
                (f"unicode-list-{name}", f"- {whitespace}alpha beta"),
                (f"unicode-blockquote-{name}", f"> {whitespace}alpha beta"),
            ]
        )

    for tag in [
        "span",
        "abbr",
        "kbd",
        "mark",
        "small",
        "time",
        "var",
        "samp",
        "b",
        "i",
        "em",
        "strong",
        "code",
        "sub",
        "sup",
    ]:
        cases.append((f"inline-html-{tag}", f"<{tag}>raw</{tag}> text after"))

    for owner in [
        "[text]",
        "`code`",
        "[link](dest)",
        "![alt](img)",
        "[@cite]",
    ]:
        cases.append(("heading-inline-attribute", f"# {owner}{{.class}}"))

    for title in ['"Title"', "'Title'", "(Title)"]:
        cases.append(
            (
                "titled-link",
                "See [documentation](https://example.com/really/really/"
                f"really/long/path {title}) for details after the link.",
            )
        )

    for command_index, command in enumerate(["LaTeX", "TeX", "alpha", "textbf{bold}"]):
        cases.extend(
            [
                ("raw-tex-boundary", f"before \\{command}\nafter words"),
                (
                    "raw-tex-wrap-boundary",
                    f"before words \\{command} after several following words",
                ),
                ("raw-tex-list", f"- before \\{command}\n  after words"),
                ("raw-tex-blockquote", f"> before \\{command}\n> after words"),
                (
                    "raw-tex-footnote",
                    f"Text.[^tex-{command_index}]\n\n"
                    f"[^tex-{command_index}]: before \\{command}\n  after words",
                ),
            ]
        )
    cases.append(
        (
            "protected-raw-looking-inline",
            "This paragraph includes `C:\\Users\u00a0name`, "
            "[link \\alpha](dest), $\\beta + x$, and "
            "<span>HTML\u00a0text</span> while enough words remain to wrap safely.",
        )
    )

    for prefix in ["\t", " \t", "  \t", "   \t", "    "]:
        cases.append(("root-indented-code", f"{prefix}code"))
    cases.append(("root-indented-setext-code", "\tTitle\n\t===="))
    for marker in ["---", "***", "___"]:
        cases.append(("root-indented-thematic-code", f"\t{marker}"))
    for prefix in ["\t", " \t", "  \t"]:
        cases.append(("root-indented-blockquote-code", f"{prefix}> code"))
        cases.append(("root-indented-list-code", f"{prefix}- item"))
    for spaces in [3, 4, 5, 6]:
        cases.append(("blockquote-indented-code", f">{' ' * spaces}code"))
    for prefix in ["> >     ", ">>     ", ">   \t"]:
        cases.append(("nested-blockquote-indented-code", f"{prefix}code"))

    cases.extend(
        [
            ("pipe-table", "| A | B |\n| :-- | --: |\n| x | y |"),
            (
                "definition-list",
                "Term\n: definition with several words that can wrap",
            ),
            (
                "definition-list-multiple-paragraphs",
                "Term\n: definition one\n\n  second paragraph",
            ),
            (
                "footnote",
                "Text with a note.[^fuzz-note]\n\n"
                "[^fuzz-note]: Footnote words that can wrap safely.",
            ),
            (
                "blockquote",
                "> Quoted paragraph with several words that can wrap safely.",
            ),
            (
                "list",
                "- First list item with several words that can wrap safely.\n"
                "- Second item with *emphasis*.",
            ),
        ]
    )

    randomizer = random.Random(SEED)
    atoms = [
        "plain",
        "*emphasis*",
        "**strong**",
        "`code span`",
        "$x + y$",
        "[link](https://example.com/path)",
        "<span>html</span>",
        "[@cite]",
    ]
    for _ in range(200):
        selected = [randomizer.choice(atoms) for _ in range(randomizer.randint(3, 12))]
        text = selected[0]
        for atom in selected[1:]:
            text += randomizer.choice([" ", "  ", "\n"]) + atom
        cases.append(("random-paragraph", text))

    return cases


def markdown_document(cases: list[tuple[str, str]]) -> str:
    sections = [
        '---\ntitle: "Yamark semantics"\nparams:\n  label: "yes"\n'
        "  values: [1, 2, 3]\n---\n\n"
    ]
    for index, (_, text) in enumerate(cases):
        text = text.rstrip("\r\n")
        sections.append(f"## Case {index:04d} {{#case-{index:04d}}}\n\n{text}\n\n")
    return "".join(sections)


def render_quarto_json(root: Path, stem: str, source: str) -> object:
    input_path = root / f"{stem}.qmd"
    output_name = f"{stem}.json"
    input_path.write_text(source, encoding="utf-8")
    result = subprocess.run(
        [
            "quarto",
            "render",
            input_path.name,
            "--to",
            "json",
            "--no-execute",
            "--output",
            output_name,
        ],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert result.returncode == 0, (
        f"quarto render failed with exit code {result.returncode}\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    return json.loads((root / output_name).read_text(encoding="utf-8"))


def canonicalize_quarto_json(value: object) -> object:
    if isinstance(value, dict):
        if value.get("t") == "SoftBreak":
            return {"t": "Space"}
        return {key: canonicalize_quarto_json(item) for key, item in value.items()}
    if isinstance(value, list):
        return [canonicalize_quarto_json(item) for item in value]
    return value


def canonicalize_known_pandoc_table_layout(value: object) -> object:
    if isinstance(value, dict):
        normalized = {
            key: canonicalize_known_pandoc_table_layout(item)
            for key, item in value.items()
        }
        if normalized.get("t") == "Table":
            contents = normalized["c"]
            assert isinstance(contents, list) and len(contents) == 6
            columns = contents[2]
            assert isinstance(columns, list)
            for column in columns:
                assert isinstance(column, list) and len(column) == 2
            contents[2] = [["known-alignment", "known-width"] for _ in columns]
        return normalized
    if isinstance(value, list):
        return [canonicalize_known_pandoc_table_layout(item) for item in value]
    return value


def case_blocks(document: object) -> dict[str, object]:
    assert isinstance(document, dict)
    blocks = document["blocks"]
    assert isinstance(blocks, list)
    cases: dict[str, list[object]] = {}
    case_id: str | None = None
    for block in blocks:
        assert isinstance(block, dict)
        if block.get("t") == "Header":
            candidate = block["c"][1][0]
            if candidate.startswith("case-"):
                case_id = candidate
                cases[case_id] = []
                continue
        if case_id is not None:
            cases[case_id].append(canonicalize_quarto_json(block))
    return cases


@pytest.mark.parametrize("width", WIDTHS)
def test_formatting_preserves_quarto_document(width: int) -> None:
    assert shutil.which("quarto") is not None, "quarto is required"
    version = subprocess.run(
        ["quarto", "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert version.returncode == 0, version.stderr
    assert version.stdout.strip() == QUARTO_VERSION
    yamark_bin = os.environ.get("YAMARK_BIN")
    assert yamark_bin is not None, "YAMARK_BIN is not set"

    cases = markdown_cases()
    before_text = markdown_document(cases)
    with TemporaryDirectory(prefix="yamark-quarto-fuzz-") as temp:
        root = Path(temp)
        before_document = render_quarto_json(root, "before", before_text)
        before = case_blocks(before_document)
        formatted_path = root / "formatted.qmd"
        formatted_path.write_text(before_text, encoding="utf-8")
        result = subprocess.run(
            [yamark_bin, "format", "--wrap", str(width), formatted_path.name],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        after_document = render_quarto_json(
            root,
            "after",
            formatted_path.read_text(encoding="utf-8"),
        )
        after = case_blocks(after_document)

    assert isinstance(before_document, dict)
    assert isinstance(after_document, dict)
    assert canonicalize_quarto_json(
        before_document["meta"]
    ) == canonicalize_quarto_json(after_document["meta"])
    expected_case_ids = {f"case-{index:04d}" for index in range(len(cases))}
    assert set(before) == expected_case_ids
    assert set(after) == expected_case_ids

    failures = []
    for index, (family, source) in enumerate(cases):
        case_id = f"case-{index:04d}"
        if before.get(case_id) != after.get(case_id):
            failures.append(
                f"{case_id} ({family})\n"
                f"source:\n{source}\n"
                f"before: {before.get(case_id)!r}\n"
                f"after:  {after.get(case_id)!r}"
            )
    assert not failures, (
        f"seed {SEED}, width {width}: formatting changed Quarto output\n\n"
        + "\n\n".join(failures[:10])
    )


@pytest.mark.parametrize(("family", "source"), KNOWN_PANDOC_TABLE_DIFFERENCES)
def test_known_pandoc_table_difference(family: str, source: str) -> None:
    version = subprocess.run(
        ["quarto", "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert version.returncode == 0, version.stderr
    assert version.stdout.strip() == QUARTO_VERSION
    yamark_bin = os.environ.get("YAMARK_BIN")
    assert yamark_bin is not None, "YAMARK_BIN is not set"
    document = markdown_document([(family, source)])
    with TemporaryDirectory(prefix="yamark-quarto-table-") as temp:
        root = Path(temp)
        before_document = render_quarto_json(root, "before", document)
        before = case_blocks(before_document)
        formatted_path = root / "formatted.qmd"
        formatted_path.write_text(document, encoding="utf-8")
        result = subprocess.run(
            [yamark_bin, "format", "--wrap", "72", formatted_path.name],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        after_document = render_quarto_json(
            root,
            "after",
            formatted_path.read_text(encoding="utf-8"),
        )
        after = case_blocks(after_document)
    assert isinstance(before_document, dict)
    assert isinstance(after_document, dict)
    assert canonicalize_quarto_json(
        before_document["meta"]
    ) == canonicalize_quarto_json(after_document["meta"])
    if before == after:
        pytest.fail(f"known {family} difference is fixed; move it into the main corpus")
    assert canonicalize_known_pandoc_table_layout(
        before
    ) == canonicalize_known_pandoc_table_layout(after)
    pytest.xfail("Yamark currently changes Pandoc table alignment or column width")
