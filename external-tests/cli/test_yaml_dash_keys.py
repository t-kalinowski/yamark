"""Dash-prefixed keys remain mappings across scalar continuation layouts."""

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("key", ["-1", "-lorem", "--lorem"])
@pytest.mark.parametrize("shape", ["root", "mapping", "sequence"])
@pytest.mark.parametrize("layout", ["below", "continuation", "properties"])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_retains_dash_prefixed_mapping_keys(
    tmp_path: Path, key: str, shape: str, layout: str, newline: str
) -> None:
    if layout == "below":
        entry = f"{key}:\n  lorem ipsum\n"
        alias = ""
    elif layout == "continuation":
        entry = f"{key}: Lorem ipsum dolor\n  sit amet, consectetur adipiscing elit.\n"
        alias = ""
    else:
        entry = f"{key}: !!str &ipsum # Lorem ipsum\n  lorem ipsum\n"
        alias = "ipsum: *ipsum\n"
    content = entry + alias + "-dolor: [lorem,ipsum]\n"
    if shape == "root":
        source = content
    elif shape == "mapping":
        source = "lorem:\n" + "".join("  " + line for line in content.splitlines(True))
    else:
        lines = content.splitlines(True)
        source = "- " + lines[0] + "".join("  " + line for line in lines[1:])
        source += "- [dolor,sit]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.replace("\n", newline).encode())
    before = read_yaml(path)

    args = ("yamark", "format", "--prose-width", "12", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after == before
    if shape == "sequence":
        mapping = after[0]
    elif shape == "mapping":
        mapping = after["lorem"]
    else:
        mapping = after
    assert isinstance(mapping, dict)
    assert (-1 if key == "-1" else key) in mapping
    assert b"-dolor: [lorem, ipsum]" in output
    if layout == "properties":
        for token in [b"!!str", b"&ipsum", b"*ipsum", b"# Lorem ipsum"]:
            assert output.count(token) == 1
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    "source",
    [
        "-lorem ipsum\n  dolor sit amet.\n",
        "- lorem ipsum\n  dolor sit amet.\n- -lorem ipsum\n",
        "-\n  -lorem ipsum\n    dolor sit amet.\n- [lorem,ipsum]\n",
    ],
    ids=["root-scalar", "inline-sequence", "out-of-line-sequence"],
)
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_distinguishes_dash_text_from_sequence_markers(
    tmp_path: Path, source: str, newline: str
) -> None:
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.replace("\n", newline).encode())
    before = read_yaml(path)
    args = ("yamark", "format", "--prose-width", "12", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after == before
    assert type(after) is type(before)
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
