"""Link destinations preserve meaning and sentence/column roundtrips."""

from __future__ import annotations

import os
from pathlib import Path

import pytest
from multimark import markdown_to_commonmark
from test_sentence_wrap import format_markdown, markdown_ast


def link_document(source: str, reader: str) -> object:
    if reader == "markdown":
        return markdown_ast(source, reader)
    # Use cmark-gfm for CommonMark/GFM: Pandoc 3.10's alternate readers reject the
    # established split-link layout that Quarto Markdown and GitHub accept.
    extensions = (
        ["table", "strikethrough", "autolink", "tagfilter", "tasklist"]
        if reader == "gfm"
        else []
    )
    return markdown_to_commonmark(source, width=72, extensions=extensions)


@pytest.mark.parametrize("reader", ["markdown", "commonmark", "gfm"])
@pytest.mark.parametrize("width", [12, 32, 72])
def test_long_links_preserve_meaning_and_roundtrips(reader: str, width: int) -> None:
    paragraphs = []
    for name in ["empty", "angle"]:
        case = (
            Path(os.environ["YAMARK_ROOT"])
            / f"tests/cases/markdown_{name}_link_destinations.case"
        ).read_text()
        prose = case.split("-- stdin\n")[1].split("-- stdout\n")[0]
        paragraphs.extend(prose.strip().split("\n\n"))
    paragraphs.extend(
        [
            "Link [text](https://example.com).",
            "See ![alt](https://example.com/image.png), after.",
            "See [![alt](https://example.com/image.png)](https://example.com), after.",
            "See [![alt](https://example.com/a/very/long/image.png)](go), after.",
            "See [![a long alt](i)](go), after.",
            "See [text](https://example.com/path_(one)), after.",
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
    expected = link_document(source, reader)
    column = format_markdown(source, str(width))
    assert format_markdown(column, str(width)) == column
    assert link_document(column, reader) == expected
    for wrap in ["sentence", f"sentence:{width}"]:
        for input_text in [source, column]:
            sentence = format_markdown(input_text, wrap)
            assert format_markdown(sentence, wrap) == sentence
            restored = format_markdown(sentence, str(width))
            assert restored == column
            assert link_document(sentence, reader) == expected
