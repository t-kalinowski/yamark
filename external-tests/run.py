#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "py-yaml12>=0.1.0",
#   "pytest>=8.0.0",
#   "pytest-xdist>=3.0.0",
# ]
# ///
#
# Run the fast external test suite with (parallel by default):
#   uv run external-tests/run.py
#
# Run serially:
#   uv run external-tests/run.py --serial
#
# Run only specific suites (repeatable):
#   uv run external-tests/run.py --suite cli
#   uv run external-tests/run.py --suite smoke --suite snapshots
#   uv run external-tests/run.py --suite cli/test_semantic_*.py
#
# Optional flags:
#   --yamark-bin PATH   Use a specific yamark binary (defaults to target/debug/yamark).
#   --serial            Run tests in serial instead of default parallel execution.
#   --suite PATH         Include only these suite directories/files (repeatable).
#                        Can be nested paths under external-tests/, e.g. cli/test_help.py.
#                        Supports glob patterns, e.g. cli/test_semantic_*.py.
#   Extra pytest arguments are forwarded, for example: -k markdown or --maxfail=1.
#   Failure output uses full assertion diffs by default while successful output
#   remains quiet.
#
# Test files are discovered from this default:
#   external-tests/**/test_*.py

from __future__ import annotations

import argparse
import glob
import os
import subprocess
import sys
from pathlib import Path

import pytest


RUNNER_DESCRIPTION = """Run Yamark's external public CLI test suites.

Default fast path: discover external-tests/**/test_*.py and run them with pytest.
Public suite directories: cli, corpus, smoke, snapshots.
Use --suite PATH to select a directory, file, or glob under external-tests/.
Extra pytest arguments are forwarded after runner flags, for example -k markdown.
"""


def main(argv: list[str]) -> int:
    args, pytest_args = parse_args(argv)
    repo_root = Path(__file__).resolve().parents[1]
    external_tests_root = Path(__file__).resolve().parent

    suite_roots = discover_suite_roots(external_tests_root, args.suite)
    set_pythonpath(external_tests_root, extra_test_dirs=pythonpath_test_directories(suite_roots))

    if args.yamark_bin is None:
        build_debug_binary(repo_root)
        yamark_bin = repo_root / "target" / "debug" / "yamark"
    else:
        yamark_bin = args.yamark_bin

    yamark_bin = yamark_bin.resolve()
    assert yamark_bin.is_file(), f"yamark binary not found: {yamark_bin}"
    os.environ["YAMARK_BIN"] = os.fspath(yamark_bin)
    os.environ["YAMARK_ROOT"] = os.fspath(repo_root)

    markdown_ast_json_bin = build_markdown_ast_json_binary(repo_root).resolve()
    assert markdown_ast_json_bin.is_file(), (
        f"markdown-ast-json binary not found: {markdown_ast_json_bin}"
    )
    os.environ["MARKDOWN_AST_JSON_BIN"] = os.fspath(markdown_ast_json_bin)

    test_files = discover_smoke_tests(suite_roots)
    assert test_files, (
        "No smoke test files found. Expected Python files under external-tests/."
    )

    command = [
        "-q",
        "-o",
        "verbosity_assertions=2",
        *(os.fspath(test) for test in test_files),
        *pytest_args,
    ]
    if not args.serial:
        command = ["-n", "auto", *command]

    return pytest.main(command)


def parse_args(argv: list[str]) -> tuple[argparse.Namespace, list[str]]:
    parser = argparse.ArgumentParser(
        description=RUNNER_DESCRIPTION,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--yamark-bin",
        type=Path,
        default=None,
        help="path to the yamark binary under test (defaults to target/debug/yamark)",
    )
    parser.add_argument(
        "--serial",
        action="store_true",
        help="run these tests serially instead of default parallel execution",
    )
    parser.add_argument(
        "--suite",
        action="append",
        default=[],
        metavar="PATH",
        help=(
            "run tests from this suite directory or file under external-tests/ "
            "(repeatable). "
            "Examples: --suite cli, --suite snapshots, --suite cli/test_semantic_*.py"
        ),
    )
    return parser.parse_known_args(argv)


