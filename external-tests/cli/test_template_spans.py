"""Wrap prose around protected code spans and braced template expressions."""

from __future__ import annotations

import pytest
from _support import format_stdin_and_check, run_cli_case


@pytest.mark.parametrize(
    "container",
    [
        "> Before `{{\n> keep   _this_\n> }}` after.\n",
        "> > Before `{{\n> > keep   _this_\n> > }}` after.\n",
        "- Before `{{\n  keep   _this_\n  }}` after.\n",
        "Term\n: Before `{{\n    keep   _this_\n    }}` after.\n",
        "[^n]: Before `{{\n  keep   _this_\n  }}` after.\n",
        "[^n]: Before `{{\n    keep   _this_\n    }}` after.\n",
        "> Before `code\n> keep   _this_` after.\n",
    ],
    ids=[
        "blockquote",
        "nested-blockquote",
        "list",
        "definition",
        "footnote",
        "indented-footnote",
        "ordinary-code",
    ],
)
@pytest.mark.parametrize("wrap", ["sentence", "20"])
@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_containers_preserve_multiline_code(
    container: str, wrap: str, newline: str
) -> None:
    source = f"Leading\nprose.\n\n{container}\nFollowing\nprose.\n"
    expected = f"Leading prose.\n\n{container}\nFollowing prose.\n"
    expected = expected.replace("\n", newline)
    for text in [source.replace("\n", newline), expected]:
        run_cli_case(
            f"yamark format --canonical --wrap {wrap} --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("newline", ["\n", "\r\n"])
@pytest.mark.parametrize(
    ("code", "expected"),
    [
        (
            "`{{\n  render(customer.delivery_address)\n}}`",
            "Before `{{\n  render(customer.delivery_address)\n}}` after the\n"
            "template and more\nprose.\n",
        ),
        (
            "`{{ render_customer_notice(\n  address\n) }}`",
            "Before\n`{{ render_customer_notice(\n  address\n) }}` after the\n"
            "template and more\nprose.\n",
        ),
        (
            "`{{\n  render(customer.delivery_address) }}`",
            "Before `{{\n  render(customer.delivery_address) }}`\n"
            "after the template\nand more prose.\n",
        ),
    ],
)
def test_multiline_template_code_wraps_by_its_first_and_last_lines(
    code: str, expected: str, newline: str
) -> None:
    source = f"Before   {code} after the template and more prose.\n"
    expected = expected.replace("\n", newline)
    for text in [source.replace("\n", newline), expected]:
        run_cli_case(
            "yamark format --wrap 20 --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    ("opening", "closing"),
    [
        ("{{% notice %}}", "{{% /notice %}}"),
        ("{{< notice >}}", "{{< /notice >}}"),
        ("{{% notice %}}", "{{% unrelated %}}"),
    ],
)
def test_shortcode_tags_leave_intervening_markdown_to_format(
    opening: str, closing: str
) -> None:
    source = (
        f"{opening}\nFormat   this paragraph\nas Markdown.\n\n"
        f"#   A heading\n\n{closing}\nFollowing\nprose.\n"
    )
    expected = (
        f"{opening}\nFormat this paragraph as Markdown.\n\n"
        f"# A heading\n\n{closing}\nFollowing prose.\n"
    )
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize(
    ("wrap", "expected"),
    [
        ("sentence", "Third sentence with {{ foo }}.\nFourth sentence.\n"),
        ("paragraph", "Third sentence with {{ foo }}. Fourth sentence.\n"),
        ("20", "Third sentence with\n{{ foo }}. Fourth\nsentence.\n"),
        ("8", "Third\nsentence\nwith\n{{ foo }}.\nFourth\nsentence.\n"),
    ],
)
def test_wrap_around_braced_template_text(wrap: str, expected: str) -> None:
    source = "Third\nsentence with {{ foo }}. Fourth\nsentence.\n"
    for text in [source, expected]:
        format_stdin_and_check(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    ("wrap", "expected"),
    [
        ("sentence", "First sentence with `${{ foo }}`.\nSecond sentence.\n"),
        ("paragraph", "First sentence with `${{ foo }}`. Second sentence.\n"),
        ("20", "First sentence with\n`${{ foo }}`. Second\nsentence.\n"),
    ],
)
@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_wrap_around_template_text_in_inline_code(
    wrap: str, expected: str, newline: str
) -> None:
    source = "First\nsentence with `${{ foo }}`. Second\nsentence.\n"
    expected = expected.replace("\n", newline)
    for text in [source.replace("\n", newline), expected]:
        format_stdin_and_check(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    "code",
    [
        "`{{ keep   this }}`",
        "`{% keep   this %}`",
        "`{# keep   this #}`",
        "`<% keep   this %>`",
        "``{{ keep ` this }}``",
        "`{{ keep `` this }}`",
        "``{{ keep ``` this }}``",
        "``{{ keep ``` x ` _this_ }}``",
        "`{{ keep\nthis }}`",
        "`{{ café   λ }}`",
        "`{{ keep   this }}\\`",
        "`{{ keep }}` and `{% more %}`",
    ],
)
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_template_delimiters_inside_complete_code_spans(
    code: str, canonical: str
) -> None:
    source = f"First\nsentence with {code}. Second\nsentence.\n"
    expected = f"First sentence with {code}.\nSecond sentence.\n"
    for text in [source, expected]:
        format_stdin_and_check(
            f"yamark format {canonical} --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    "template",
    [
        "{{ keep   this }}",
        "`code` {{ keep   this }}",
        "{{ keep   this }} `code`",
        "`{{ code }}` then {{ keep   this }}",
        "`{{ code }}` then {% keep   this %}",
        "{{ keep `code`   this }}",
        "{{ keep   _this_ }}",
        "{% keep   this %}",
        "{# keep   this #}",
        "[target](https://example.com/`{{key}}`)",
        '[target](url "`{{ title }}`")',
        '[![alt](inner)](outer "`{{ title }}`")',
        '<span title="`{{ title }}`">text</span>',
        "$`{{ math }}`$",
    ],
)
@pytest.mark.parametrize("canonical", ["", "--canonical"])
def test_wrap_around_protected_template_tokens(template: str, canonical: str) -> None:
    source = f"First\nsentence with {template}. Second\nsentence.\n"
    expected = f"First sentence with {template}.\nSecond sentence.\n"
    for text in [source, expected]:
        format_stdin_and_check(
            f"yamark format {canonical} --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    "template",
    [
        "`{{ keep   this }}",
        "``{{ keep   this }}`",
        "``{{ keep   this }}```",
        "\\`{{ keep   this }}\\`",
        "<% keep   this %>",
        "`{{ unmatched` then `}}`",
    ],
)
def test_unsupported_inline_syntax_preserves_paragraph(template: str) -> None:
    source = f"First\nsentence with {template}. Second\nsentence.\n"
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=source,
    )


