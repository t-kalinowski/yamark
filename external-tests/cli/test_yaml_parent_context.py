"""Compact entries and Markdown scalars retain their parent context."""

import json
import os
import sys
from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize(
    ("indent", "width", "newline", "quote"),
    [(2, 2, "\n", "'"), (4, 2, "\r\n", '"'), (6, 4, "\r", "'")],
    ids=["two-lf-single", "four-crlf-double", "six-cr-single"],
)
@pytest.mark.parametrize("properties", ["", "!!str &ipsum "])
def test_cli_preserves_compact_nested_sequence_entries(
    tmp_path: Path,
    indent: int,
    width: int,
    newline: str,
    quote: str,
    properties: str,
) -> None:
    preserved = (
        f"-{' ' * (indent - 1)}- {properties}# Lorem ipsum\n"
        f"{' ' * (indent + 2)}{quote}Lorem ipsum\n"
        f"{' ' * (indent + 4)}dolor sit amet.{quote}\n"
    )
    alias = f"{' ' * indent}- *ipsum\n" if properties else ""
    sibling = f"{' ' * indent}- [lorem,ipsum]\n"
    source = preserved + alias + sibling + "- [dolor,sit]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.replace("\n", newline).encode())
    before = read_yaml(path)

    args = ("yamark", "format", "--indent-width", str(width), "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert output.startswith(preserved.replace("\n", newline).encode())
    assert b"[lorem, ipsum]" in output
    assert b"[dolor, sit]" in output
    after = read_yaml(path)
    assert after == before
    assert isinstance(after[0], list)
    if properties:
        assert after[0][0] == after[0][1]
        for token in [b"!!str", b"&ipsum", b"*ipsum"]:
            assert output.count(token) == 1

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("indent", "width", "newline", "quote"),
    [(2, 2, "\n", "'"), (4, 2, "\r\n", '"'), (6, 4, "\r", "'")],
    ids=["two-lf-single", "four-crlf-double", "six-cr-single"],
)
@pytest.mark.parametrize("shape", ["mapping", "sequence", "explicit"])
def test_cli_retains_comments_around_parent_tagged_markdown(
    tmp_path: Path,
    indent: int,
    width: int,
    newline: str,
    quote: str,
    shape: str,
) -> None:
    if shape == "mapping":
        header = "lorem: !md # Lorem ipsum\n"
        sibling = "ipsum: [dolor,sit]\n"
    elif shape == "sequence":
        header = "- !md # Lorem ipsum\n"
        sibling = "- [dolor,sit]\n"
    else:
        header = "? lorem\n: !md # Lorem ipsum\n"
        sibling = "ipsum: [dolor,sit]\n"
    source = (
        header
        + f"{' ' * indent}# Dolor sit amet.\n"
        + f"{' ' * indent}{quote}Lorem __ipsum__\n"
        + f"{' ' * (indent + 2)}dolor sit amet.{quote}\n"
        + sibling
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    before_scalar = before[0] if shape == "sequence" else before["lorem"]
    assert before_scalar.tag == "!md"
    assert before_scalar.value == "Lorem __ipsum__ dolor sit amet."

    args = (
        "yamark",
        "format",
        "--indent-width",
        str(width),
        "--wrap",
        "none",
        "--canonical",
        "--verify",
        path,
    )
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    for comment in ["# Lorem ipsum", "# Dolor sit amet."]:
        assert output.count((comment + newline).encode()) == 1
    assert output.count(b"!md") == 1
    assert b"[dolor, sit]" in output
    after = read_yaml(path)
    after_scalar = after[0] if shape == "sequence" else after["lorem"]
    assert after_scalar.tag == before_scalar.tag
    assert after_scalar.value == "Lorem **ipsum** dolor sit amet.\n"
    if shape == "sequence":
        assert after[1:] == before[1:]
    else:
        assert after["ipsum"] == before["ipsum"]

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_aligns_nested_markdown_siblings_with_preserved_parent_comments(
    tmp_path: Path,
) -> None:
    preserved = "    ipsum: !md # Lorem ipsum\n      # Dolor sit amet.\n"
    source = (
        "lorem:\n"
        + preserved
        + "      'Lorem __ipsum__\n        dolor sit amet.'\n"
        + "    dolor: [lorem,ipsum]\nsit: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)

    args = (
        "yamark",
        "format",
        "--indent-width",
        "2",
        "--wrap",
        "none",
        "--canonical",
        "--verify",
        path,
    )
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.encode() in output
    assert output.count(b"# Lorem ipsum") == 1
    assert output.count(b"# Dolor sit amet.") == 1
    assert b"[lorem, ipsum]" in output
    assert b"[dolor, sit]" in output
    after = read_yaml(path)
    assert after["lorem"]["ipsum"].tag == before["lorem"]["ipsum"].tag
    assert after["lorem"]["ipsum"].value == "Lorem **ipsum** dolor sit amet.\n"
    assert after["lorem"]["dolor"] == before["lorem"]["dolor"]
    assert after["sit"] == before["sit"]

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("prefix", "preserved", "suffix", "newline", "skip_embedded"),
    [
        (
            "",
            "- - - # Lorem ipsum\n      'Lorem ipsum\n        dolor sit amet.'\n",
            "    - [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\n",
            False,
        ),
        (
            "",
            "- - lorem: # Lorem ipsum\n      'Lorem ipsum\n        dolor sit amet.'\n",
            "    ipsum: [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\r\n",
            False,
        ),
        (
            "- - [lorem,ipsum]\n",
            "  - # Lorem ipsum\n    'Lorem ipsum\n      dolor sit amet.'\n",
            "  - [dolor,sit]\n- [amet,elit]\n",
            "\r",
            False,
        ),
        (
            "",
            "- - - 'Lorem ipsum\n        {{ dolor }} sit amet.'\n",
            "    - [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\r\n",
            False,
        ),
        (
            "",
            "- - lorem: 'Lorem ipsum\n        {{ dolor }} sit amet.'\n",
            "    ipsum: [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\r",
            False,
        ),
        (
            "- - [lorem,ipsum]\n",
            "  - 'Lorem ipsum\n      {{ dolor }} sit amet.'\n",
            "  - [dolor,sit]\n- [amet,elit]\n",
            "\n",
            False,
        ),
        (
            "- - [lorem,ipsum]\n",
            "  -\n    # fmt: skip\n    'Lorem ipsum\n      dolor sit amet.'\n",
            "  - [dolor,sit]\n- [amet,elit]\n",
            "\r\n",
            False,
        ),
        (
            "# fmt: embedded lorem\n",
            "- - - |\n        Lorem ipsum.\n",
            "    - [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\n",
            True,
        ),
        (
            "",
            "- - lorem: &ipsum # Lorem ipsum\n      'Lorem ipsum\n        dolor sit amet.'\n",
            "    ipsum: *ipsum\n    dolor: [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\r",
            False,
        ),
    ],
    ids=[
        "depth-three-comments",
        "mapping-first-child-comments",
        "later-child-comments",
        "depth-three-template",
        "mapping-first-child-template",
        "later-child-template",
        "later-child-skip",
        "depth-three-skipped-formatter",
        "mapping-first-child-anchor",
    ],
)
def test_cli_preserves_compact_ancestor_prefix_ownership(
    tmp_path: Path,
    prefix: str,
    preserved: str,
    suffix: str,
    newline: str,
    skip_embedded: bool,
) -> None:
    source = (prefix + preserved + suffix).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    args = ["yamark", "format", "--indent-width", "2", "--verify"]
    if skip_embedded:
        command = json.dumps(
            [
                sys.executable,
                "-c",
                "import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())",
                "{path}",
            ]
        )
        (tmp_path / "yamark.toml").write_bytes(
            (
                "[embedded.lorem]\n"
                f'formatter = {{ command = {command}, path_suffix = ".txt" }}\n'
            ).encode()
        )
        args.append("--skip-embedded-formatters")
    args.append(str(path))

    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert output.count(preserved.replace("\n", newline).encode()) == 1
    if skip_embedded:
        assert output.startswith((prefix + preserved).replace("\n", newline).encode())
    for editable in [b"[lorem, ipsum]", b"[dolor, sit]", b"[amet, elit]"]:
        assert editable in output
    assert read_yaml(path) == before

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_preserves_compact_prefix_when_optional_formatter_is_unavailable(
    tmp_path: Path,
) -> None:
    preserved = "# fmt: embedded lorem\n- - - |\n        Lorem ipsum.\n"
    source = preserved + "    - [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    (tmp_path / "yamark.toml").write_bytes(
        b"[embedded.lorem]\n"
        b'formatter = { command = ["ruff", "{path}"], path_suffix = ".txt" }\n'
    )
    env = {**os.environ, "PATH": ""}
    args = ("yamark", "format", "--indent-width", "2", "--verify", path)

    run_command(*args, cwd=tmp_path, env=env)
    output = path.read_bytes()
    assert output.startswith(preserved.encode())
    for editable in [b"[lorem, ipsum]", b"[dolor, sit]", b"[amet, elit]"]:
        assert editable in output
    assert read_yaml(path) == before

    run_command(*args, cwd=tmp_path, env=env)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("prefix", "indent", "suffix", "newline"),
    [
        (
            "- - !md # Lorem ipsum\n    # Dolor sit amet.\n",
            4,
            "  - [lorem,ipsum]\n- [dolor,sit]\n",
            "\n",
        ),
        (
            "- - lorem: !md # Lorem ipsum\n      # Dolor sit amet.\n",
            6,
            "    ipsum: [lorem,ipsum]\n  - [dolor,sit]\n- [amet,elit]\n",
            "\r\n",
        ),
    ],
    ids=["sequence", "mapping"],
)
def test_cli_keeps_compact_ancestors_around_markdown_parent_comments(
    tmp_path: Path, prefix: str, indent: int, suffix: str, newline: str
) -> None:
    scalar = f"{' ' * indent}'Lorem __ipsum__\n{' ' * (indent + 2)}dolor sit amet.'\n"
    source = (prefix + scalar + suffix).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    read_yaml(path)
    canonical = f'{" " * indent}"Lorem **ipsum** dolor sit amet.\\n"\n'
    expected_path = tmp_path / "ipsum.yaml"
    expected_path.write_bytes(
        (prefix + canonical + suffix).replace("\n", newline).encode()
    )
    expected = read_yaml(expected_path)

    args = (
        "yamark",
        "format",
        "--indent-width",
        "2",
        "--wrap",
        "none",
        "--canonical",
        "--verify",
        path,
    )
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert output.startswith(prefix.replace("\n", newline).encode())
    assert output.count(b"# Lorem ipsum") == 1
    assert output.count(b"# Dolor sit amet.") == 1
    assert b"[lorem, ipsum]" in output
    assert b"[dolor, sit]" in output
    assert read_yaml(path) == expected

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
