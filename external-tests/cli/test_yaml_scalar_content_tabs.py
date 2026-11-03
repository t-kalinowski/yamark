"""Scalar content tabs remain distinct from YAML indentation."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize(
    ("prefix", "indent", "suffix"),
    [
        ("", "  ", ""),
        ("lorem: ", "  ", "ipsum: [dolor,sit]\n"),
        ("- ", "  ", "- [dolor,sit]\n"),
        ("- lorem: ", "    ", "  ipsum: [dolor,sit]\n"),
        ("- - ", "    ", "  - [dolor,sit]\n"),
        ("- - lorem: ", "      ", "    ipsum: [dolor,sit]\n"),
    ],
    ids=[
        "root",
        "mapping",
        "sequence",
        "sequence-mapping",
        "compact-sequence",
        "compact-mapping",
    ],
)
def test_cli_converts_tab_leading_quotes_to_verifiable_literals(
    tmp_path: Path, newline: str, quote: str, prefix: str, indent: str, suffix: str
) -> None:
    source = (f"{prefix}{quote}\tLorem\n\n{indent}ipsum{quote}\n{suffix}").replace(
        "\n", newline
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    args = ("yamark", "format", "--verify", path)

    run_command(*args, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path) == before
    assert b"|-" in output
    assert b"\tLorem" in output
    if suffix:
        assert b"[dolor, sit]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("header", ["|", "|-", "|2-", ">-", ">2-"])
@pytest.mark.parametrize("first_line", ["Lorem", "\tLorem", "\t", "\t \t"])
@pytest.mark.parametrize("indent_width", [2, 4])
def test_cli_formats_siblings_of_block_scalar_content_tabs(
    tmp_path: Path, header: str, first_line: str, indent_width: int
) -> None:
    source = (
        f"# Lorem ipsum\nlorem: {header}\n  {first_line}\n  \tipsum\n"
        "dolor: [sit,amet]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    args = ("yamark", "format", "--indent-width", str(indent_width), "--verify", path)

    run_command(*args, cwd=tmp_path)

    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.startswith(b"# Lorem ipsum\n")
    assert b"dolor: [sit, amet]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    "value",
    ['"\\t\\n"', '"\\t\\n  Lorem\\n"', "|\n  \t\n    Lorem"],
    ids=["tab-only", "tab-then-spaces", "literal-tab-then-spaces"],
)
def test_cli_retains_tabs_that_determine_block_indentation(
    tmp_path: Path, value: str
) -> None:
    path = tmp_path / "lorem.yaml"
    path.write_bytes(f"lorem: {value}\ndolor: [sit,amet]\n".encode())
    before = read_yaml(path)
    args = ("yamark", "format", "--indent-width", "4", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert b"dolor: [sit, amet]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    "source",
    [
        "lorem:\n\tipsum: [dolor,sit]\n",
        "lorem: |2\n \tipsum\ndolor: [sit,amet]\n",
        "lorem: |\n  ipsum\n \tdolor\nsit: [amet,lorem]\n",
        "lorem: |\n\t\ndolor: [sit,amet]\n",
    ],
    ids=[
        "mapping-indent",
        "explicit-body-indent",
        "implicit-body-indent",
        "unindented-tab-only",
    ],
)
def test_cli_keeps_actual_tab_indentation_preserved(
    tmp_path: Path, source: str
) -> None:
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == source.encode()
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == source.encode()
