"""Multiline YAML prose reflows without changing parsed scalar values."""

from __future__ import annotations

from pathlib import Path

import pytest
from _support import run_command
from yaml12 import read_yaml


@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
@pytest.mark.parametrize(
    "final_newline", [True, False], ids=["final-newline", "no-final-newline"]
)
@pytest.mark.parametrize("quote", ["", "'", '"'], ids=["plain", "single", "double"])
@pytest.mark.parametrize(
    ("prefix", "indent", "suffix"),
    [
        ("", "  ", ""),
        ("lorem: ", "  ", "ipsum: [dolor,sit]\n"),
        ("- ", "  ", "- [dolor,sit]\n"),
        ("- lorem: ", "    ", "  ipsum: [dolor,sit]\n"),
    ],
    ids=["root", "mapping", "sequence", "sequence-mapping"],
)
def test_cli_reflows_multiline_yaml_prose(
    tmp_path: Path,
    newline: str,
    final_newline: bool,
    quote: str,
    prefix: str,
    indent: str,
    suffix: str,
) -> None:
    source = (
        f"{prefix}{quote}Lorem ipsum dolor sit amet,\n"
        f"{indent}consectetur adipiscing elit, sed do eiusmod tempor incididunt "
        f"ut labore et dolore magna aliqua.{quote}\n"
        f"{suffix}"
    ).replace("\n", newline)
    if not final_newline:
        source = source.removesuffix(newline)
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert any(line.endswith(b">-") for line in output.splitlines())
    assert all(len(line.lstrip()) <= 36 for line in output.splitlines())
    if suffix:
        assert b"[dolor, sit]" in output

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("value", "trailing_comment"),
    [
        ("Lorem ipsum dolor sit amet,\n\n  consectetur adipiscing elit.", ""),
        ("'Lorem ipsum dolor sit amet,\n\n  consectetur adipiscing elit.'", ""),
        ('"Lorem ipsum dolor sit amet,\n\n\n  consectetur adipiscing elit."', ""),
        ("Lorem  ipsum    dolor sit amet,\n  consectetur adipiscing elit.", ""),
        ("'Lorem  ipsum    dolor sit amet,\n  consectetur adipiscing elit.'", ""),
        ('"Lorem  ipsum    dolor sit amet,\n  consectetur adipiscing elit."', ""),
        ("'Lorem ipsum''dolor sit amet,\n  consectetur adipiscing elit.'", ""),
        (
            (
                '"Lorem ipsum \\\n  dolor sit amet,\\nconsectetur \\"adipiscing\\" elit, '
                '\\\\ sed do eiusmod tempor incididunt."'
            ),
            "",
        ),
        (
            "'Lorem ipsum dolor sit amet,\n  # consectetur adipiscing elit\n  sed do eiusmod.'",
            "",
        ),
        (
            "Lorem ipsum dolor sit amet,\n  consectetur adipiscing elit.",
            " # sed do eiusmod",
        ),
        (
            "'Lorem ipsum dolor sit amet,\n  consectetur adipiscing elit.'",
            " # sed do eiusmod",
        ),
        (
            '"Lorem ipsum dolor sit amet,\n  consectetur adipiscing elit."',
            " # sed do eiusmod",
        ),
        ("'Lorem ipsum \t \n  \t dolor sit amet, consectetur adipiscing elit.'", ""),
        ('"Lorem ipsum \t \n  \t dolor sit amet, consectetur adipiscing elit."', ""),
        ("' Lorem ipsum dolor sit amet,\n  consectetur adipiscing elit. '", ""),
        ('" Lorem ipsum dolor sit amet,\n  consectetur adipiscing elit. "', ""),
        ('"Lorem ipsum\\x20\n  \\x20dolor sit amet, consectetur adipiscing elit."', ""),
        ('"Lorem ipsum\\\n\n  dolor sit amet, consectetur adipiscing elit."', ""),
        ('"Lorem ipsum\\\n\n\n  dolor sit amet, consectetur adipiscing elit."', ""),
    ],
    ids=[
        "plain-blank-line",
        "single-blank-line",
        "double-blank-lines",
        "plain-spaces",
        "single-spaces",
        "double-spaces",
        "single-escape",
        "double-escapes",
        "quoted-comment-content",
        "plain-trailing-comment",
        "single-trailing-comment",
        "double-trailing-comment",
        "single-break-whitespace",
        "double-break-whitespace",
        "single-boundary-spaces",
        "double-boundary-spaces",
        "escaped-spaces-around-break",
        "escaped-continuation-blank-line",
        "escaped-continuation-blank-lines",
    ],
)
def test_cli_preserves_multiline_yaml_scalar_boundaries(
    tmp_path: Path, value: str, trailing_comment: str
) -> None:
    source = (
        f"# lorem ipsum\nlorem: {value}{trailing_comment}\n"
        "# dolor sit amet\nipsum: [dolor,sit]\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert output.count(b"# lorem ipsum") == 1
    assert output.count(b"# dolor sit amet") == 1
    if trailing_comment:
        assert output.count(trailing_comment.encode("utf-8")) == 1

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert path.read_bytes() == output


@pytest.mark.parametrize("quote", ["", "'", '"'], ids=["plain", "single", "double"])
def test_cli_preserves_tagged_anchored_multiline_yaml_values(
    tmp_path: Path, quote: str
) -> None:
    source = (
        f"lorem: !!str &ipsum {quote}Lorem ipsum dolor sit amet,\n"
        f"  consectetur adipiscing elit, sed do eiusmod tempor incididunt.{quote}\n"
        "dolor: *ipsum\n"
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    for token in [b"!!str", b"&ipsum", b"*ipsum"]:
        assert token in output

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert path.read_bytes() == output


@pytest.mark.parametrize("directive", ["literal", "next", "file"])
def test_cli_distinguishes_multiline_yaml_content_from_skip_directives(
    tmp_path: Path, directive: str
) -> None:
    literal = "  # fmt: skip\n" if directive == "literal" else ""
    scalar = (
        "lorem: 'Lorem ipsum dolor sit amet,\n"
        f"{literal}"
        "    consectetur adipiscing elit, sed do eiusmod tempor incididunt.'\n"
    )
    prefix = "# fmt: skip\n" if directive == "next" else ""
    suffix = "# fmt: skip scope=file\n" if directive == "file" else ""
    source = f"{prefix}{scalar}ipsum: [dolor,sit]\n{suffix}"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    if directive == "file":
        assert output == source.encode("utf-8")
    else:
        assert b"ipsum: [dolor, sit]" in output
        if directive == "next":
            assert output.startswith(f"{prefix}{scalar}".encode())

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert path.read_bytes() == output


@pytest.mark.parametrize(
    ("opening", "closing"),
    [(b"---\n", b"---\n"), (b"```yaml\n", b"```\n")],
    ids=["frontmatter", "fence"],
)
def test_cli_reflows_multiline_yaml_inside_markdown(
    tmp_path: Path, opening: bytes, closing: bytes
) -> None:
    source = (
        b'lorem: "Lorem ipsum dolor sit amet,\n'
        b'  consectetur adipiscing elit, sed do eiusmod tempor incididunt."\n'
        b"ipsum: [dolor,sit]\n"
    )
    yaml_path = tmp_path / "lorem.yaml"
    yaml_path.write_bytes(source)
    before = read_yaml(yaml_path, multi=True)
    path = tmp_path / "ipsum.md"
    path.write_bytes(opening + source + closing)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert output.startswith(opening)
    assert output.endswith(closing)
    formatted_yaml = output[len(opening) : -len(closing)]
    yaml_path.write_bytes(formatted_yaml)
    assert read_yaml(yaml_path, multi=True) == before
    assert b">-" in formatted_yaml
    assert b"ipsum: [dolor, sit]" in formatted_yaml

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert path.read_bytes() == output


@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize(
    "value",
    [
        " Lorem ipsum\n\n  dolor sit amet, consectetur adipiscing elit.",
        "Lorem ipsum\n\n  dolor sit amet, consectetur adipiscing elit. ",
        " Lorem ipsum\n\n\n  dolor sit amet, consectetur adipiscing elit. ",
    ],
    ids=["leading-space", "trailing-space", "boundary-spaces"],
)
def test_cli_preserves_root_multiline_yaml_trivia(
    tmp_path: Path, quote: str, value: str
) -> None:
    prefix = "---\n# lorem ipsum\n"
    source = f"{prefix}{quote}{value}{quote}\n"
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )

    output = path.read_bytes()
    assert read_yaml(path, multi=True) == before
    assert output.startswith(prefix.encode("utf-8"))
    assert any(line.startswith(b"|") for line in output.splitlines())

    run_command(
        "yamark", "format", "--prose-width", "36", "--verify", path, cwd=tmp_path
    )
    assert read_yaml(path, multi=True) == before
    assert path.read_bytes() == output


@pytest.mark.parametrize("quote", ["'", '"'], ids=["single", "double"])
@pytest.mark.parametrize("source_indent", [0, 2, 4])
@pytest.mark.parametrize("indent_width", [2, 4])
@pytest.mark.parametrize("newline", ["\n", "\r\n", "\r"], ids=["lf", "crlf", "cr"])
def test_cli_indents_root_literals_from_the_emitted_header(
    tmp_path: Path, quote: str, source_indent: int, indent_width: int, newline: str
) -> None:
    prefix = "---\n# Lorem ipsum\n"
    source = (
        f"{prefix}{' ' * source_indent}{quote} Lorem ipsum\n\n"
        f"{' ' * (source_indent + 2)}dolor sit amet {quote}\n"
    ).replace("\n", newline)
    expected = (
        (
            f"{prefix}|{indent_width}-\n"
            f"{' ' * (indent_width + 1)}Lorem ipsum\n"
            f"{' ' * indent_width}dolor sit amet \n"
        )
        .replace("\n", newline)
        .encode("utf-8")
    )
    path = tmp_path / "lorem.yaml"
    path.write_bytes(source.encode("utf-8"))
    before = read_yaml(path, multi=True)
    assert before == [" Lorem ipsum\ndolor sit amet "]

    run_command(
        "yamark",
        "format",
        "--indent-width",
        str(indent_width),
        "--verify",
        path,
        cwd=tmp_path,
    )

    assert path.read_bytes() == expected
    assert read_yaml(path, multi=True) == before

    run_command(
        "yamark",
        "format",
        "--indent-width",
        str(indent_width),
        "--verify",
        path,
        cwd=tmp_path,
    )
    assert path.read_bytes() == expected
    assert read_yaml(path, multi=True) == before
