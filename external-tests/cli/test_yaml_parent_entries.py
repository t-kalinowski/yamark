"""Preserved out-of-line scalars retain their complete parent entry."""

import json
import sys
from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize("width", [2, 4])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize("shape", ["mapping", "sequence", "explicit"])
@pytest.mark.parametrize("boundary", ["template", "parent", "child", "both", "skip"])
def test_cli_retains_complete_scalar_entries(
    tmp_path: Path,
    quote: str,
    width: int,
    newline: str,
    shape: str,
    boundary: str,
) -> None:
    if shape == "mapping":
        header = "    ipsum:"
        sibling = "    dolor: [lorem,ipsum]\n"
    elif shape == "sequence":
        header = "    -"
        sibling = "    - [lorem,ipsum]\n"
    else:
        header = "    ? ipsum\n    :"
        sibling = "    dolor: [lorem,ipsum]\n"
    parent_comment = " # Lorem ipsum" if boundary in {"parent", "both"} else ""
    child_comment = "      # Dolor sit amet.\n" if boundary in {"child", "both"} else ""
    directive = "      # fmt: skip\n" if boundary == "skip" else ""
    content = "{{ dolor }} sit amet." if boundary == "template" else "dolor sit amet."
    preserved = (
        f"{header}{parent_comment}\n"
        f"{directive}{child_comment}      {quote}Lorem ipsum\n"
        f"        {content}{quote}\n"
    )
    source = "lorem:\n" + preserved + sibling + "sit: [lorem,ipsum]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.replace("\n", newline).encode())
    before = read_yaml(path)

    args = ("yamark", "format", "--indent-width", str(width), "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.replace("\n", newline).encode() in output
    assert sibling.replace(",", ", ").replace("\n", newline).encode() in output
    assert read_yaml(path) == before
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("skip", [False, True], ids=["formatted", "preserved"])
def test_cli_keeps_parent_properties_with_embedded_scalars(
    tmp_path: Path, skip: bool
) -> None:
    formatter = tmp_path / "lorem.py"
    formatter.write_bytes(b"import sys\nsys.stdout.buffer.write(sys.stdin.buffer.read())\n")
    command = json.dumps([sys.executable, str(formatter), "{path}"])
    (tmp_path / "yamark.toml").write_bytes(
        f'[embedded.lorem]\nformatter = {{ command = {command}, path_suffix = ".txt" }}\n'.encode()
    )
    preserved = (
        "# fmt: embedded lorem\n"
        "lorem: &ipsum\n"
        "  !dolor |\n"
        "    Lorem ipsum.\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes((preserved + "ipsum: *ipsum\ndolor: [lorem,ipsum]\n").encode())
    before = read_yaml(path)
    args = ["yamark", "format", "--verify"]
    if skip:
        args.append("--skip-embedded-formatters")
    args.append(str(path))
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.count(b"&ipsum") == 1
    assert output.count(b"!dolor") == 1
    assert b"[lorem, ipsum]" in output
    if skip:
        assert preserved.encode() in output
    else:
        assert b"lorem: &ipsum !dolor |\n" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
