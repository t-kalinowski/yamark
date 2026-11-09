"""List boundaries and setext precedence preserve Pandoc structure."""

from __future__ import annotations

import pytest
from test_sentence_wrap import format_markdown, markdown_ast


@pytest.mark.parametrize("wrap", ["none", "72", "sentence"])
@pytest.mark.parametrize(
    "source",
    [
        "- item one\nlazy text\n\n  para in item\n",
        "- item two\nx  continued\n\n  para in item\n",
        "- item one\nlazy text\n\n  para in item\n- next\n",
        "- item one\nlazy text\n\n  para in item\n\nOutside first. Outside second.\n",
        "- Parent.\n  - Child.\n\n  Outer first. Outer second.\n- Next.\n",
        "- Item.\n ~~~text\n alpha    beta\n ~~~\n- Next.\n <pre>\n gamma    delta\n </pre>\nOutside first. Outside second.\n",
        "- Before fence.\n~~~text\nkeep  code\n~~~\nOutside first. Outside second.\n",
    ],
)
def test_lazy_list_continuation_preserves_item_paragraphs(
    source: str, wrap: str
) -> None:
    output = format_markdown(source, wrap)
    assert markdown_ast(output, "markdown") == markdown_ast(source, "markdown")
    assert format_markdown(output, wrap) == output


@pytest.mark.parametrize("wrap", ["none", "72", "sentence"])
@pytest.mark.parametrize(
    "source",
    [
        "* * *\n---\n",
        "* * *\n===\n",
        "- - -\n---\n",
        "_ _ _\n===\n",
        "* item\n---\n",
        "- item\n===\n",
        "* * *\n\nAfter break. Next sentence.\n",
    ],
)
def test_setext_underline_precedes_thematic_text(source: str, wrap: str) -> None:
    output = format_markdown(source, wrap)
    assert markdown_ast(output, "markdown") == markdown_ast(source, "markdown")
    assert format_markdown(output, wrap) == output
