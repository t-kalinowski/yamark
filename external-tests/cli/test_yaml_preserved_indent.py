"""Preserved scalar bytes constrain collection indentation through the CLI."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize("width", [2, 4])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize(
    ("opening", "preserved", "closing"),
    [
        (
            "lorem:\n",
            "    ipsum: {quote}Lorem ipsum\n      {{{{ dolor }}}} sit amet.{quote}\n",
            "    dolor: [lorem,ipsum]\nsit: [lorem,ipsum]\n",
        ),
        (
            "lorem:\n",
            "    - {quote}Lorem ipsum\n      {{{{ dolor }}}} sit amet.{quote}\n",
            "    - [lorem,ipsum]\nsit: [lorem,ipsum]\n",
        ),
        (
            "lorem:\n ipsum:\n",
            "  dolor: {quote}Lorem ipsum\n   {{{{ sit }}}} amet.{quote}\n",
            "  sit: [lorem,ipsum]\n amet: [lorem,ipsum]\ndolor: [lorem,ipsum]\n",
        ),
        (
            "lorem:\n",
            "    -   ipsum: {quote}Lorem ipsum\n          {{{{ dolor }}}} sit amet.{quote}\n",
            "        dolor: [lorem,ipsum]\n    - [lorem,ipsum]\nsit: [lorem,ipsum]\n",
        ),
        (
            "lorem:\n    -   ipsum: [lorem,ipsum]\n",
            "        dolor: {quote}Lorem ipsum\n          {{{{ sit }}}} amet.{quote}\n",
            "        sit: [lorem,ipsum]\n    - [lorem,ipsum]\nsit: [lorem,ipsum]\n",
        ),
        (
            "lorem:\n -\n  -\n",
            "   ipsum: {quote}Lorem ipsum\n    {{{{ dolor }}}} sit amet.{quote}\n",
            "   dolor: [lorem,ipsum]\n  - [lorem,ipsum]\n - [lorem,ipsum]\n",
        ),
    ],
    ids=[
        "mapping",
        "sequence",
        "ancestors",
        "inline-first",
        "inline-later",
        "nested-sequence",
    ],
)
def test_cli_keeps_preserved_scalars_aligned(
    tmp_path: Path,
    quote: str,
    width: int,
    newline: str,
    opening: str,
    preserved: str,
    closing: str,
) -> None:
    preserved = preserved.format(quote=quote)
    source = (opening + preserved + closing).replace("\n", newline).encode()
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source)
    before = read_yaml(path, multi=True)

    args = ("yamark", "format", "--indent-width", str(width), "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.replace("\n", newline).encode() in output
    assert b"[lorem, ipsum]" in output
    assert read_yaml(path, multi=True) == before
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("preservation", ["single-line", "custom", "skip"])
def test_cli_preserved_indent_obeys_template_and_skip_policies(
    tmp_path: Path,
    preservation: str,
) -> None:
    prefix = ""
    if preservation == "single-line":
        preserved = "    ipsum: '{{ dolor }} sit amet.'\n"
    elif preservation == "custom":
        prefix = '# fmt: template.delimiters "<<" ">>" scope=file\n'
        preserved = "    ipsum: 'Lorem ipsum\n      << dolor >> sit amet.'\n"
    else:
        preserved = "    # fmt: skip\n    ipsum: 'Lorem ipsum\n      dolor sit amet.'\n"
    source = (
        prefix + "lorem:\n" + preserved + "    dolor: [lorem,ipsum]\n"
        "    sit: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod.\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path, multi=True)
    args = ("yamark", "format", "--prose-width", "36", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.encode() in output
    assert b"    dolor: [lorem, ipsum]" in output
    assert b"    sit: >-" in output
    assert read_yaml(path, multi=True) == before
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
