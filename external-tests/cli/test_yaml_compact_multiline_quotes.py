"""Compact YAML collections preserve multiline quoted scalar values."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize(
    ("prefix", "indent", "suffix"),
    [
        ("- ", "  ", ""),
        ("lorem: ", "  ", "ipsum: dolor\n"),
        ("lorem:\n  - ", "    ", "  - dolor\n"),
        ("- lorem: ", "    ", "  ipsum: dolor\n"),
    ],
    ids=["sequence", "mapping", "nested-sequence", "sequence-mapping"],
)
@pytest.mark.parametrize(
    "value",
    [
        "'lorem\nipsum'",
        '"lorem\nipsum"',
        "'lorem\n\nipsum'",
        '"lorem\n\n\nipsum"',
        "'lorem''ipsum\ndolor'",
        '"lorem\\\nipsum"',
        '"lorem\\\n\nipsum"',
        '"lorem\\x20\n\\x20ipsum"',
        "' lorem\nipsum '",
        '"lorem\\\\ipsum\ndolor"',
        '"lorem\\"ipsum\ndolor"',
    ],
    ids=[
        "single",
        "double",
        "single-blank",
        "double-blanks",
        "single-escape",
        "continuation",
        "continuation-blank",
        "escaped-spaces",
        "boundary-spaces",
        "backslash",
        "double-escape",
    ],
)
def test_cli_compacts_multiline_quotes_without_changing_values(
    tmp_path: Path,
    newline: str,
    prefix: str,
    indent: str,
    suffix: str,
    value: str,
) -> None:
    value = value.replace("\n", "\n" + indent)
    source = (prefix + value + "\n" + suffix).replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--compact", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert b"[" in output or b"{" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize(
    ("prefix", "value", "suffix", "preserved"),
    [
        ("- ", "lorem\n  ipsum", " # dolor\n", "# dolor"),
        ("- &lorem ", "lorem\n  ipsum", "\n- *lorem\n", "&lorem"),
        ("- !!str &lorem ", "lorem\n  ipsum", "\n- *lorem\n", "!!str &lorem"),
        ("- ", "lorem\n  {{ ipsum }}", "\n", "lorem\n  {{ ipsum }}"),
        (
            '# fmt: template.delimiters "<<" ">>" scope=file\n- ',
            "lorem\n  << ipsum >>",
            "\n",
            "lorem\n  << ipsum >>",
        ),
        ("# fmt: skip\n- ", "lorem\n  ipsum", "\n", "lorem\n  ipsum"),
    ],
    ids=["comment", "anchor", "tag-anchor", "template", "custom-template", "skip"],
)
def test_cli_compact_keeps_multiline_quote_boundaries(
    tmp_path: Path,
    quote: str,
    newline: str,
    prefix: str,
    value: str,
    suffix: str,
    preserved: str,
) -> None:
    source = (prefix + quote + value + quote + suffix).replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--compact", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.replace("\n", newline).encode() in output
    assert read_yaml(path, multi=True) == before
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
