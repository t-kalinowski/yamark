"""Block scalar indentation distinguishes content from comments and directives."""

import json
import sys
from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("style", ["|", ">"], ids=["literal", "folded"])
@pytest.mark.parametrize("explicit", [False, True], ids=["implicit", "explicit"])
@pytest.mark.parametrize("shape", ["root", "mapping", "sequence"])
def test_cli_distinguishes_block_content_from_dedented_comments(
    tmp_path: Path, style: str, explicit: bool, shape: str
) -> None:
    if shape == "root":
        prefix, suffix, indent, comment_indent, newline = (
            "",
            "---\nipsum: [lorem,ipsum]\n",
            2,
            1,
            "\n",
        )
    elif shape == "mapping":
        prefix, suffix, indent, comment_indent, newline = (
            "lorem:\n  ",
            "ipsum: [lorem,ipsum]\n",
            4,
            2,
            "\r\n",
        )
    else:
        prefix, suffix, indent, comment_indent, newline = (
            "-\n  ",
            "- [lorem,ipsum]\n",
            4,
            2,
            "\r",
        )
    indicator = str(indent) if explicit else ""
    source = (
        f"{prefix}{style}{indicator}-\n"
        f"{' ' * indent}Lorem ipsum.\n"
        f"{' ' * indent}# Dolor sit amet.\n\n"
        f"{' ' * comment_indent}# Consectetur adipiscing elit.\n\n"
        f"{suffix}"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path, multi=True)
    scalar = before[0]
    if shape == "mapping":
        scalar = scalar["lorem"]
    elif shape == "sequence":
        scalar = scalar[0]
    assert "# Dolor sit amet." in scalar
    assert "# Consectetur adipiscing elit." not in scalar

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert output.count(b"# Dolor sit amet.") == 1
    assert output.count(b"# Consectetur adipiscing elit.") == 1
    assert b"[lorem, ipsum]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("dedented", [False, True], ids=["content", "directive"])
