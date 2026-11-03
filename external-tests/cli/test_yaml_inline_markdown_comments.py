"""Inline Markdown targets keep following YAML comments outside their values."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("target", ["tag", "directive"])
@pytest.mark.parametrize("quote", ["", "'", '"'], ids=["plain", "single", "double"])
@pytest.mark.parametrize(
    (
        "opening",
        "prefix",
        "suffix",
        "target_path",
        "body_indent",
        "comment_indent",
        "gap",
        "newline",
    ),
    [
        ("", "", "", (0,), 2, 2, "", "\n"),
        ("", "lorem: ", "ipsum: [dolor,sit]\n", (0, "lorem"), 2, 4, "\n", "\r\n"),
        ("", "- ", "- [dolor,sit]\n", (0, 0), 2, 2, "", "\r"),
        (
            "root:\n",
            "    lorem: ",
            "    ipsum: [dolor,sit]\n",
            (0, "root", "lorem"),
            4,
            4,
            "",
            "\n",
        ),
        (
            "root:\n",
            " - ",
            " - [dolor,sit]\n",
            (0, "root", 0),
            4,
            4,
            "\n\n",
            "\r\n",
        ),
        (
            "root:\n",
            "    lorem: ",
            "    ipsum: [dolor,sit]\n",
            (0, "root", "lorem"),
            4,
            3,
            "",
            "\r",
        ),
    ],
    ids=[
        "root-equal",
        "mapping-deeper-after-blank",
        "sequence-equal",
        "mapping-moves-left",
        "sequence-moves-right-after-blanks",
        "safe-dedented-comment",
    ],
)
def test_cli_formats_inline_markdown_without_absorbing_comments(
    tmp_path: Path,
    target: str,
    quote: str,
    opening: str,
    prefix: str,
    suffix: str,
    target_path: tuple[int | str, ...],
    body_indent: int,
    comment_indent: int,
    gap: str,
    newline: str,
) -> None:
    directive = ""
    tag = "!md " if target == "tag" else ""
    if target == "directive":
        prefix_indent = len(prefix) - len(prefix.lstrip(" "))
        directive = " " * prefix_indent + "# fmt: markdown\n"
    comment = " " * comment_indent + "# Outside the scalar.\n"
    source = (
        f"{opening}{directive}{prefix}{tag}{quote}Lorem __ipsum__.{quote}\n"
        f"{gap}{comment}{suffix}"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path, multi=True)

    args = (
        "yamark",
        "format",
        "--wrap",
        "none",
        "--canonical",
        "--verify",
        path,
    )
    run_command(*args, cwd=tmp_path)

    output = path.read_bytes()
    expected_indent = max(body_indent, comment_indent + 1)
    assert (" " * expected_indent + "Lorem **ipsum**.").encode() in output.splitlines()
    assert output.count(comment.replace("\n", newline).encode()) == 1
    if suffix:
        assert b"[dolor, sit]" in output
    after = read_yaml(path, multi=True)
    before_parent = before
    after_parent = after
    for key in target_path[:-1]:
        before_parent = before_parent[key]
        after_parent = after_parent[key]
    before_scalar = before_parent[target_path[-1]]
    after_scalar = after_parent[target_path[-1]]
    if target == "tag":
        assert before_scalar.tag == after_scalar.tag == "!md"
        assert before_scalar.value == "Lorem __ipsum__."
        assert after_scalar.value == "Lorem **ipsum**.\n"
    else:
        assert before_scalar == "Lorem __ipsum__."
        assert after_scalar == "Lorem **ipsum**.\n"
    before_parent[target_path[-1]] = after_scalar
    assert after == before

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("target", ["tag", "directive"])
def test_cli_keeps_empty_inline_markdown_quoted_before_comments(
    tmp_path: Path, target: str
) -> None:
    directive = "# fmt: markdown\n" if target == "directive" else ""
    tag = "!md " if target == "tag" else ""
    source = f"{directive}lorem: {tag}''\n  # Outside the scalar.\nipsum: [dolor,sit]\n"
    expected = (
        f'{directive}lorem: {tag}""\n  # Outside the scalar.\nipsum: [dolor, sit]\n'
    )
    path = tmp_path / "lorem.yaml"
    path.write_text(source)
    before = read_yaml(path)

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    assert path.read_text() == expected
    assert read_yaml(path) == before

    run_command(*args, cwd=tmp_path)
    assert path.read_text() == expected
