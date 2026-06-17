"""Run via `uv run external-tests/run.py --suite cli/test_yaml_semantics.py`."""

from __future__ import annotations

from pathlib import Path

from yaml12 import read_yaml

from _support import run_command


LOCAL_YAML_CASES: list[tuple[str, str]] = [
    (
        "flow-spacing",
        "items: [{a: b,c: d}, {a: e,c: f}]\n",
    ),
    (
        "skip-node",
        """normal: [1,2,3]
# fmt: skip
manual:
    -   [ 1,2,3]
    -   [ 4,5,6]
""",
    ),
    (
        "flow-table",
        """# fmt: table
- {name: a, type: int, default: 0}
- {name: long_name, type: string, default: ""}
""",
    ),
    ("crlf", "items: [a,b,c]\r\n"),
    (
        "literal-block",
        """script: |
  echo "do not fold this"
  echo "hard line breaks matter"
""",
    ),
    (
        "quoted-numeric-strings",
        """version: '1.10'
scientific: "1e2"
hex: '0x2A'
octal: '0o52'
positive: '+42'
inf: '.Inf'
nan: '.NaN'
plain_string: 'release 1.10 candidate'
""",
    ),
    (
        "quoted-strings-needing-style",
        r"""boolish: "true"
nullish: "null"
empty: ""
leading: " space"
trailing: "space "
tabbed: "a\tb"
colon: "a: b"
comment: "a # b"
""",
    ),
    (
        "core-tags",
        """string: !!str hello world
string_bool: !!str true
int: !!int 3
bool: !!bool true
float: !!float 1.10
null: !!null null
""",
    ),
    (
        "anchored-tagged-flow",
        """items: &items [a,b]
tagged: !seq [c,d]
both: !seq &both [e,f]
ref: *items
""",
    ),
    (
        "multiline-plain-scalar",
        """description: first line
  second line
items: [a,b]
""",
    ),
]


def test_cli_preserves_yaml12_values_for_local_yaml_regressions(
    tmp_path: Path,
) -> None:
    paths = []
    for idx, (_, input_text) in enumerate(LOCAL_YAML_CASES):
        path = tmp_path / f"{idx:03}-case.yaml"
        path.write_text(input_text, encoding="utf-8")
        paths.append(path)

    before = [read_yaml(path, multi=True) for path in paths]

    run_command("yamark", "format", *paths)

    after = [read_yaml(path, multi=True) for path in paths]

    for path, before_value, after_value in zip(paths, before, after):
        assert before_value == after_value, (
            f"YAML values are not identical for {path}"
        )

    run_command("yamark", "format", *paths)
    reformatted = [path.read_text(encoding="utf-8") for path in paths]
    run_command("yamark", "format", *paths)
    assert reformatted == [path.read_text(encoding="utf-8") for path in paths]
