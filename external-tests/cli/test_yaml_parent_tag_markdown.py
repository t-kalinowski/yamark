"""Parent tags prepare block Markdown before the first formatting pass."""

import json
import os
import sys
from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("tag", ["!md", "!markdown"])
@pytest.mark.parametrize("header", ["|", ">-"])
@pytest.mark.parametrize("sequence", [False, True], ids=["mapping", "sequence"])
@pytest.mark.parametrize(
    "split_anchor", [False, True], ids=["parent-anchor", "child-anchor"]
)
@pytest.mark.parametrize("comments", [False, True], ids=["plain", "comments"])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_formats_parent_tagged_markdown_blocks(
    tmp_path: Path,
    tag: str,
    header: str,
    sequence: bool,
    split_anchor: bool,
    comments: bool,
    newline: str,
) -> None:
    prefix = "- " if sequence else "lorem: "
    parent_anchor = "" if split_anchor else " &ipsum"
    child_anchor = "&ipsum " if split_anchor else ""
    parent_comment = " # Lorem ipsum" if comments else ""
    leading_comment = "  # Dolor sit amet.\n" if comments else ""
    child_comment = " # Consectetur adipiscing elit." if comments else ""
    suffix = (
        "- *ipsum\n- [dolor,sit]\n"
        if sequence
        else "ipsum: *ipsum\ndolor: [dolor,sit]\n"
    )
    source = (
        f"{prefix}{tag}{parent_anchor}{parent_comment}\n"
        f"{leading_comment}  {child_anchor}{header}{child_comment}\n"
        "    Lorem __ipsum__ dolor sit amet.\n"
        f"{suffix}"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    before_value = before[0] if sequence else before["lorem"]
    assert before_value.tag == tag
    ending = "\n" if header == "|" else ""
    assert before_value.value == f"Lorem __ipsum__ dolor sit amet.{ending}"

    args = ("yamark", "format", "--canonical", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    after_value = after[0] if sequence else after["lorem"]
    after_alias = after[1] if sequence else after["ipsum"]
    assert after_value.tag == tag
    assert after_value.value == f"Lorem **ipsum** dolor sit amet.{ending}"
    assert after_alias == after_value
    assert output.count(tag.encode()) == 1
    assert output.count(b"&ipsum") == 1
    assert output.splitlines().count(b"- *ipsum" if sequence else b"ipsum: *ipsum") == 1
    assert b"[dolor, sit]" in output
    if comments:
        for comment in [
            b"# Lorem ipsum",
            b"# Dolor sit amet.",
            b"# Consectetur adipiscing elit.",
        ]:
            assert output.count(comment) == 1
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_retains_markdown_code_indent_below_parent_tag(tmp_path: Path) -> None:
    source = (
        "lorem: !md\n  |\n"
        "    Lorem __ipsum__.\n\n"
        "        Dolor __sit__ amet.\n"
        "ipsum: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    args = ("yamark", "format", "--canonical", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    value = read_yaml(path)["lorem"]
    assert value.tag == "!md"
    assert value.value == "Lorem **ipsum**.\n\n    Dolor __sit__ amet.\n"
    assert b"[dolor, sit]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_preserves_skipped_parent_tagged_markdown(tmp_path: Path) -> None:
    preserved = "lorem: !md\n  # fmt: skip\n  |\n    Lorem __ipsum__ dolor sit amet.\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes((preserved + "ipsum: [dolor,sit]\n").encode())
    before = read_yaml(path)
    args = ("yamark", "format", "--canonical", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert preserved.encode() in output
    assert read_yaml(path) == before
    assert b"[dolor, sit]" in output
    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_reports_original_line_for_nested_optional_formatter(
    tmp_path: Path,
) -> None:
    source = "lorem: !md\n  |\n    ```python\n    Lorem ipsum.\n    ```\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    result = run_command(
        "yamark",
        "format",
        "--diagnostics",
        "--verify",
        path,
        cwd=tmp_path,
        env={**os.environ, "PATH": ""},
        stderr=None,
    )
    note = f"{path}:4:1: note: missing optional embedded formatter `ruff`; preserved source"
    assert note in result.stderr.decode().splitlines()
    assert read_yaml(path)["lorem"].value == "```python\nLorem ipsum.\n```\n"


@pytest.mark.parametrize("nested", [False, True], ids=["scalar", "recursive"])
def test_cli_retains_original_line_in_nested_formatter_path(
    tmp_path: Path, nested: bool
) -> None:
    capture = tmp_path / "ipsum.txt"
    formatter = tmp_path / "lorem.py"
    formatter.write_bytes(
        b"import sys\nfrom pathlib import Path\n"
        b"Path(sys.argv[1]).write_text(sys.argv[2], encoding='utf-8')\n"
        b"sys.stdout.buffer.write(sys.stdin.buffer.read())\n"
    )
    command = json.dumps([sys.executable, str(formatter), str(capture), "{path}"])
    (tmp_path / "yamark.toml").write_bytes(
        f'[embedded.lorem]\nformatter = {{ command = {command}, path_suffix = ".txt" }}\n'.encode()
    )
    source = (
        "lorem: !md\n  |\n    ````yaml\n    ipsum: !md\n      |\n"
        "        ```lorem\n        Lorem ipsum.\n        ```\n    ````\n"
        if nested
        else "lorem: !md\n  |\n    ```lorem\n    Lorem ipsum.\n    ```\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    run_command("yamark", "format", "--verify", path, cwd=tmp_path)
    assert (
        capture.read_text(encoding="utf-8")
        == f"{path}.embedded.{7 if nested else 4}.txt"
    )


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_composes_nested_markdown_error_positions(
    tmp_path: Path, newline: str
) -> None:
    source = (
        "lorem: !md\n  |\n    ```yaml\n    ipsum: !md\n      |\n"
        "        <!-- fmt: lorem -->\n    ```\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    run_command(
        "yamark",
        "format",
        path,
        cwd=tmp_path,
        status=1,
        stdout="1 files scanned, 0 formatted, 0 unchanged, 0 skipped, 1 failed\n",
        stderr=f"{path}:6:14: error: invalid fmt directive: lorem\n",
    )
