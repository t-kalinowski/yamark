"""Long-link layouts preserve rendering and sentence/column roundtrips."""

from __future__ import annotations

import os
from pathlib import Path

import pytest
from test_sentence_wrap import format_markdown, markdown_ast


@pytest.mark.parametrize("reader", ["markdown", "commonmark", "gfm"])
@pytest.mark.parametrize("width", [12, 32, 72])
def test_long_links_preserve_meaning_and_roundtrips(reader: str, width: int) -> None:
    case = (
        Path(os.environ["YAMARK_ROOT"])
        / "tests/cases/markdown_long_link_boundaries.case"
    ).read_text()
    prose = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
    paragraphs = prose.strip().split("\n\n")
    paragraphs.extend(
        [
            "Link [text](https://example.com).",
            "See [text](https://example.com 'Title') after.",
            "See [text](https://example.com (Title)) after.",
            'See ![alt](https://example.com/image.png "Title") after.',
            "See ![alt](<https://example.com/image.png>) after.",
            "See [![alt](<https://example.com/image.png>)](https://example.com) after.",
            r"See [text](https://example.com/path\(one\)) after.",
            "See `[text](https://example.com)` after.",
            "See <https://example.com> after.",
            "See [text][ref] after.",
        ]
    )
    source = "\n\n".join(
        f"{prefix}{paragraph}"
        for paragraph in paragraphs
        for prefix in ["", "- ", "> "]
    )
    source += "\n\n[ref]: https://example.com\n"
    expected = markdown_ast(source, reader)
    column = format_markdown(source, str(width))
    assert format_markdown(column, str(width)) == column
    assert markdown_ast(column, reader) == expected
    for wrap in ["sentence", f"sentence:{width}"]:
        for input_text in [source, column]:
            sentence = format_markdown(input_text, wrap)
            assert format_markdown(sentence, wrap) == sentence
            restored = format_markdown(sentence, str(width))
            assert restored == column
            assert markdown_ast(sentence, reader) == expected
