"""Out-of-line scalar properties survive formatting and semantic verification."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize("sequence", [False, True], ids=["mapping", "sequence"])
@pytest.mark.parametrize(
    ("parent_properties", "child_properties"),
    [
        ("&ipsum", ""),
        ("!!str &ipsum", ""),
        ("!lorem &ipsum", ""),
        ("!!str", "&ipsum "),
        ("&ipsum", "!!str "),
        ("!lorem", "&ipsum "),
        ("&ipsum", "!lorem "),
    ],
)
def test_cli_retains_parent_scalar_properties(
    tmp_path: Path,
    newline: str,
    quote: str,
    sequence: bool,
    parent_properties: str,
    child_properties: str,
) -> None:
    prefix = "- " if sequence else "lorem: "
    preserved = (
        f"{prefix}{parent_properties}\n"
        f"  {child_properties}{quote}Lorem ipsum\n"
        f"    dolor sit amet.{quote}\n"
    ).replace("\n", newline)
    suffix = (
        "- *ipsum\n- [dolor,sit]\n"
        if sequence
        else "ipsum: *ipsum\ndolor: [dolor,sit]\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes((preserved + suffix).encode())
    before = read_yaml(path)

    args = ("yamark", "format", "--prose-width", "24", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.encode() in output
    assert b"[dolor, sit]" in output
    after = read_yaml(path)
    assert after == before
    if sequence:
        assert after[0] == after[1]
    else:
        assert after["lorem"] == after["ipsum"]
    for property_text in (parent_properties + " " + child_properties).split():
        assert output.count(property_text.encode()) == 1
    assert output.count(b"*ipsum") == 1

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("tag", "scalar", "expected"),
    [
        ("!!str", "true", "true"),
        ("!!str", '"1\\\n    2"', "12"),
        ("!!bool", "true", True),
        ("!!int", "12", 12),
        ("!!float", "1.5", 1.5),
        ("!!null", "null", None),
    ],
)
def test_cli_preserves_parent_tag_scalar_types(
    tmp_path: Path,
    tag: str,
    scalar: str,
    expected: object,
) -> None:
    preserved = (
        f"lorem: {tag} &ipsum # Lorem ipsum\n"
        "  # Dolor sit amet.\n"
        f"  {scalar}\n"
    )
    source = preserved + "ipsum: *ipsum\ndolor: [dolor,sit]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert before["lorem"] == expected
    assert type(before["lorem"]) is type(expected)

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.encode() in output
    assert b"[dolor, sit]" in output
    after = read_yaml(path)
    assert after == before
    assert type(after["lorem"]) is type(expected)
    assert type(after["ipsum"]) is type(expected)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize(
    ("directive", "parent_properties", "child_properties"),
    [
        ("", "&ipsum", "!md "),
        ("", "!md", "&ipsum "),
        ("", "!md &ipsum", ""),
        ("# fmt: markdown\n", "&ipsum", ""),
    ],
)
def test_cli_formats_markdown_with_parent_scalar_properties(
    tmp_path: Path,
    newline: str,
    quote: str,
    directive: str,
    parent_properties: str,
    child_properties: str,
) -> None:
    source = (
        f"{directive}lorem: {parent_properties}\n"
        f"  {child_properties}{quote}Lorem __ipsum__\n"
        f"    dolor sit amet.{quote}\n"
        "ipsum: *ipsum\ndolor: [dolor,sit]\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)["lorem"]
    if directive:
        assert before == "Lorem __ipsum__ dolor sit amet."
    else:
        assert before.tag == "!md"
        assert before.value == "Lorem __ipsum__ dolor sit amet."

    args = ("yamark", "format", "--wrap", "none", "--canonical", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    if directive:
        assert after["lorem"] == "Lorem **ipsum** dolor sit amet.\n"
    else:
        assert after["lorem"].tag == "!md"
        assert after["lorem"].value == "Lorem **ipsum** dolor sit amet.\n"
    assert after["ipsum"] == after["lorem"]
    assert output.count(b"&ipsum") == 1
    assert output.splitlines().count(b"ipsum: *ipsum") == 1
    assert output.count(b"!md") == (0 if directive else 1)
    assert b"[dolor, sit]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
