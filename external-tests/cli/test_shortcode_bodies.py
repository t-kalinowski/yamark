"""Format Markdown shortcode bodies while preserving shortcode arguments."""

from __future__ import annotations

import pytest
from _support import format_stdin_and_check, run_cli_case


@pytest.mark.parametrize("wrap", ["sentence", "paragraph"])
def test_markdown_shortcode_bodies_follow_wrap_setting(wrap: str) -> None:
    opening = (
        '{{% notice title="Keep   this overwide shortcode argument unchanged" %}}\n'
    )
    source = (
        opening
        + "First\nsentence with a label. Second\nsentence.\n\n"
        + "{{% notice %}}\n"
        + "Third\nsentence with `literal`. Fourth\nsentence.\n"
        + "{{% /notice %}}\n\n"
        + "Fifth\nsentence follows the nested shortcode. Sixth\nsentence.\n"
        + "{{% /notice %}}\n"
    )
    separator = "\n" if wrap == "sentence" else " "
    expected = (
        opening
        + f"First sentence with a label.{separator}Second sentence.\n\n"
        + "{{% notice %}}\n"
        + f"Third sentence with `literal`.{separator}Fourth sentence.\n"
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
