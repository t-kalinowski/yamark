"""Run via `uv run external-tests/run.py`."""

from __future__ import annotations

from textwrap import dedent

from _support import format_and_check


def test_basic_markdown_smoke() -> None:
    """Basic smoke test of column wrapping simple Markdown."""

    input_text = dedent(
        """\
        #   Title

        A long line that needs to be wrapped because it is over the default width and keeps going with a few more words.
        """
    )

    expected = dedent(
        """\
        # Title

        A long line that needs to be wrapped because it is over the default
        width and keeps going with a few more words.
        """
    )

    format_and_check("yamark format {path}.md", input_text, expected)
