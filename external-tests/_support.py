#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import shlex
import subprocess
from collections.abc import Callable
from collections.abc import Mapping
from collections.abc import Sequence
from dataclasses import dataclass
from difflib import unified_diff
from pathlib import Path
from tempfile import TemporaryDirectory

from yaml12 import read_yaml


@dataclass(frozen=True)
class Executable:
    contents: str


def format_and_check(
    command: str,
    input_text: str,
    expected: str,
    *,
    files: Mapping[str, str | Executable] | None = None,
    env: Mapping[str, str] | None = None,
    env_path: Sequence[str] | None = None,
    stdout_contains: str | Sequence[str] | None = None,
    stderr: str | None = "",
) -> None:
    assert "{path}" in command

    with TemporaryDirectory(prefix="yamark-pytest-") as temp:
        root = Path(temp)
        write_test_files(root, files)

        stem = root / "input"
        args = command_args(command, root=root, stem=stem)
        path = command_path(args, stem)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(input_text.encode("utf-8"))

        semantic_reader = file_semantic_reader(path)
        before = semantic_reader(path) if semantic_reader is not None else None
        run_command(
            *args,
            cwd=root,
            env=command_env(root, env=env, env_path=env_path),
            stdout_contains=stdout_contains,
            stderr=stderr,
        )
        if semantic_reader is not None:
            assert before == semantic_reader(path)
        expect_identical(path.read_bytes(), expected)


def format_stdin_and_check(
    command: str,
    input_text: str,
    expected: str,
    *,
    stdin_file_path: str,
    files: Mapping[str, str | Executable] | None = None,
    env: Mapping[str, str] | None = None,
    env_path: Sequence[str] | None = None,
    stderr: str | None = "",
) -> None:
    with TemporaryDirectory(prefix="yamark-pytest-") as temp:
        root = Path(temp)
        write_test_files(root, files)

        input_path = root / stdin_file_path
        input_path.parent.mkdir(parents=True, exist_ok=True)
        input_path.write_bytes(input_text.encode("utf-8"))
        semantic_reader = file_semantic_reader(input_path)
        before = semantic_reader(input_path) if semantic_reader is not None else None

        args = command_args(command, root=root, stem=root / "input")
        result = run_command(
            *args,
            cwd=root,
            env=command_env(root, env=env, env_path=env_path),
            stdin=input_text,
            stdout=expected,
            stderr=stderr,
        )

        if semantic_reader is not None:
            output_path = root / f"output{input_path.suffix}"
            output_path.write_bytes(result.stdout)
            assert before == semantic_reader(output_path)


