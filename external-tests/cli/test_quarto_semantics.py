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
PANDOC_TABLE_CASES = [
    (
        "simple-default-header-wider-body",
        "Name    Value\n----    -----\nlonger  example",
    ),
    (
        "pipe-table-unequal-widths",
        "| A        | B |\n| -------- | - |\n| A long cell whose contents make the source exceed seventy two characters in total | y |",
    ),
    (
        "pipe-table-padding-width-threshold",
        "| A                                   | B                              |\n| ----------------------------------- | ------------------------------ |\n| x                                   | y                              |",
    ),
    (
        "grid-table-explicit-alignments",
        "+----------+------------+\n| Name     | Value      |\n+=========:+:==========:+\n| x        | y          |\n+----------+------------+",
    ),
    (
        "simple-table-alignment",
        "A             B\n-----  ------------\nx      y",
    ),
    (
        "grid-table-column-width",
        (
            "+------+------+\n"
            "| A    | B    |\n"
            "+======+======+\n"
            "| x    | y    |\n"
            "+------+------+"
        ),
    ),
    (
        "multiline-table-column-width",
        (
            "Before.\n"
            "\n"
            "----------  ----------\n"
            "A           B\n"
            "----------  ----------\n"
            "x           y\n"
            "----------  ----------"
        ),
    ),
    (
        "simple-table-alignments",
        (
            "  Right     Left     Center     Default\n"
            "-------     ------ ----------   -------\n"
            "     12     12        12            12\n"
            "    123     123       123          123"
        ),
    ),
    (
        "simple-table-header-spacing",
        "Two  words  Other\n----------  -----\nx           y",
    ),
    (
        "simple-table-headerless",
        (
            "-------     ------ ----------   -------\n"
            "     12     12        12            12\n"
            "    123     123       123          123\n"
            "-------     ------ ----------   -------"
        ),
    ),
    (
        "simple-table-unicode",
        "Name        Value\n----------  -----\ncafé        one\n漢字        two",
    ),
    (
        "simple-table-emoji-header",
        "👩‍💻  Bée  End\n----  ---  ---\none   two  end",
    ),
    ("simple-table-tabs", "A\tB\n---\t---\nx\ty"),
    ("simple-table-tabs-spaced-border", "A\tB\n--- ---\nx\ty"),
    ("simple-table-consecutive-tabs", "A\t\tB\n---\t\t---\nx\t\ty"),
    ("simple-table-tab-after-wide-characters", "漢字\tB\n---\t----\n甲\t乙"),
    ("simple-table-tab-after-combining-mark", "e\u0301\tB\n---\t---\nx\ty"),
    ("simple-table-headerless-tabs", "---\t---\na\tb\n---\t---"),
    (
        "multiline-table-tabs",
        "------------\nA\tB\n---\t---\nx\ty\n\na\tb\n------------",
    ),
    (
        "grid-table-tabs",
        (
            "+-------+-------+\n"
            "| A\t\t| B\t\t|\n"
            "+=======+=======+\n"
            "| x\t\t| y\t\t|\n"
            "+-------+-------+"
        ),
    ),
    (
        "simple-table-combining-marks-right",
        (
            "A           B\n"
            "-----  ------\n"
            "cafe\u0301   y\n"
            "abcdefg\u0301x\n"
            "abcdefg\u0301\u0308x"
        ),
    ),
    (
        "simple-table-combining-marks-center",
        (
            "A        B\n"
            "-----  ------\n"
            "cafe\u0301   y\n"
            "abcdefg\u0301x\n"
            "abcdefg\u0301\u0308x"
        ),
    ),
    (
        "grid-table-unequal-widths",
        (
            "+----------+------------------+\n"
            "| A        | B                |\n"
            "+==========+==================+\n"
            "| x  one   | y  two           |\n"
            "+----------+------------------+"
        ),
    ),
    (
        "grid-table-headerless",
        (
            "+----------+------------------+\n"
            "| x        | y                |\n"
            "+----------+------------------+"
        ),
    ),
    (
        "multiline-table-alignments",
        (
            "-------------------------------------------------------------\n"
            "Centered   Default           Right Left\n"
            " Header    Aligned         Aligned Aligned\n"
            "---------  -------  -------------- -------------------------\n"
            "First      row                12.0 Example of a row that\n"
            "                                   spans multiple lines.\n"
            "\n"
            "Second     row                 5.0 Another row.\n"
            "-------------------------------------------------------------"
        ),
    ),
    (
        "multiline-table-headerless",
        (
            "----------  --------------------\n"
            "x           y\n"
            "            z\n"
            "\n"
            "a           b\n"
            "----------  --------------------"
        ),
    ),
    (
        "multiline-table-caption-like-body",
        (
            "------  ------\n"
            "a       b\n"
            "d       e\n"
            "------  ------\n"
            "Table: keep    cell spacing\n"
            "\n"
            "next    row\n"
            "------  ------"
        ),
    ),
    (
        "multiline-table-unequal-gaps",
        (
            "-----------------------------------\n"
            " A      B          C\n"
            "----- -----   ---------------------\n"
            "x     y       z\n"
            "\n"
            "one   two     three\n"
            "-----------------------------------"
        ),
    ),
    (
        "simple-table-canonical-header",
        "_Two_  words  Other\n------------  -----\n_x_           __y__",
    ),
    (
        "grid-table-narrow-cells",
        "+----+-----+\n|Name|Value|\n+====+=====+\n|_x_ |__y__|\n+----+-----+",
    ),
    (
        "multiline-table-canonical-header",
        (
            "--------------------------------\n"
            " _Two_          Other\n"
            "------------    ----------------\n"
            "_x_             __y__\n"
            "\n"
            "a               b\n"
            "--------------------------------"
        ),
    ),
]


