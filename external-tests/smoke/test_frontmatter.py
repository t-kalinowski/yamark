"""Run via `uv run external-tests/run.py`."""

from __future__ import annotations

from textwrap import dedent

from _support import format_and_check


def test_markdown_frontmatter_rewrap_smoke() -> None:
    """Smoke test of Markdown with YAML front matter and prose wrapping."""

    input_text = dedent(
        """\
        ---
        title: Demo
        tags: [docs,smoke]
        ---

        #   Release note

        This paragraph has enough words to demonstrate the command line prose wrapping behavior in a compact fixture.
        """
    )

    expected = dedent(
        """\
        ---
        title: Demo
        tags: [docs, smoke]
        ---

        # Release note

        This paragraph has enough words to
        demonstrate the command line prose
        wrapping behavior in a compact fixture.
        """
    )

    format_and_check("yamark format --wrap 40 {path}.md", input_text, expected)
