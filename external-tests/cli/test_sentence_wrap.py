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
def test_parenthesized_marker_escapes_preserve_math_meaning(reader: str, wrap: str) -> None:
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
