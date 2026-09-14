"""Wrap prose around protected code spans and braced template expressions."""

from __future__ import annotations

import pytest
from _support import format_stdin_and_check, run_cli_case


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
        "`{{ unmatched` then `}}`",
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


@pytest.mark.parametrize("wrap", ["sentence", "paragraph"])
def test_markdown_shortcode_bodies_follow_wrap_setting(wrap: str) -> None:
    opening = (
        '{{% notice title="Keep   this overwide shortcode argument unchanged" %}}\n'
    )
    source = (
        opening
        + "First\nsentence with {{ foo }}. Second\nsentence.\n\n"
        + "{{% notice %}}\n"
        + "Third\nsentence with `{{ bar }}`. Fourth\nsentence.\n"
        + "{{% /notice %}}\n\n"
        + "Fifth\nsentence follows the nested shortcode. Sixth\nsentence.\n"
        + "{{% /notice %}}\n"
    )
    separator = "\n" if wrap == "sentence" else " "
    expected = (
        opening
        + f"First sentence with {{{{ foo }}}}.{separator}Second sentence.\n\n"
        + "{{% notice %}}\n"
        + f"Third sentence with `{{{{ bar }}}}`.{separator}Fourth sentence.\n"
        + "{{% /notice %}}\n\n"
        + f"Fifth sentence follows the nested shortcode.{separator}Sixth sentence.\n"
        + "{{% /notice %}}\n"
    )
    for text in [source, expected]:
        format_stdin_and_check(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    "title",
    [
        '"Keep   this title"',
        '"Keep %}} in the title"',
        r'"Keep \"%}}\" in the title"',
        '`Keep %}} and "quotes"`',
    ],
)
def test_multiline_markdown_shortcode_tags_preserve_arguments(title: str) -> None:
    opening = f'{{{{% notice\n    title={title}\n    class="wide   notice"\n%}}}}\n'
    closing = "{{% /notice %}}\n"
    source = opening + "First\nsentence. Second\nsentence.\n" + closing
    expected = opening + "First sentence.\nSecond sentence.\n" + closing
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    ("opening", "closing"),
    [
        ("{{< notice >}}", "{{< /notice >}}"),
        ("{{% greeting.inline %}}", "{{% /greeting.inline %}}"),
    ],
)
def test_shortcode_template_bodies_remain_unchanged(opening: str, closing: str) -> None:
    body = (
        opening
        + '\n{{ if .Get "name" }}\nHello,   {{ .Get "name" }}.\n{{ end }}\n'
        + closing
        + "\n"
    )
    source = body + "\nFollowing\nprose still wraps.\n"
    expected = body + "\nFollowing prose still wraps.\n"
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize("directive", ["skip", "wrap=paragraph scope=next"])
def test_markdown_shortcode_directives_apply_to_the_whole_body(directive: str) -> None:
    body = (
        "{{% notice %}}\n"
        "First   short.\nSecond   short.\n\n"
        "{{% notice %}}\nThird   short.\nFourth   short.\n{{% /notice %}}\n\n"
        "Fifth   short.\nSixth   short.\n"
        "{{% /notice %}}\n"
    )
    expected_body = (
        body
        if directive == "skip"
        else (
            "{{% notice %}}\n"
            "First short. Second short.\n\n"
            "{{% notice %}}\nThird short. Fourth short.\n{{% /notice %}}\n\n"
            "Fifth short. Sixth short.\n"
            "{{% /notice %}}\n"
        )
    )
    prefix = f"<!-- fmt: {directive} -->\n"
    suffix = "\nFollowing\nprose keeps its line break.\n"
    wrap = "sentence" if directive == "skip" else "none"
    expected_suffix = (
        "\nFollowing prose keeps its line break.\n" if directive == "skip" else suffix
    )
    source = prefix + body + suffix
    expected = prefix + expected_body + expected_suffix
    for text in [source, expected]:
        format_stdin_and_check(
            f"yamark format --wrap {wrap} --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(("left", "right"), [("%", "%"), ("<", ">")])
def test_self_closing_shortcodes_do_not_extend_the_parent(
    left: str, right: str
) -> None:
    opening = f"{{{{{left} notice {right}}}}}\n"
    child = f"{{{{{left} notice /{right}}}}}\n"
    closing = f"{{{{{left} /notice {right}}}}}\n"
    source = (
        opening
        + "First\nsentence.\n"
        + child
        + "Second\nsentence.\n"
        + closing
        + "\nFollowing\nsentence.\n"
    )
    expected = (
        opening
        + ("First sentence.\n" if left == "%" else "First\nsentence.\n")
        + child
        + ("Second sentence.\n" if left == "%" else "Second\nsentence.\n")
        + closing
        + "\nFollowing sentence.\n"
    )
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


def test_shortcode_body_directives_stay_in_the_shortcode() -> None:
    source = (
        "{{% notice %}}\n{{% notice %}}\n"
        "First\nsentence. Second\nsentence.\n"
        "{{% /notice %}}\n\n"
        "<!-- fmt: wrap=paragraph scope=file -->\n"
        "Third\nsentence. Fourth\nsentence.\n"
        "{{% /notice %}}\n\nFollowing\nprose keeps its line break.\n"
    )
    expected = (
        "{{% notice %}}\n{{% notice %}}\n"
        "First sentence. Second sentence.\n"
        "{{% /notice %}}\n\n"
        "<!-- fmt: wrap=paragraph scope=file -->\n"
        "Third sentence. Fourth sentence.\n"
        "{{% /notice %}}\n\nFollowing\nprose keeps its line break.\n"
    )
    for text in [source, expected]:
        format_stdin_and_check(
            "yamark format --wrap none --stdin-file-path input.md --verify",
            text,
            expected,
            stdin_file_path="input.md",
        )


@pytest.mark.parametrize(
    "opening",
    [
        "{{% notice",
        "{{< notice",
        '{{% notice title="unterminated',
        '{{% notice title="unterminated\nFirst   sentence.\n{{% /notice %}}',
    ],
)
def test_unterminated_shortcode_arguments_are_preserved(opening: str) -> None:
    # Until the opening tag is complete, later lines may still be arguments.
    remainder = opening + "\n\nFollowing\nprose stays unchanged.\n"
    source = "Before\nthis shortcode.\n\n" + remainder
    expected = "Before this shortcode.\n\n" + remainder
    run_cli_case(
        "yamark format --wrap sentence --stdin-file-path input.md --verify",
        stdin=source,
        stdout=expected,
    )
