"""Run via `uv run external-tests/run.py --suite cli/test_semantic_invariants.py`."""

from __future__ import annotations

from textwrap import dedent

import pytest

from _support import format_and_check
from _support import run_cli_case


def fixture(text: str) -> str:
    return dedent(text)


YAML_CASES = [
    (
        "flow-spacing",
        "items: [{a: b,c: d}, {a: e,c: f}]\n",
        "items: [{a: b, c: d}, {a: e, c: f}]\n",
    ),
    (
        "scalar-styles-and-tags",
        fixture(
            """\
            version: '1.10'
            scientific: "1e2"
            boolish: !!str true
            bool_value: true
            null_value: null
            """
        ),
        fixture(
            """\
            version: '1.10'
            scientific: "1e2"
            boolish: !!str 'true'
            bool_value: true
            null_value: null
            """
        ),
    ),
    (
        "anchors-aliases-and-tags",
        fixture(
            """\
            items: &items [a,b]
            tagged: !seq [c,d]
            both: !seq &both [e,f]
            ref: *items
            """
        ),
        fixture(
            """\
            items: &items [a, b]
            tagged: !seq [c,d]
            both: !seq &both [e,f]
            ref: *items
            """
        ),
    ),
    (
        "multiple-documents",
        fixture(
            """\
            --- !doc
            items: [a,b]
            --- !doc
            items: [c,d]
            """
        ),
        fixture(
            """\
            --- !doc
            items: [a, b]
            --- !doc
            items: [c, d]
            """
        ),
    ),
    (
        "formatter-directives",
        fixture(
            """\
            rows:
              # fmt: table
              - {name: a, type: int, default: 0}
              - {name: long_name, type: string, default: ""}
            # fmt: skip
            manual:
                -   [ 1,2,3]
            """
        ),
        fixture(
            """\
            rows:
              # fmt: table
              - {name: a,         type: int,    default: 0}
              - {name: long_name, type: string, default: ""}
            # fmt: skip
            manual:
                -   [ 1,2,3]
            """
        ),
    ),
]


MARKDOWN_CASES = [
    (
        "front-matter-fences-tables-lists-footnotes",
        fixture(
            """\
            ---
            title: Demo
            tags: [yaml,markdown]
            editor_options:
              markdown:
                canonical: true
                wrap: 48
            ---

            #   Title

            This paragraph links to [the dashboard](https://example.com/service) and keeps `inline code` stable with a footnote.[^note]

            -   first item with _emphasis_
            -   second item with **strong text**

            | Field | Value |
            |---|---:|
            | retry_count | 3 |
            | mode | single path |

            ```yaml
            items: [a,b]
            nested: {enabled: true, tags: [markdown,yaml]}
            ```

            ````markdown
            ##   Nested

            Nested prose has *emphasis* and a list.

            -   nested item

            ```yaml
            items: [nested,yaml]
            ```
            ````

            [^note]: Footnote text with [linked text](https://example.com/ref).
            """
        ),
        fixture(
            """\
            ---
            title: Demo
            tags: [yaml, markdown]
            editor_options:
              markdown:
                canonical: true
                wrap: 48
            ---

            # Title

            This paragraph links to
            [the dashboard](https://example.com/service) and
            keeps `inline code` stable with a
            footnote.[^note]

            - first item with *emphasis*
            - second item with **strong text**

            | Field       |       Value |
            | ----------- | ----------: |
            | retry_count |           3 |
            | mode        | single path |

            ```yaml
            items: [a, b]
            nested: {enabled: true, tags: [markdown, yaml]}
            ```

            ````markdown
            ## Nested

            Nested prose has *emphasis* and a list.

            - nested item

            ```yaml
            items: [nested, yaml]
            ```
            ````

            [^note]: Footnote text with
              [linked text](https://example.com/ref).
            """
        ),
    ),
]


def test_formats_multiple_yaml_and_markdown_paths_without_semantic_changes() -> None:
    run_cli_case(
        "yamark format config.yaml post.md",
        files={
            "config.yaml": "items: [a,b]\n",
            "post.md": "#   Title\n\nThis is *one*\n",
        },
        expected_files={
            "config.yaml": "items: [a, b]\n",
            "post.md": "# Title\n\nThis is *one*\n",
        },
        stdout_contains="2 files scanned",
    )


@pytest.mark.parametrize(
    "name,input_text,expected",
    YAML_CASES,
    ids=[case[0] for case in YAML_CASES],
)
def test_cli_preserves_yaml_values_for_replacement_cases(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_and_check("yamark format {path}.yaml", input_text, expected)


@pytest.mark.parametrize(
    "name,input_text,expected",
    MARKDOWN_CASES,
    ids=[case[0] for case in MARKDOWN_CASES],
)
def test_cli_preserves_markdown_ast_for_replacement_cases(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_and_check("yamark format {path}.md", input_text, expected)
