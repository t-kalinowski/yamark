"""Explicit block indentation is relative to the enclosing YAML container."""

from pathlib import Path

import pytest
from _support import read_markdown, run_command
from yaml12 import read_yaml


def scalar_value(parsed: object, shape: str) -> str:
    if shape == "mapping":
        value = parsed["lorem"]
    elif shape == "sequence":
        value = parsed[0]
    elif shape == "nested-mapping":
        value = parsed["lorem"]["ipsum"]
    elif shape == "nested-sequence":
        value = parsed["lorem"][0]
    else:
        value = parsed
    return value.value


@pytest.mark.parametrize(
    "shape", ["mapping", "sequence", "nested-mapping", "nested-sequence", "root"]
)
@pytest.mark.parametrize("indicator", [1, 2, 4])
@pytest.mark.parametrize("retained", [False, True], ids=["relocated", "retained"])
@pytest.mark.parametrize(
    ("tag", "newline", "header_offset"),
    [("!md", "\n", 2), ("!markdown", "\r\n", 4), ("!md", "\r", 4)],
    ids=["md-lf", "markdown-crlf", "md-cr"],
)
def test_cli_preserves_explicit_parent_indent(
    tmp_path: Path,
    shape: str,
    indicator: int,
    retained: bool,
    tag: str,
    newline: str,
    header_offset: int,
) -> None:
    parent_indent = 4 if shape.startswith("nested-") else 0
    if shape == "root":
        opening = ""
        child_properties = f"{tag} "
        suffix = ""
    else:
        opening = "lorem:\n" if shape.startswith("nested-") else ""
        marker = (
            "-"
            if shape.endswith("sequence")
            else "ipsum:"
            if parent_indent
            else "lorem:"
        )
        comment = " # Lorem ipsum" if retained else ""
        opening += f"{' ' * parent_indent}{marker} {tag}{comment}\n"
        child_properties = ""
        sibling = "-" if shape.endswith("sequence") else "dolor:"
        suffix = f"{' ' * parent_indent}{sibling} [dolor,sit]\n"
    child_comment = " # Dolor sit amet." if retained else ""
    source = (
        opening
        + f"{' ' * (parent_indent + header_offset)}{child_properties}|{indicator}{child_comment}\n"
        + f"{' ' * (parent_indent + indicator + 4)}Lorem __ipsum__.\n"
        + suffix
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert scalar_value(before, shape) == "    Lorem __ipsum__.\n"
    markdown_path = tmp_path / "ipsum.md"
    markdown_path.write_text(scalar_value(before, shape), encoding="utf-8")
    markdown_before = read_markdown(markdown_path)

    args = ("yamark", "format", "--canonical", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after == before
    markdown_path.write_text(scalar_value(after, shape), encoding="utf-8")
    assert read_markdown(markdown_path) == markdown_before
    if suffix:
        assert b"[dolor, sit]" in output
    if retained:
        assert output.count(b"# Dolor sit amet.") == 1
        if shape != "root":
            assert output.count(b"# Lorem ipsum") == 1
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    "directive", ["", "# fmt: markdown\n"], ids=["ordinary", "markdown"]
)
@pytest.mark.parametrize("retained", [False, True], ids=["relocated", "retained"])
def test_cli_preserves_untagged_explicit_parent_indent(
    tmp_path: Path,
    directive: str,
    retained: bool,
) -> None:
    comment = " # Lorem ipsum" if retained else ""
    source = f"{directive}lorem:{comment}\n  |2\n      Lorem __ipsum__.\nipsum: [dolor,sit]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert before["lorem"] == "    Lorem __ipsum__.\n"
    args = ("yamark", "format", "--canonical", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert b"[dolor, sit]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
