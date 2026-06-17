"""Run via `uv run external-tests/run.py --suite corpus`."""

from __future__ import annotations

from textwrap import dedent

import pytest

from _support import run_cli_case


def fixture(text: str) -> str:
    return dedent(text)


def format_exact(command: str, filename: str, input_text: str, expected: str) -> None:
    run_cli_case(
        command,
        files={filename: input_text},
        expected_files={filename: expected},
        stderr="",
    )
    run_cli_case(
        command,
        files={filename: expected},
        expected_files={filename: expected},
        stderr="",
    )


YAML_CASES = [
    (
        "directives-tags-comments",
        fixture(
            """\
            %TAG !e! tag:example.com,2026:
            ---
            # fmt: table
            rows:
              - { name: alpha,tags: [docs,markdown], enabled: true } # first row
              - { name: beta-long,tags: [yaml,fmt], enabled: false } # second row
            anchors:
              base: &base { retries: 3, mode: "safe" }
              copy: *base
            tagged: !e!widget { id:1, name:widget, notes: "a # b" }
            markdown: !markdown |
              #   Heading

              -   item one
              -   item two
            ...
            """
        ),
        fixture(
            """\
            %TAG !e! tag:example.com,2026:
            ---
            # fmt: table
            rows:
              - { name: alpha,tags: [docs,markdown], enabled: true } # first row
              - { name: beta-long,tags: [yaml,fmt], enabled: false } # second row
            anchors:
              base: &base { retries: 3, mode: "safe" }
              copy: *base
            tagged: !e!widget { id:1, name:widget, notes: "a # b" }
            markdown: !markdown |
              #   Heading

              -   item one
              -   item two
            ...
            """
        ),
    ),
    (
        "anchors-aliases-tags-comments",
        fixture(
            """\
            # Shared service defaults
            defaults: &defaults
              retries: 3
              labels: [core,api]
              health: !endpoint {path: /health, method: GET} # custom-tagged flow
            services:
              - &api
                name: api
                <<: *defaults
                ports: [80,443] # preserve trailing comment
              - name: worker
                <<: *defaults
                depends_on: [*api]
            templates:
              tagged: !service &template {name: base, enabled:true}
              alias: *template
            notes: &notes !markdown |
              #   Anchored Note

              See   alias.
            reference: *notes
            """
        ),
        fixture(
            """\
            # Shared service defaults
            defaults: &defaults
              retries: 3
              labels: [core, api]
              health: !endpoint {path: /health, method: GET} # custom-tagged flow
            services:
              - &api
                name: api
                <<: *defaults
                ports: [80, 443] # preserve trailing comment
              - name: worker
                <<: *defaults
                depends_on: [*api]
            templates:
              tagged: !service &template {name: base, enabled:true}
              alias: *template
            notes: &notes !markdown |
              # Anchored Note

              See alias.
            reference: *notes
            """
        ),
    ),
]


MARKDOWN_CASES = [
    (
        "front-matter-fenced-yaml",
        fixture(
            """\
            ---
            title: "Pathological corpus"
            tags: [yaml,markdown,fixtures]
            editor_options:
              markdown:
                wrap: sentence
            ---

            #   Corpus fixture

            First sentence that should stay on its own line. Second sentence that should wrap as a separate sentence when front matter options are honored.

            ```yaml
            # fmt: table
            rows:
              - { name: alpha,tags: [docs,yaml], enabled: true }
              - { name: beta-long,tags: [markdown,front-matter], enabled: false }
            ```

            ```{python}
            #| fmt: skip
            value={ "do": "not touch" }
            ```

            ::: callout-note
            Preserve this fenced div.
            :::
            """
        ),
        fixture(
            """\
            ---
            title: Pathological corpus
            tags: [yaml, markdown, fixtures]
            editor_options:
              markdown:
                wrap: sentence
            ---

            # Corpus fixture

            First sentence that should stay on its own line.
            Second sentence that should wrap as a separate sentence when front matter options are honored.

            ```yaml
            # fmt: table
            rows:
              - {name: alpha,     tags: [docs, yaml],             enabled: true}
              - {name: beta-long, tags: [markdown, front-matter], enabled: false}
            ```

            ```{python}
            #| fmt: skip
            value={ "do": "not touch" }
            ```

            ::: callout-note
            Preserve this fenced div.
            :::
            """
        ),
    ),
    (
        "rmarkdown-nested-frontmatter-comments",
        fixture(
            """\
            ---
            title: "achor_sections"
            author: "christophe"
            date: "04/01/2022"
            output:
              html_document:
                anchor_sections:
                  style: symbol # use symbol style ("dash", "symbol", "icon")
                  depth: 2 # max depth to apply anchor on (default to max which is 6)
            ---

            #   Hello

            ##   Sub 1

            ###   Sub 2

            Content
            """
        ),
        fixture(
            """\
            ---
            title: achor_sections
            author: christophe
            date: 04/01/2022
            output:
              html_document:
                anchor_sections:
                  style: symbol # use symbol style ("dash", "symbol", "icon")
                  depth: 2 # max depth to apply anchor on (default to max which is 6)
            ---

            # Hello

            ## Sub 1

            ### Sub 2

            Content
            """
        ),
    ),
    (
        "markdown-constructs",
        fixture(
            """\
            #   Markdown constructs

            This paragraph links to [yamark](https://example.com/yamark) and uses a reference [fixture][fixture-docs] before a footnote.[^long]

            :::{.callout-tip}
            A fenced div paragraph stays inside the div and is wrapped as ordinary Markdown content while the fence markers remain intact.
            :::

            ```python
            # fmt: skip
            data={ "alpha": [1,2,3] }
            ```

            ```text
            literal code fence
                keeps indentation
            ```

            [fixture-docs]: https://example.com/docs "Fixture docs"

            [^long]: The footnote body is long enough to wrap while preserving [linked text](https://example.com/ref) and inline `code` inside the definition.
            """
        ),
        fixture(
            """\
            # Markdown constructs

            This paragraph links to [yamark](https://example.com/yamark) and uses a
            reference [fixture][fixture-docs] before a footnote.[^long]

            :::{.callout-tip}
            A fenced div paragraph stays inside the div and is wrapped as ordinary
            Markdown content while the fence markers remain intact.
            :::

            ```python
            # fmt: skip
            data = {"alpha": [1, 2, 3]}
            ```

            ```text
            literal code fence
                keeps indentation
            ```

            [fixture-docs]: https://example.com/docs "Fixture docs"

            [^long]: The footnote body is long enough to wrap while preserving
              [linked text](https://example.com/ref) and inline `code` inside the
              definition.
            """
        ),
    ),
]


@pytest.mark.parametrize(
    "name,input_text,expected",
    YAML_CASES,
    ids=[case[0] for case in YAML_CASES],
)
def test_yaml_pathological_corpus_matches_cli_output(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_exact("yamark format input.yaml", "input.yaml", input_text, expected)


@pytest.mark.parametrize(
    "name,input_text,expected",
    MARKDOWN_CASES,
    ids=[case[0] for case in MARKDOWN_CASES],
)
def test_markdown_pathological_corpus_matches_cli_output(
    name: str,
    input_text: str,
    expected: str,
) -> None:
    format_exact("yamark format input.md", "input.md", input_text, expected)