def markdown_cases() -> list[tuple[str, str]]:
    cases = list(PANDOC_TABLE_CASES)

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
                (
                    "See [documentation](https://example.com/really/really/"
                    f"really/long/path {title}) for details after the link."
                ),
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
                    (
                        f"Text.[^tex-{command_index}]\n\n"
                        f"[^tex-{command_index}]: before \\{command}\n  after words"
                    ),
                ),
            ]
        )
    cases.append(
        (
            "protected-raw-looking-inline",
            (
                "This paragraph includes `C:\\Users\u00a0name`, "
                "[link \\alpha](dest), $\\beta + x$, and "
                "<span>HTML\u00a0text</span> while enough words remain to wrap safely."
            ),
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
                (
                    "Text with a note.[^fuzz-note]\n\n"
                    "[^fuzz-note]: Footnote words that can wrap safely."
                ),
            ),
            (
                "blockquote",
                "> Quoted paragraph with several words that can wrap safely.",
            ),
            (
                "list",
                (
                    "- First list item with several words that can wrap safely.\n"
                    "- Second item with *emphasis*."
                ),
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
        (
            '---\ntitle: "Yamark semantics"\nparams:\n  label: "yes"\n'
            "  values: [1, 2, 3]\n---\n\n"
        )
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
        capture_output=True,
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


def without_table_widths(value: object) -> object:
    """Fit mode may change column widths; all other Pandoc fields must match."""
    if isinstance(value, dict):
        result = {key: without_table_widths(item) for key, item in value.items()}
        if result.get("t") == "Table":
            result["c"][2] = [
                [alignment, {"t": "ColWidthDefault"}] for alignment, _ in result["c"][2]
            ]
        return result
    if isinstance(value, list):
        return [without_table_widths(item) for item in value]
    return value


def case_blocks(document: object, table_widths: str) -> dict[str, object]:
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
            block = canonicalize_quarto_json(block)
            cases[case_id].append(
                without_table_widths(block) if table_widths == "fit" else block
            )
    return cases


@pytest.mark.parametrize("body", ["a  b    c\n", "a  b    c\n        d\n\ne       f\n"])
@pytest.mark.parametrize("table_widths", ["fit", "preserve"])
@pytest.mark.parametrize(
    "following",
    [
        "",
        "Table: keep    caption spacing\n\nFirst sentence. Second sentence.\n",
        "#   Following heading\n\nFirst sentence. Second sentence.\n",
        *[
            following + "\n\nLater paragraph.\n\n" + later
            for following in ["Table: caption", "# Heading"]
            for later in [
                "C       D\n------  ------\nu       v\n",
                "------  ------\nC       D\n------  ------\nu       v\n------  ------\n",
            ]
        ],
    ],
)
def test_headerless_table_preserves_quarto_document(
    body: str, following: str, table_widths: str
) -> None:
    source = f"Before.\n\n------  ------\n{body}------  ------\n{following}"
    with TemporaryDirectory(prefix="yamark-quarto-headerless-") as temp:
        root = Path(temp)
        before = render_quarto_json(root, "before", source)
        result = subprocess.run(
            [
                os.environ["YAMARK_BIN"],
                "format",
                "--wrap",
                "sentence",
                "--table-widths",
                table_widths,
                "--stdin-file-path",
                "input.qmd",
            ],
            input=source,
            capture_output=True,
            check=False,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        after = render_quarto_json(root, "after", result.stdout)
    before, after = canonicalize_quarto_json(before), canonicalize_quarto_json(after)
    if table_widths == "fit":
        before, after = without_table_widths(before), without_table_widths(after)
    assert before == after


def test_multiline_shortcode_preserves_quarto_document() -> None:
    source = (
        '---\ntitle: "Multiline shortcode control"\n---\n\n'
        "Before {{< meta\n title >}} after.\n"
    )
    with TemporaryDirectory(prefix="yamark-quarto-shortcode-") as temp:
        root = Path(temp)
        before = render_quarto_json(root, "before", source)
        result = subprocess.run(
            [
                os.environ["YAMARK_BIN"],
                "format",
                "--wrap",
                "20",
                "--stdin-file-path",
                "input.qmd",
            ],
            input=source,
            capture_output=True,
            check=False,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        assert "{{< meta\n title >}}" in result.stdout
        after = render_quarto_json(root, "after", result.stdout)
    assert "{{<" not in json.dumps(before["blocks"])
    assert canonicalize_quarto_json(before) == canonicalize_quarto_json(after)


@pytest.mark.parametrize("width", WIDTHS)
@pytest.mark.parametrize("canonical", [False, True])
@pytest.mark.parametrize("table_widths", ["fit", "preserve"])
def test_formatting_preserves_quarto_document(
    width: int, canonical: bool, table_widths: str
) -> None:
    assert shutil.which("quarto") is not None, "quarto is required"
    version = subprocess.run(
        ["quarto", "--version"],
        capture_output=True,
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
        before = case_blocks(before_document, table_widths)
        formatted_path = root / "formatted.qmd"
        formatted_path.write_text(before_text, encoding="utf-8")
        result = subprocess.run(
            [
                yamark_bin,
                "format",
                "--wrap",
                str(width),
                "--table-widths",
                table_widths,
                *(["--canonical"] if canonical else []),
                formatted_path.name,
            ],
            cwd=root,
            capture_output=True,
            check=False,
            text=True,
        )
        assert result.returncode == 0, result.stderr
        after_document = render_quarto_json(
            root,
            "after",
            formatted_path.read_text(encoding="utf-8"),
        )
        after = case_blocks(after_document, table_widths)

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
