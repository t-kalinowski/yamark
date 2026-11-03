"""Quoted continuations remain valid when their containing YAML moves."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize(
    "value",
    [
        "'hello\nworld'",
        '"hello\nworld"',
        "'hello\n  world'",
        '"hello\\\nworld"',
        '"hello\\x20\n\\x20world"',
        "'hello\n    ---\n    ...\nworld'",
    ],
    ids=[
        "single",
        "double",
        "partial-indent",
        "escaped-break",
        "escaped-spaces",
        "document-marker-content",
    ],
)
def test_cli_keeps_root_quoted_continuations_when_removing_indentation(
    tmp_path: Path, newline: str, value: str
) -> None:
    source = ("    " + value + "\n").replace("\n", newline).encode()
    expected = (value + "\n").replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected
    assert read_yaml(path, multi=True) == before

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_preserves_unsupported_tab_indentation(tmp_path: Path, newline: str) -> None:
    source = "    'hello\n \tworld'\n".replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    for _ in range(2):
        run_command("yamark", "format", "--verify", path, cwd=tmp_path)
        assert path.read_bytes() == source
        assert read_yaml(path, multi=True) == before


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize(
    ("properties", "comment"),
    [("&lorem ", ""), ("!!str &lorem ", " # retained")],
    ids=["anchor", "tag-anchor-comment"],
)
def test_cli_keeps_root_quote_properties_during_reindentation(
    tmp_path: Path, newline: str, quote: str, properties: str, comment: str
) -> None:
    value = f"{properties}{quote}hello\nworld{quote}{comment}\n"
    source = ("    " + value).replace("\n", newline).encode()
    expected = value.replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected
    assert read_yaml(path, multi=True) == before

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize(
    ("source_indent", "continuation_indent", "output_continuation_indent"),
    [(4, 5, 5), (1, 2, 3)],
    ids=["moves-left", "moves-right"],
)
def test_cli_reindents_nested_quoted_scalars_without_changing_values(
    tmp_path: Path,
    newline: str,
    quote: str,
    source_indent: int,
    continuation_indent: int,
    output_continuation_indent: int,
) -> None:
    source = (
        "lorem:\n"
        f"{' ' * source_indent}ipsum: {quote}hello\n"
        f"{' ' * continuation_indent}world{quote}\n"
        f"{' ' * source_indent}dolor: [x,y]\n"
    ).replace("\n", newline).encode()
    expected = (
        "lorem:\n"
        f"  ipsum: {quote}hello\n"
        f"{' ' * output_continuation_indent}world{quote}\n"
        "  dolor: [x, y]\n"
    ).replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--indent-width", "2", "--verify", path)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected
    assert read_yaml(path, multi=True) == before

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == expected
