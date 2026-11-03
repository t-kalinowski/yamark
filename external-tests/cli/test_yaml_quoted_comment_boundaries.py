"""Scalar layout conversion keeps following YAML comments outside the value."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("verify", [False, True], ids=["default", "verify"])
@pytest.mark.parametrize(
    ("quote", "literal"),
    [("'", True), ('"', True), ("", False), ("'", False), ('"', False)],
    ids=[
        "literal-single",
        "literal-double",
        "folded-plain",
        "folded-single",
        "folded-double",
    ],
)
@pytest.mark.parametrize(
    ("prefix", "continuation", "suffix", "body_indent", "newline"),
    [
        ("", "  ", "---\nipsum: [lorem,ipsum]\n", 2, "\n"),
        ("lorem: ", "  ", "ipsum: [lorem,ipsum]\n", 2, "\r\n"),
        ("- ", "  ", "- [lorem,ipsum]\n", 2, "\r"),
        ("root:\n    lorem: ", "      ", "    ipsum: [lorem,ipsum]\n", 4, "\n"),
    ],
    ids=["root", "mapping", "sequence", "reindented-mapping"],
)
@pytest.mark.parametrize(
    ("comment_delta", "gap"),
    [(-1, ""), (0, "\n"), (2, "\n\n")],
    ids=["dedented", "same-indent-after-blank", "deeper-after-blanks"],
)
def test_cli_keeps_following_comments_outside_converted_scalars(
    tmp_path: Path,
    verify: bool,
    quote: str,
    literal: bool,
    prefix: str,
    continuation: str,
    suffix: str,
    body_indent: int,
    newline: str,
    comment_delta: int,
    gap: str,
) -> None:
    value = (
        f"Lorem ipsum\n\n{continuation}dolor sit amet."
        if literal
        else f"Lorem ipsum dolor sit amet,\n{continuation}"
        "consectetur adipiscing elit, sed do eiusmod tempor incididunt."
    )
    comment = f"{' ' * (body_indent + comment_delta)}# Outside the scalar.\n"
    source = f"{prefix}{quote}{value}{quote}\n{gap}{comment}{suffix}".replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path, multi=True)
    args = ["yamark", "format", "--prose-width", "36"]
    if verify:
        args.append("--verify")

    run_command(*args, path, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert output.count(comment.replace("\n", newline).encode()) == 1
    assert b"[lorem, ipsum]" in output
    marker = b"|-" if literal else b">-"
    converted = any(line.endswith(marker) for line in output.splitlines())
    assert converted is (comment_delta < 0)

    run_command(*args, path, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("verify", [False, True], ids=["default", "verify"])
def test_cli_keeps_next_node_skip_after_quoted_scalar(
    tmp_path: Path, verify: bool
) -> None:
    source = (
        "lorem: 'Lorem ipsum\n\n  dolor sit amet.'\n\n"
        "  # fmt: skip scope=next\n"
        "ipsum: [lorem,ipsum]\n"
        "dolor: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    args = ["yamark", "format"]
    if verify:
        args.append("--verify")

    run_command(*args, path, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.count(b"  # fmt: skip scope=next\n") == 1
    assert b"ipsum: [lorem,ipsum]\n" in output
    assert b"dolor: [dolor, sit]\n" in output

    run_command(*args, path, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_stops_comment_boundary_check_at_a_dedented_comment(tmp_path: Path) -> None:
    source = (
        "lorem: 'Lorem ipsum\n\n  dolor sit amet.'\n"
        " # Outside the scalar.\n"
        "    # Still outside the scalar.\n"
        "ipsum: [lorem,ipsum]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)

    run_command("yamark", "format", "--verify", path, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path) == before
    assert b"lorem: |-\n" in output
    assert output.count(b" # Outside the scalar.\n") == 1
    assert output.count(b"    # Still outside the scalar.\n") == 1

    run_command("yamark", "format", "--verify", path, cwd=tmp_path)
    assert path.read_bytes() == output