@pytest.mark.parametrize("configuration", ["file", "directive"])
def test_custom_template_delimiters_inside_code(configuration: str) -> None:
    files = {}
    prefix = ""
    if configuration == "file":
        files["yamark.toml"] = (
            '[template]\nadd_delimiters = [{ open = "<<", close = ">>" }]\n'
        )
    else:
        prefix = '<!-- fmt: template.delimiters "<<" ">>" scope=file -->\n'
    source = prefix + "First\nsentence with `<< keep   this >>`. Second\nsentence.\n"
    expected = prefix + "First sentence with `<< keep   this >>`.\nSecond sentence.\n"
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
            files=files,
        )


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        ("#   `{{ title }}` ##\n", "# `{{ title }}`\n"),
        ("`{{ title }}`\n====\n", "# `{{ title }}`\n"),
        (
            "- First\n  sentence with `{{ foo }}`. Second\n  sentence.\n",
            "- First sentence with `{{ foo }}`.\n  Second sentence.\n",
        ),
        (
            "> First\n> sentence with `{{ foo }}`. Second\n> sentence.\n",
            "> First sentence with `{{ foo }}`.\n> Second sentence.\n",
        ),
        (
            "First\nsentence with [`{{ foo }}`](url). Second\nsentence.\n",
            "First sentence with [`{{ foo }}`](url).\nSecond sentence.\n",
        ),
    ],
)
def test_inline_code_templates_in_other_markdown_blocks(
    source: str, expected: str
) -> None:
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    ("suffix", "prefix", "opening", "closing"),
    [
        ("md", "", "```markdown\n", "```\n"),
        ("md", "", "<!-- fmt: markdown -->\n", ""),
        ("py", "", '# fmt: markdown\ntext = """\n', '"""\n'),
        ("R", "", '# fmt: markdown\ntext <- r"(\n', ')"\n'),
        ("yaml", "  ", "text: !markdown |\n", ""),
        ("yaml", "  ", "# fmt: markdown\ntext: |\n", ""),
    ],
)
@pytest.mark.parametrize("template", ["`{{ foo }}`", "{{ foo }}"])
def test_inline_code_templates_in_marked_markdown(
    suffix: str, prefix: str, opening: str, closing: str, template: str
) -> None:
    source = f"First\nsentence with {template}. Second\nsentence.\n"
    expected = f"First sentence with {template}.\nSecond sentence.\n"
    source = (
        opening + "".join(prefix + line for line in source.splitlines(True)) + closing
    )
    expected = (
        opening + "".join(prefix + line for line in expected.splitlines(True)) + closing
    )
    for text in [source, expected]:
        run_cli_case(
            f"yamark format --wrap sentence --stdin-file-path input.{suffix} --verify",
            stdin=text,
            stdout=expected,
        )


def test_yaml_template_text_with_backticks_is_preserved() -> None:
    source = 'value: "`{{ keep   this }}`"\nitems: [a,b]\n'
    expected = 'value: "`{{ keep   this }}`"\nitems: [a, b]\n'
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --stdin-file-path input.yaml --verify",
            text,
            expected,
            stdin_file_path="input.yaml",
        )