def discover_smoke_tests(suite_roots: list[Path]) -> list[Path]:
    discovered: list[Path] = []
    for suite_root in suite_roots:
        if suite_root.is_dir():
            discovered.extend(_discover_python_files(suite_root))
            continue
        test_file = _normalize_test_file(suite_root)
        discovered.append(test_file)
    return discovered


def discover_suite_roots(external_tests_root: Path, suites: list[str]) -> list[Path]:
    if not suites:
        return discover_default_test_files(external_tests_root)

    suite_roots = []
    for suite in suites:
        suite_roots.extend(_expand_suite_path(external_tests_root, suite))
    return suite_roots


def _expand_suite_path(external_tests_root: Path, suite: str) -> list[Path]:
    suite_path = Path(suite)
    if not suite_path.is_absolute():
        if suite_path.parts[:1] == ("external-tests",):
            suite_path = external_tests_root.parent / suite_path
        else:
            suite_path = external_tests_root / suite_path

    return _expand_path_pattern(suite_path)


def _expand_path_pattern(path: Path) -> list[Path]:
    if glob.has_magic(str(path)):
        matches = [_normalize_match(match) for match in sorted(glob_for_pattern(path))]
        assert matches, f"Suite pattern did not match any paths: {path}"
        return matches
    assert path.exists(), f"Suite path does not exist: {path}"
    return [path.resolve()]


def glob_for_pattern(path: Path) -> list[Path]:
    matches = glob.iglob(str(path), recursive=True)
    return [Path(match) for match in matches]


def _normalize_match(path: Path) -> Path:
    assert path.exists(), f"Suite match does not exist: {path}"
    return path.resolve()


def discover_default_test_files(tests_root: Path) -> list[Path]:
    return _discover_python_files(tests_root)


def pythonpath_test_directories(suite_roots: list[Path]) -> list[Path]:
    return [path if path.is_dir() else path.parent for path in suite_roots]


def _discover_python_files(directory: Path) -> list[Path]:
    return sorted(directory.rglob("test_*.py"))


def _normalize_test_file(path: Path) -> Path:
    assert path.is_file(), f"Suite file does not exist: {path}"
    assert path.suffix == ".py", f"Suite file must be a Python file: {path}"
    assert path.name.startswith("test_"), f"Suite file must start with 'test_': {path}"
    return path


def set_pythonpath(external_tests_root: Path, extra_test_dirs: list[Path]) -> None:
    existing = os.environ.get("PYTHONPATH", "")
    paths = []
    for path in [str(external_tests_root), *(str(p) for p in extra_test_dirs), existing]:
        if not path:
            continue
        if path not in paths:
            paths.append(path)
    os.environ["PYTHONPATH"] = os.pathsep.join(paths)


def build_debug_binary(repo_root: Path) -> None:
    result = subprocess.run(
        ["cargo", "build"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
        text=True,
    )
    assert result.returncode == 0, (
        "cargo build failed for local yamark binary.\n"
        f"{result.stdout if result.stdout else '<no output>'}"
    )


def build_markdown_ast_json_binary(repo_root: Path) -> Path:
    target_dir = repo_root / "target" / "markdown-ast-json"
    manifest = (
        repo_root / "external-tests" / "support" / "markdown-ast-json" / "Cargo.toml"
    )
    result = subprocess.run(
        ["cargo", "build", "--manifest-path", os.fspath(manifest)],
        cwd=repo_root,
        env={**os.environ, "CARGO_TARGET_DIR": os.fspath(target_dir)},
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
        text=True,
    )
    assert result.returncode == 0, (
        "cargo build failed for local markdown-ast-json binary.\n"
        f"{result.stdout if result.stdout else '<no output>'}"
    )
    return target_dir / "debug" / executable_name("markdown-ast-json")


def executable_name(name: str) -> str:
    if sys.platform == "win32":
        return f"{name}.exe"
    return name


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
