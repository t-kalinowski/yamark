"""Template preservation follows Markdown syntax and decoded YAML contents."""

from __future__ import annotations

import pytest
from _support import run_cli_case


@pytest.mark.parametrize("quote", ['"', "'"])
@pytest.mark.parametrize("heading", [False, True])
def test_template_in_quoted_html_attribute_is_not_code(
    quote: str, heading: bool
) -> None:
    tag = f"<span title={quote}> `<% keep   this %>`{quote}>text</span>"
    source = (
        f"#   Heading {tag}   ##\n"
        if heading
        else f"First\nsentence with {tag}. Second\nsentence.\n"
    )
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("destination", [r"foo\_bar", r"foo\(bar\)"])
def test_template_in_escaped_link_destination_is_not_code(destination: str) -> None:
    source = f"#   Heading [x]({destination}/`<%value%>`)   ##\n"
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("spacing", ["", " ", "\t"])
def test_template_in_image_attributes_is_not_code(spacing: str) -> None:
    source = f'![x](url){spacing}{{fig-alt="`<% keep   this %>`"}}\n'
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("template", ["{{ keep   this }}", "`{{ keep   this }}`"])
def test_braced_templates_survive_image_attribute_normalization(template: str) -> None:
    source = f'![x](url) {{ fig-alt="{template}" }}\n'
    expected = f'![x](url){{fig-alt="{template}"}}\n'
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("heading", ["# Title", "Title\n====="])
@pytest.mark.parametrize("prefix", ["", "> "])
def test_template_in_heading_attributes_is_not_code(heading: str, prefix: str) -> None:
    lines = heading.splitlines()
    lines[0] += ' {fig-alt="`<%= %q(keep   this) %>`"}'
    source = "".join(prefix + line + "\n" for line in lines)
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize(
    "expression",
    [
        '{{ "}}" ~ "keep   this" }}',
        "{{ '}}' ~ 'keep   this' }}",
        r'{{ "escaped \"}}" ~ "keep   this" }}',
        '{{ render({"label": "} keep   this", "other": "{{"}) }}',
    ],
)
@pytest.mark.parametrize("wrap", ["sentence", "20"])
def test_quoted_braces_stay_inside_the_template(expression: str, wrap: str) -> None:
    source = f"First\nsentence with {expression}. Second\nsentence.\n"
    expected = (
        f"First sentence with {expression}.\nSecond sentence.\n"
        if wrap == "sentence"
        else f"First sentence with\n{expression}.\nSecond sentence.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("backslash", [r"\\", r"\u005c", r"\x5c"])
def test_quoted_markdown_yaml_uses_decoded_backtick_escapes(backslash: str) -> None:
    scalar = (
        f'doc: !markdown "First   sentence with {backslash}`<% keep   this %>'
        f'{backslash}`. Second sentence.\\n"\n'
    )
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.yaml --verify",
        stdin=scalar + "items: [a,b]\n",
        stdout=scalar + "items: [a, b]\n",
    )


@pytest.mark.parametrize(
    "source",
    [
        "-   Before `\n-   <% keep   this %>\n-   After `\n",
        (
            "First `\n:   First description.\nMiddle\n:   <% keep   this %>\n"
            "Last `\n:   Last description.\n"
        ),
        "> #   First `\n>\n> <% keep   this %>\n>\n> #   Last `\n",
        "> > #   First `\n> >\n> > <% keep   this %>\n> >\n> > #   Last `\n",
    ],
    ids=["list-items", "definitions", "blockquote-paragraphs", "nested-blockquote"],
)
def test_code_spans_do_not_cross_container_blocks(source: str) -> None:
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("prefix", ["- ", "> "])
def test_code_spans_can_cross_lines_in_one_container_paragraph(prefix: str) -> None:
    continuation = "  " if prefix == "- " else prefix
    source = (
        f"{prefix}First\n{continuation}sentence with `<% keep\n"
        f"{continuation}this %>`. Second\n{continuation}sentence.\n"
    )
    expected = (
        f"{prefix}First sentence with `<% keep this %>`.\n"
        f"{continuation}Second sentence.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )
