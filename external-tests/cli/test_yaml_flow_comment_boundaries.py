"""Flow expansion keeps following comments outside its final scalar value."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("verify", [False, True], ids=["default", "verify"])
@pytest.mark.parametrize(
    "value",
    [
        '"line one\\nline two"',
        "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod",
        "'Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod'",
        '"Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod"',
    ],
    ids=["literal-double", "folded-plain", "folded-single", "folded-double"],
)
@pytest.mark.parametrize(
    ("opening", "closing", "body_indent", "newline"),
    [
        ("[", "\n]", 2, "\n"),
        ("lorem: [", "\n  ]", 4, "\r"),
        ("lorem: {ipsum: ", "\n}", 4, "\r\n"),
        ("lorem: {ipsum: [", "\n]\n}", 6, "\n"),
        ("lorem: [{ipsum: ", "\n}\n]", 6, "\n"),
    ],
    ids=["sequence", "mapping-value", "mapping", "nested-sequence", "nested-mapping"],
)
@pytest.mark.parametrize(
    "comment_delta", [-1, 0, 2], ids=["dedented", "equal", "deeper"]
)
def test_cli_keeps_comments_after_expanded_flow_collection(
    tmp_path: Path,
    verify: bool,
    value: str,
    opening: str,
    closing: str,
    body_indent: int,
    newline: str,
    comment_delta: int,
) -> None:
    comment = f"{' ' * (body_indent + comment_delta)}# Outside the collection.\n"
    separator = "---\n" if opening == "[" else ""
    source = (
        f"{opening}{value}{closing}\n\n{comment}{separator}control: [x,y]\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path, multi=True)
    args = ["yamark", "format", "--line-width", "20", "--prose-width", "36"]
    if verify:
        args.append("--verify")

    run_command(*args, path, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert output.count(comment.replace("\n", newline).encode()) == 1
    assert b"control: [x, y]" in output
    converted = any(line.endswith((b"|-", b">-")) for line in output.splitlines())
    assert converted is (comment_delta < 0)

    run_command(*args, path, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("collection", ["sequence", "mapping"])
@pytest.mark.parametrize("closing_newline", ["", "\n"], ids=["same-line", "next-line"])
def test_cli_converts_nonfinal_flow_values_before_indented_comments(
    tmp_path: Path, collection: str, closing_newline: str
) -> None:
    source = (
        f'lorem: ["line one\\nline two", last{closing_newline}]\n'
        if collection == "sequence"
        else f'lorem: {{first: "line one\\nline two", second: last{closing_newline}}}\n'
    )
    source += "    # Outside the collection.\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)

    args = ("yamark", "format", "--line-width", "20", "--verify", path)
    run_command(*args, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path) == before
    assert any(line.endswith(b"|-") for line in output.splitlines())
    assert output.count(b"    # Outside the collection.\n") == 1

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
