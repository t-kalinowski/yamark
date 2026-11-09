"""Heading rewrites and paragraph boundaries retain Pandoc's rendered structure."""

from __future__ import annotations

import pytest
from test_sentence_wrap import format_markdown, markdown_ast


@pytest.mark.parametrize("wrap", ["none", "20", "72", "sentence"])
@pytest.mark.parametrize(
    "source",
    [
        "# Title\n***\n",
        "Title\n===\n***\n",
        "Title\n===\n---\n",
        "# Title\n---\n",
        "## Title\n---\n",
        "#\n---\n",
        "# Title ##\n---\n",
        "# Title\n===\n",
        "Text\n***\n",
        "Text\n* * *\n",
        "Text\n- - -\n",
        "Text\n_ _ _\n",
        "Text\nmore\n---\n",
        "Text\n  more\n---\n",
        "Text\nTitle\n===\n",
        "Text\n[^a]: text\n---\n",
        "Text\n* a\n",
        "Text\n+ a\n",
        "Text\n1. a\n",
        "Text\n| a | b |\n|---|---|\n| c | d |\n",
        "Learning C#\n---\n",
        "A# \n---\n",
        "Hello #\n---\n",
        "Title # {#id}\n---\n",
        "Even \\\\#\n---\n",
        "Odd \\#\n---\n",
        "Many ###\n---\n",
        'Attribute # {title="a b"}\n---\n',
        " One space\n===\n",
        "   Three spaces\n---\n",
    ],
)
def test_heading_and_paragraph_rewrites_preserve_pandoc(source: str, wrap: str) -> None:
    output = format_markdown(source, wrap)
    assert markdown_ast(output, "markdown") == markdown_ast(source, "markdown")
    assert format_markdown(output, wrap) == output