def run_cli_case(
    command: str,
    *,
    files: Mapping[str, str | Executable] | None = None,
    expected_files: Mapping[str, str] | None = None,
    stdin: str | bytes | None = None,
    env: Mapping[str, str] | None = None,
    env_path: Sequence[str] | None = None,
    status: int = 0,
    stdout: str | None = None,
    stderr: str | None = "",
    stdout_contains: str | Sequence[str] | None = None,
    stderr_contains: str | Sequence[str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    with TemporaryDirectory(prefix="yamark-pytest-") as temp:
        root = Path(temp)
        write_test_files(root, files)
        args = command_args(command, root=root, stem=root / "input")
        result = run_command(
            *args,
            cwd=root,
            env=command_env(root, env=env, env_path=env_path),
            stdin=stdin,
            status=status,
            stdout=stdout,
            stderr=stderr,
            stdout_contains=stdout_contains,
            stderr_contains=stderr_contains,
        )
        for relative_path, expected in (expected_files or {}).items():
            expect_identical((root / relative_path).read_bytes(), expected)
        return result


def command_args(command: str, *, root: Path, stem: Path) -> list[str]:
    replacements = {
        "path": shlex.quote(os.fspath(stem)),
        "root": shlex.quote(os.fspath(root)),
    }
    return shlex.split(command.format(**replacements))


def command_path(args: list[str], stem: Path) -> Path:
    stem_text = os.fspath(stem)
    candidates = [
        Path(arg)
        for arg in args[1:]
        if stem_text in arg and not arg.startswith("-")
    ]
    assert len(candidates) == 1, (
        f"expected command to contain exactly one formatted path, found {candidates!r}"
    )
    return candidates[0]


def is_yaml_path(path: Path) -> bool:
    return path.suffix in {".yaml", ".yml"}


def is_markdown_path(path: Path) -> bool:
    return path.suffix.removeprefix(".").lower().endswith("md")


def file_semantic_reader(path: Path) -> Callable[[Path], object] | None:
    if is_yaml_path(path):
        return read_yaml
    if is_markdown_path(path):
        return read_markdown
    return None


def read_markdown(path: Path) -> object:
    markdown_ast_json_bin = os.environ.get("MARKDOWN_AST_JSON_BIN")
    assert markdown_ast_json_bin is not None, "MARKDOWN_AST_JSON_BIN is not set"

    result = subprocess.run(
        [markdown_ast_json_bin, os.fspath(path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )

    assert result.returncode == 0, (
        f"markdown-ast-json {path} failed with exit code {result.returncode}\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as err:
        raise AssertionError(
            f"markdown-ast-json {path} emitted invalid JSON: {err}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        ) from err


def run_command(
    *args: object,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
    stdin: str | bytes | None = None,
    status: int = 0,
    stdout: str | None = None,
    stderr: str | None = "",
    stdout_contains: str | Sequence[str] | None = None,
    stderr_contains: str | Sequence[str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    assert args[0] == "yamark"
    yamark_bin = os.environ.get("YAMARK_BIN")
    assert yamark_bin is not None, "YAMARK_BIN is not set"

    shown = [command_arg(arg) for arg in args]
    actual = [yamark_bin, *shown[1:]]
    result = subprocess.run(
        actual,
        input=stdin_bytes(stdin),
        cwd=cwd,
        env=dict(env) if env is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )

    assert_result(
        result,
        shown,
        status=status,
        stdout=stdout,
        stderr=stderr,
        stdout_contains=stdout_contains,
        stderr_contains=stderr_contains,
    )
    return result


def assert_result(
    result: subprocess.CompletedProcess[bytes],
    shown: Sequence[str],
    *,
    status: int,
    stdout: str | None,
    stderr: str | None,
    stdout_contains: str | Sequence[str] | None,
    stderr_contains: str | Sequence[str] | None,
) -> None:
    assert result.returncode == status, (
        f"{shlex.join(shown)} failed with exit code {result.returncode}; "
        f"expected {status}\n"
        f"stdout:\n{decode_output(result.stdout)}\n"
        f"stderr:\n{decode_output(result.stderr)}"
    )
    if stdout is not None:
        expect_identical(result.stdout, stdout)
    if stderr is not None:
        expect_identical(result.stderr, stderr)
    assert_contains(result.stdout, stdout_contains, "stdout")
    assert_contains(result.stderr, stderr_contains, "stderr")


def assert_contains(
    output: bytes,
    expected: str | Sequence[str] | None,
    stream_name: str,
) -> None:
    if expected is None:
        return
    values = [expected] if isinstance(expected, str) else expected
    text = decode_output(output)
    for value in values:
        assert value in text, f"expected {stream_name} to contain {value!r}\n{text}"


def stdin_bytes(stdin: str | bytes | None) -> bytes | None:
    if stdin is None or isinstance(stdin, bytes):
        return stdin
    return stdin.encode("utf-8")


def decode_output(output: bytes) -> str:
    return output.decode("utf-8", errors="replace")


def command_env(
    root: Path,
    *,
    env: Mapping[str, str] | None,
    env_path: Sequence[str] | None,
) -> dict[str, str]:
    result = dict(os.environ)
    if env is not None:
        result.update(env)
    if env_path:
        paths = [os.fspath(root / path) for path in env_path]
        paths.append(result.get("PATH", ""))
        result["PATH"] = os.pathsep.join(path for path in paths if path)
    return result


def write_test_files(
    root: Path,
    files: Mapping[str, str | Executable] | None,
) -> None:
    for relative_path, contents in (files or {}).items():
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(contents, Executable):
            path.write_bytes(contents.contents.encode("utf-8"))
            path.chmod(0o755)
        else:
            path.write_bytes(contents.encode("utf-8"))


def command_arg(arg: object) -> str:
    return os.fspath(arg) if isinstance(arg, Path) else str(arg)


def tempfile(tmp_path: Path, contents: str, filename: str = "input.md") -> Path:
    path = tmp_path / filename
    path.write_bytes(contents.encode("utf-8"))
    return path


def expect_identical(actual: bytes, expected: str) -> None:
    expected_bytes = expected.encode("utf-8")
    assert actual == expected_bytes, format_diff(expected_bytes, actual)


def format_diff(expected: bytes, actual: bytes) -> str:
    expected_text = expected.decode("utf-8")
    actual_text = actual.decode("utf-8", errors="replace")
    return "".join(
        unified_diff(
            expected_text.splitlines(keepends=True),
            actual_text.splitlines(keepends=True),
            fromfile="expected",
            tofile="actual",
        )
    )