def test_cli_applies_file_skip_only_outside_block_content(
    tmp_path: Path, dedented: bool
) -> None:
    boundary = (
        "  # fmt: skip file\n"
        if dedented
        else "    # fmt: skip file\n  # Dolor sit amet.\n"
    )
    newline = "\r\n" if dedented else "\r"
    source = (
        "ipsum: [lorem,ipsum]\nlorem:\n  |\n    Lorem ipsum.\n"
        + boundary
        + "dolor: [dolor,sit]\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert ("# fmt: skip file" in before["lorem"]) is not dedented

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.count(b"# fmt: skip file") == 1
    if dedented:
        assert output == source.encode()
    else:
        assert b"[lorem, ipsum]" in output
        assert b"[dolor, sit]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("scalar", "newline"),
    [
        ("lorem:\n  |\n # fmt: skip file\n Lorem ipsum.\n", "\n"),
        ("lorem:\n  |\n  # fmt: skip file\n  Lorem ipsum.\n", "\r"),
        ("lorem:\n  |2\n    Lorem ipsum.\n  # fmt: skip file\n", "\r\n"),
        ("lorem:\n  |\n  \tLorem ipsum.\n  # fmt: skip file\n", "\n"),
        ("lorem:\n  |2\n  \tLorem ipsum.\n  # fmt: skip file\n", "\r\n"),
    ],
    ids=[
        "implicit-first-comment-one",
        "implicit-first-comment-two",
        "explicit-two",
        "implicit-first-tab",
        "explicit-first-tab",
    ],
)
def test_cli_uses_content_indentation_instead_of_header_column(
    tmp_path: Path, scalar: str, newline: str
) -> None:
    source = (scalar + "ipsum: [lorem,ipsum]\n").replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert "# fmt: skip file" in before["lorem"]
    if "\t" in scalar:
        assert "\t" in before["lorem"]

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.count(b"# fmt: skip file") == 1
    assert b"[lorem, ipsum]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_applies_next_node_skip_after_block_content(tmp_path: Path) -> None:
    source = (
        "lorem:\n  |\n    Lorem ipsum.\n"
        "  # fmt: skip scope=next\n"
        "ipsum: [lorem,ipsum]\n"
        "dolor: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert "# fmt:" not in before["lorem"]

    args = ("yamark", "format", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    assert read_yaml(path) == before
    assert output.count(b"# fmt: skip scope=next") == 1
    assert b"ipsum: [lorem,ipsum]\n" in output
    assert b"dolor: [dolor, sit]\n" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("style", ["|", ">"], ids=["literal", "folded"])
def test_cli_applies_late_formatting_directives_after_block_content(
    tmp_path: Path, style: str
) -> None:
    newline = "\n" if style == "|" else "\r\n"
    source = (
        "ipsum: !md 'Lorem __ipsum__.'\n"
        f"lorem:\n  {style}\n    Dolor sit amet.\n"
        "  # fmt: canonical=true scope=file\n"
        "dolor: [lorem,ipsum]\n"
    ).replace("\n", newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert before["ipsum"].value == "Lorem __ipsum__."
    assert "# fmt:" not in before["lorem"]

    args = ("yamark", "format", "--wrap", "none", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after["ipsum"].tag == before["ipsum"].tag
    assert after["ipsum"].value == "Lorem **ipsum**.\n"
    assert after["lorem"] == before["lorem"]
    assert after["dolor"] == before["dolor"]
    assert output.count(b"# fmt: canonical=true scope=file") == 1
    assert b"[lorem, ipsum]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


@pytest.mark.parametrize("explicit", [False, True], ids=["implicit", "explicit-four"])
@pytest.mark.parametrize("shape", ["root", "mapping", "nested-mapping"])
def test_cli_keeps_dedented_comments_outside_formatted_markdown_scalars(
    tmp_path: Path, explicit: bool, shape: str
) -> None:
    indicator = "4" if explicit else ""
    if shape == "root":
        source = (
            f"!md |{indicator}\n    Lorem __ipsum__.\n"
            "  # Dolor sit amet.\n---\nipsum: [dolor,sit]\n"
        )
        newline = "\n"
    elif shape == "mapping":
        source = (
            f"lorem: !md\n  |{indicator}\n    Lorem __ipsum__.\n"
            "  # Dolor sit amet.\nipsum: [dolor,sit]\n"
        )
        newline = "\r\n"
    else:
        source = (
            f"lorem:\n    ipsum: !md\n      |{indicator}\n"
            "        Lorem __ipsum__.\n      # Dolor sit amet.\n"
            "    dolor: [lorem,ipsum]\nsit: [dolor,sit]\n"
        )
        newline = "\r"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.replace("\n", newline).encode())
    before = read_yaml(path, multi=True)
    before_scalar = before[0]
    if shape == "mapping":
        before_scalar = before_scalar["lorem"]
    elif shape == "nested-mapping":
        before_scalar = before_scalar["lorem"]["ipsum"]
    assert before_scalar.tag == "!md"
    assert before_scalar.value == "Lorem __ipsum__.\n"

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
    after = read_yaml(path, multi=True)
    after_scalar = after[0]
    if shape == "root":
        assert after[1:] == before[1:]
    elif shape == "mapping":
        after_scalar = after_scalar["lorem"]
        assert after[0]["ipsum"] == before[0]["ipsum"]
    else:
        after_scalar = after_scalar["lorem"]["ipsum"]
        assert after[0]["lorem"]["dolor"] == before[0]["lorem"]["dolor"]
        assert after[0]["sit"] == before[0]["sit"]
        assert b"[lorem, ipsum]" in output
    assert after_scalar.tag == before_scalar.tag
    assert after_scalar.value == "Lorem **ipsum**.\n"
    assert output.count(b"# Dolor sit amet.") == 1
    assert b"[dolor, sit]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def test_cli_keeps_dedented_comment_outside_external_formatter_output(
    tmp_path: Path,
) -> None:
    formatter = tmp_path / "lorem.py"
    formatter.write_bytes(
        b"import sys\n"
        b"sys.stdout.buffer.write(sys.stdin.buffer.read().replace("
        b"b'Lorem  ipsum.', b'Lorem ipsum.'))\n"
    )
    command = json.dumps([sys.executable, str(formatter), "{path}"])
    (tmp_path / "yamark.toml").write_bytes(
        (
            "[embedded.lorem]\n"
            f'formatter = {{ command = {command}, path_suffix = ".txt" }}\n'
        ).encode()
    )
    source = (
        "lorem:\n    # fmt: embedded lorem\n    ipsum:\n      |4\n"
        "        Lorem  ipsum.\n      # Dolor sit amet.\n"
        "    dolor: [lorem,ipsum]\nsit: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert before["lorem"]["ipsum"] == "Lorem  ipsum.\n"

    args = ("yamark", "format", "--indent-width", "2", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after["lorem"]["ipsum"] == "Lorem ipsum.\n"
    assert after["lorem"]["dolor"] == before["lorem"]["dolor"]
    assert after["sit"] == before["sit"]
    assert output.count(b"# Dolor sit amet.") == 1
    assert b"[lorem, ipsum]" in output
    assert b"[dolor, sit]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output


def configure_empty_formatter(tmp_path: Path, output: bytes = b"") -> None:
    formatter = tmp_path / "lorem.py"
    formatter.write_bytes(
        (
            "import sys\nsys.stdin.buffer.read()\n"
            f"sys.stdout.buffer.write({output!r})\n"
        ).encode()
    )
    command = json.dumps([sys.executable, str(formatter), "{path}"])
    (tmp_path / "yamark.toml").write_bytes(
        (
            "[embedded.lorem]\n"
            f'formatter = {{ command = {command}, path_suffix = ".txt" }}\n'
        ).encode()
    )


@pytest.mark.parametrize("explicit", [False, True], ids=["implicit", "explicit-four"])
@pytest.mark.parametrize("parent_indent", [0, 4], ids=["mapping", "nested-mapping"])
def test_cli_rejects_empty_external_output_before_indented_comments(
    tmp_path: Path, explicit: bool, parent_indent: int
) -> None:
    configure_empty_formatter(tmp_path)
    indicator = "4" if explicit else ""
    comment = f"{' ' * (parent_indent + 2)}# Dolor sit amet.\n"
    if parent_indent == 0:
        prefix = f"# fmt: embedded lorem\nlorem:\n  |{indicator}\n"
        suffix = "ipsum: [lorem,ipsum]\ndolor: [dolor,sit]\n"
    else:
        prefix = f"lorem:\n    # fmt: embedded lorem\n    ipsum:\n      |{indicator}\n"
        suffix = "    dolor: [lorem,ipsum]\nsit: [dolor,sit]\n"
    source = prefix + f"{' ' * (parent_indent + 4)}Lorem ipsum.\n" + comment + suffix
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    before_scalar = before["lorem"] if parent_indent == 0 else before["lorem"]["ipsum"]
    assert before_scalar == "Lorem ipsum.\n"

    location = "3:3" if parent_indent == 0 else "4:7"
    run_command(
        "yamark",
        "format",
        "--indent-width",
        "2",
        "--verify",
        path,
        cwd=tmp_path,
        status=1,
        stdout="1 files scanned, 0 formatted, 0 unchanged, 0 skipped, 1 failed\n",
        stderr=(
            f"{path}:{location}: error: blank embedded output would absorb "
            "a following YAML comment\n"
        ),
    )
    assert path.read_bytes() == source.encode()


def test_cli_rejects_empty_formatter_output_before_deeply_indented_comment(
    tmp_path: Path,
) -> None:
    configure_empty_formatter(tmp_path)
    source = (
        "# fmt: embedded lorem\nlorem:\n  |\n"
        "            Lorem ipsum.\n          # Dolor sit amet.\n"
        "ipsum: [lorem,ipsum]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    assert read_yaml(path)["lorem"] == "Lorem ipsum.\n"

    run_command(
        "yamark",
        "format",
        "--indent-width",
        "2",
        "--verify",
        path,
        cwd=tmp_path,
        status=1,
        stdout="1 files scanned, 0 formatted, 0 unchanged, 0 skipped, 1 failed\n",
        stderr=(
            f"{path}:3:3: error: blank embedded output would absorb "
            "a following YAML comment\n"
        ),
    )
    assert path.read_bytes() == source.encode()


@pytest.mark.parametrize(
    "formatter_output", [b"\n", b"   \n"], ids=["newline", "spaces"]
)
def test_cli_rejects_whitespace_formatter_output_before_indented_comment(
    tmp_path: Path, formatter_output: bytes
) -> None:
    configure_empty_formatter(tmp_path, formatter_output)
    source = (
        "# fmt: embedded lorem\nlorem:\n  |\n    Lorem ipsum.\n"
        "  # Dolor sit amet.\nipsum: [lorem,ipsum]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    assert read_yaml(path)["lorem"] == "Lorem ipsum.\n"

    run_command(
        "yamark",
        "format",
        "--indent-width",
        "2",
        "--verify",
        path,
        cwd=tmp_path,
        status=1,
        stdout="1 files scanned, 0 formatted, 0 unchanged, 0 skipped, 1 failed\n",
        stderr=(
            f"{path}:3:3: error: blank embedded output would absorb "
            "a following YAML comment\n"
        ),
    )
    assert path.read_bytes() == source.encode()


def test_cli_allows_empty_formatter_output_before_comment_at_parent_indent(
    tmp_path: Path,
) -> None:
    configure_empty_formatter(tmp_path)
    source = (
        "lorem:\n    # fmt: embedded lorem\n    ipsum:\n      |\n"
        "        Lorem ipsum.\n    # Dolor sit amet.\n"
        "    dolor: [lorem,ipsum]\nsit: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode())
    before = read_yaml(path)
    assert before["lorem"]["ipsum"] == "Lorem ipsum.\n"

    args = ("yamark", "format", "--indent-width", "2", "--verify", path)
    run_command(*args, cwd=tmp_path)
    output = path.read_bytes()
    after = read_yaml(path)
    assert after["lorem"]["ipsum"] == ""
    assert after["lorem"]["dolor"] == before["lorem"]["dolor"]
    assert after["sit"] == before["sit"]
    assert output.count(b"# Dolor sit amet.") == 1
    assert b"[lorem, ipsum]" in output
    assert b"[dolor, sit]" in output

    run_command(*args, cwd=tmp_path)
    assert path.read_bytes() == output
