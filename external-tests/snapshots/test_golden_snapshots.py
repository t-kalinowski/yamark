"""Run via `uv run external-tests/run.py --suite snapshots`."""

from __future__ import annotations

from textwrap import dedent

import pytest

from _support import format_and_check


def fixture(text: str) -> str:
    return dedent(text)


YAML_CASES = [
    (
        "flow-table",
        fixture(
            """\
            # fmt: table
            - {name: alpha,type: string,default: [one,two],description: "Short"}
            - {name: beta,type: int,default: 0,description: "Number"}
            """
        ),
        fixture(
            """\
            # fmt: table
            - {name: alpha, type: string, default: [one, two], description: Short}
            - {name: beta,  type: int,    default: 0,          description: Number}
            """
        ),
    ),
    (
        "folded-prose",
        fixture(
            """\
            title: "Formatter snapshots"
            description: "This is a long prose string that should be represented as a folded block scalar so snapshot diffs reveal broad formatter output changes."
            tags: [yaml,snapshots,regression-tests]
            """
        ),
        fixture(
            """\
            title: Formatter snapshots
            description: >-
              This is a long prose string that should be represented as a folded block
              scalar so snapshot diffs reveal broad formatter output changes.
            tags: [yaml, snapshots, regression-tests]
            """
        ),
    ),
]


MARKDOWN_CASES = [
    (
        "front-matter",
        fixture(
            """\
            ---
            title: "Snapshot page"
            tags: [rust,yaml,formatters]
            editor_options:
              markdown:
                wrap: sentence
            ---

            #   Snapshot Page

            First sentence. Second sentence that should move onto its own line when sentence wrapping comes from front matter.
            """
        ),
        fixture(
            """\
            ---
            title: Snapshot page
            tags: [rust, yaml, formatters]
            editor_options:
              markdown:
                wrap: sentence
            ---

            # Snapshot Page

            First sentence.
            Second sentence that should move onto its own line when sentence wrapping comes from front matter.
            """
        ),
    ),
    (
        "fenced-yaml",
        fixture(
            """\
            # Embedded YAML

            ```yaml
            items: [{name: alpha,type: string},{name: beta,type: int}]
            ```

            After.
            """
        ),
        fixture(
            """\
            # Embedded YAML

            ```yaml
            items: [{name: alpha, type: string}, {name: beta, type: int}]
            ```

            After.
            """
        ),
    ),
]


@pytest.mark.parametrize(
    "name,input_text,expected",
    YAML_CASES,
    ids=[case[0] for case in YAML_CASES],
)
def test_yaml_golden_snapshots_match_cli_output(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_and_check("yamark format {path}.yaml", input_text, expected)
    format_and_check("yamark format {path}.yaml", expected, expected)


@pytest.mark.parametrize(
    "name,input_text,expected",
    MARKDOWN_CASES,
    ids=[case[0] for case in MARKDOWN_CASES],
)
def test_markdown_golden_snapshots_match_cli_output(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_and_check("yamark format {path}.md", input_text, expected)
    format_and_check("yamark format {path}.md", expected, expected)
