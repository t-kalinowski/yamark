"""Markdown shortcode bodies respect nested raw regions and fragment boundaries."""

from __future__ import annotations

import pytest
from _support import run_cli_case


@pytest.mark.parametrize(
    ("opening", "closing"),
    [
        ("{{< raw >}}", "{{< /raw >}}"),
        ("{{% raw.inline %}}", "{{% /raw.inline %}}"),
    ],
)
def test_nested_raw_shortcode_can_contain_the_outer_close(
    opening: str, closing: str
) -> None:
    raw = (
        opening + "\nKeep   this payload\nexactly as written.\n"
        "{{% /notice %}}\nAnd   this payload too.\n" + closing + "\n"
    )
    source = "{{% notice %}}\n" + raw + "Following\nprose.\n{{% /notice %}}\n"
    expected = "{{% notice %}}\n" + raw + "Following prose.\n{{% /notice %}}\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )


@pytest.mark.parametrize("body", ["[link](url)", "items: [a,b]"])
@pytest.mark.parametrize(
    ("opening", "closing"),
    [("{{% notice %}}", "{{% /notice %}}"), ("::: note", ":::")],
)
def test_markdown_fragment_does_not_parse_front_matter(
    body: str, opening: str, closing: str
) -> None:
    fragment = opening + f"\n---\n\n{body}\n\n---\n" + closing + "\n"
    source = fragment + "\nFollowing\nprose.\n"
    expected = fragment + "\nFollowing prose.\n"
    for text in [source, expected]:
        run_cli_case(
            "yamark format --wrap sentence --stdin-file-path input.md --verify",
            stdin=text,
            stdout=expected,
        )
