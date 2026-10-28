"""Sentence escapes preserve Markdown meaning and column roundtrips."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest
from test_quarto_semantics import canonicalize_quarto_json


def format_markdown(source: str, wrap: str) -> str:
    result = subprocess.run(
        [
            os.environ["YAMARK_BIN"],
            "format",
            "--stdin-file-path",
            "input.md",
            "--wrap",
            wrap,
        ],
        input=source,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def markdown_ast(source: str, reader: str) -> object:
    result = subprocess.run(
        ["quarto", "pandoc", "--from", reader, "--to", "json"],
        input=source,
        capture_output=True,
        text=True,
        check=True,
    )
    return canonicalize_quarto_json(json.loads(result.stdout))


@pytest.mark.parametrize("reader", ["markdown", "commonmark", "gfm"])
@pytest.mark.parametrize("width", [8, 32, 72])
@pytest.mark.parametrize("sentence_column", [False, True])
def test_sentence_column_roundtrip_preserves_meaning(
    reader: str, width: int, sentence_column: bool
) -> None:
    paragraphs = []
    for marker in ["-", "+", "*", "(b)", "(ii)", "2.", "2)", "b.", "ii.", ">", "#"]:
        prose = f"Opening sentence. First sentence. {marker} More prose. Last sentence."
        paragraphs.extend([prose, f"- {prose}", f"> {prose}"])
    source = "\n\n".join(paragraphs) + "\n"
    column = format_markdown(source, str(width))
    wrap = f"sentence:{width}" if sentence_column else "sentence"
    sentence = format_markdown(column, wrap)
    assert format_markdown(sentence, wrap) == sentence
    restored = format_markdown(sentence, str(width))
    assert restored == column
    expected = markdown_ast(source, reader)
    for output in [column, sentence, restored]:
        assert markdown_ast(output, reader) == expected


@pytest.mark.parametrize("reader", ["markdown", "commonmark", "gfm"])
def test_meaning_sensitive_sentence_boundaries_stay_joined(reader: str) -> None:
    case = (
        Path(os.environ["YAMARK_ROOT"])
        / "tests/cases/markdown_sentence_meaning_sensitive.case"
    ).read_text()
    source = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    output = format_markdown(source, "sentence")
    assert markdown_ast(output, reader) == markdown_ast(source, reader)


@pytest.mark.parametrize("reader", ["markdown", "commonmark", "gfm"])
@pytest.mark.parametrize("width", [12, 72])
def test_column_wrapping_preserves_inline_html_escapes(reader: str, width: int) -> None:
    case = (
        Path(os.environ["YAMARK_ROOT"])
        / "tests/cases/markdown_column_inline_html_escapes.case"
    ).read_text()
    source = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    output = format_markdown(source, str(width))
    assert markdown_ast(output, reader) == markdown_ast(source, reader)


@pytest.mark.parametrize(
    "reader", ["markdown", "markdown+tex_math_single_backslash", "commonmark", "gfm"]
)
@pytest.mark.parametrize("wrap", ["sentence", "sentence:12", "sentence:72"])
def test_parenthesized_marker_escapes_preserve_math_meaning(
    reader: str, wrap: str
) -> None:
    case = (
        Path(os.environ["YAMARK_ROOT"])
        / "tests/cases/markdown_sentence_parenthesized_marker_escapes.case"
    ).read_text()
    source = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    column = format_markdown(source, "72")
    sentence = format_markdown(column, wrap)
    assert format_markdown(sentence, wrap) == sentence
    restored = format_markdown(sentence, "72")
    assert restored == column
    expected = markdown_ast(source, reader)
    for output in [column, sentence, restored]:
        assert markdown_ast(output, reader) == expected


@pytest.mark.parametrize(
    "reader", ["markdown", "markdown+tex_math_single_backslash", "commonmark", "gfm"]
)
@pytest.mark.parametrize(
    ("case_name", "wrap"),
    [
        ("markdown_container_raw_html_boundaries.case", "sentence"),
        ("markdown_container_raw_html_boundaries.case", "72"),
        ("markdown_container_raw_html_boundaries.case", "sentence:12"),
        ("markdown_container_literal_block_markers.case", "sentence"),
        ("markdown_container_literal_block_markers.case", "72"),
        ("markdown_container_literal_block_markers.case", "sentence:12"),
        ("markdown_column_preserved_html_literals.case", "72"),
        ("markdown_column_preserved_html_literals.case", "12"),
        ("markdown_column_preserved_html_literals.case", "sentence:12"),
        ("markdown_column_math_sensitive_emphasis.case", "24"),
        ("markdown_column_math_sensitive_emphasis.case", "12"),
        ("markdown_column_math_sensitive_emphasis.case", "sentence:12"),
        ("markdown_column_quoted_html_escapes.case", "72"),
        ("markdown_column_quoted_html_escapes.case", "12"),
        ("markdown_column_quoted_html_escapes.case", "sentence:12"),
        ("markdown_column_final_marker_escapes.case", "12"),
        ("markdown_column_final_marker_escapes.case", "sentence:12"),
        ("markdown_column_emphasis_math_escapes.case", "72"),
        ("markdown_column_emphasis_math_escapes.case", "12"),
        ("markdown_column_emphasis_math_escapes.case", "sentence:12"),
        ("markdown_column_hard_break_marker_escapes.case", "12"),
        ("markdown_column_hard_break_asterisk_escapes.case", "72"),
        ("markdown_column_trailing_asterisk_repairs.case", "23"),
        ("markdown_column_trailing_asterisk_repairs.case", "12"),
        ("markdown_column_trailing_asterisk_repairs.case", "sentence:12"),
        ("markdown_column_nested_headings.case", "72"),
        ("markdown_column_nested_headings.case", "sentence"),
        ("markdown_sentence_column_nested_headings.case", "sentence:12"),
        ("markdown_sentence_column_emphasis_repairs.case", "sentence:72"),
        ("markdown_column_emphasis_repairs.case", "12"),
        ("markdown_column_opaque_html_escapes.case", "72"),
        ("markdown_column_raw_tex_escapes.case", "72"),
        ("markdown_column_raw_tex_escapes.case", "sentence:12"),
        ("markdown_column_asterisk_escapes.case", "12"),
        ("markdown_sentence_asterisk_boundaries.case", "sentence"),
        ("markdown_sentence_column_math_contents.case", "sentence:12"),
        ("markdown_sentence_setext_candidates.case", "sentence"),
        ("markdown_sentence_setext_candidates.case", "sentence:12"),
        ("markdown_sentence_reference_boundaries.case", "sentence"),
        ("markdown_sentence_definition_markers.case", "sentence"),
        ("markdown_sentence_definition_markers.case", "sentence:12"),
        ("markdown_sentence_definition_markers.case", "12"),
    ],
)
def test_marker_repair_preserves_inline_and_block_meaning(
    reader: str, case_name: str, wrap: str
) -> None:
    case = (Path(os.environ["YAMARK_ROOT"]) / "tests/cases" / case_name).read_text()
    source = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    output = format_markdown(source, wrap)
    assert format_markdown(output, wrap) == output
    assert markdown_ast(output, reader) == markdown_ast(source, reader)
    column = format_markdown(source, "72")
    sentence = format_markdown(column, "sentence:12")
    assert format_markdown(sentence, "72") == column


@pytest.mark.parametrize("width", [8, 12, 20, 72])
@pytest.mark.parametrize("wrap", ["sentence", "sentence:12"])
def test_asterisk_boundaries_keep_column_roundtrips(width: int, wrap: str) -> None:
    source = "One sentence. * two words. * three words. Last sentence.\n"
    column = format_markdown(source, str(width))
    sentence = format_markdown(column, wrap)
    assert format_markdown(sentence, str(width)) == column
    expected = markdown_ast(source, "markdown")
    for output in [column, sentence]:
        assert markdown_ast(output, "markdown") == expected


@pytest.mark.parametrize(
    ("case_name", "wrap"),
    [
        ("markdown_sentence_shortcode_math_opener.case", "sentence:12"),
        ("markdown_column_final_marker_escapes.case", "12"),
    ],
)
def test_formatted_transcripts_are_stable_and_pass_check(
    tmp_path: Path,
    case_name: str,
    wrap: str,
) -> None:
    case = (Path(os.environ["YAMARK_ROOT"]) / "tests/cases" / case_name).read_text()
    source = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    expected = case.split("-- stdout\n")[1].split("-- stderr\n")[0]
    assert format_markdown(source, wrap) == expected
    assert format_markdown(expected, wrap) == expected
    path = tmp_path / "input.md"
    path.write_text(expected)
    result = subprocess.run(
        [
            os.environ["YAMARK_BIN"],
            "format",
            "--check",
            "--wrap",
            wrap,
            str(path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
