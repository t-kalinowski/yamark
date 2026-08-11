#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shlex
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TARGET_DIR = ROOT / "target"
BENCH_DIR = ROOT / "tools" / "bench"
BIG_TOOLS = (
    "yamark,panache,mdformat,prettier,dprint-markdown,deno-fmt,"
    "yamlfmt,yamlfix,dprint-yaml"
)
DIRECTORY_TOOLS = "yamark,yamlfmt,prettier,yamlfix,dprint-yaml,deno-fmt"
REQUIRED_COMMANDS = (
    "cargo",
    "rustc",
    "Rscript",
    "panache",
    "mdformat",
    "prettier",
    "dprint",
    "deno",
    "yamlfmt",
    "yamlfix",
)
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Run Yamark's complete canonical benchmark suite: deterministic "
            "performance tests, the default per-file YAML benchmark, the "
            "large-file benchmark, and the directory YAML benchmark."
        )
    )
    parser.add_argument(
        "--result-dir",
        type=Path,
        help=(
            "New directory for the frozen binary, logs, scratch data, and "
            "artifacts (default: target/bench-all/<timestamp>-<commit>)."
        ),
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the exact commands without checking prerequisites or writing files.",
    )
    args = parser.parse_args()

    commit = git_output(["rev-parse", "HEAD"])
    tree = git_output(["rev-parse", "HEAD^{tree}"])
    short_commit = git_output(["rev-parse", "--short=12", "HEAD"])
    result_dir = (args.result_dir or default_result_dir(short_commit)).resolve()
    steps = benchmark_steps(result_dir)

    print(f"benchmark commit: {commit}")
    print(f"benchmark tree:   {tree}")
    print(f"results:          {result_dir}")
    if args.dry_run:
        print_steps(steps)
        return 0

    require_clean_worktree()
    validate_result_dir(result_dir)
    r_packages = preflight()

    result_dir.mkdir(parents=True)
    (result_dir / "logs").mkdir()
    (result_dir / "bin").mkdir()
    started_at = utc_now()
    frozen_binary = result_dir / "bin" / "yamark"

    for label, command in steps:
        run_step(label, command, result_dir / "logs" / f"{label}.log")
        if label == "build":
            cargo_target = result_dir / "scratch" / "cargo-target"
            built_binary = find_built_binary(cargo_target)
            shutil.copy2(built_binary, frozen_binary)
            shutil.rmtree(cargo_target)

    if not frozen_binary.is_file():
        raise SystemExit(f"release build did not produce {frozen_binary}")

    artifacts = validate_artifacts(
        result_dir, commit, tree, frozen_binary=frozen_binary
    )
    corpora = corpus_records(result_dir)
    require_unchanged_worktree(commit, tree)
    write_manifest(
        result_dir=result_dir,
        started_at=started_at,
        commit=commit,
        tree=tree,
        steps=steps,
        frozen_binary=frozen_binary,
        artifacts=artifacts,
        corpora=corpora,
        r_packages=r_packages,
    )
    print(f"\ncomplete: {result_dir / 'manifest.json'}")
    return 0


def default_result_dir(short_commit: str) -> Path:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return TARGET_DIR / "bench-all" / f"{timestamp}-{short_commit}"


def benchmark_steps(result_dir: Path) -> list[tuple[str, list[str]]]:
    python = sys.executable
    yamark_bin = result_dir / "bin" / "yamark"
    scratch = result_dir / "scratch"
    artifacts = result_dir / "artifacts"
    common = [
        "--skip-yamark-build",
        "--yamark-bin",
        str(yamark_bin),
        "--keep-corpus",
    ]
    return [
        (
            "yaml-performance-tests",
            ["cargo", "test", "--locked", "--test", "yaml_performance"],
        ),
        (
            "benchmark-tool-tests",
            ["cargo", "test", "--locked", "--test", "benchmark_tools"],
        ),
        (
            "build",
            [
                "cargo",
                "build",
                "--release",
                "--locked",
                "--bin",
                "yamark",
                "--target-dir",
                str(scratch / "cargo-target"),
            ],
        ),
        (
            "default-yaml",
            [
                python,
                str(BENCH_DIR / "run.py"),
                "--corpus",
                "yaml",
                "--corpus-shape",
                "flow-heavy",
                "--invocation",
                "per-file",
                "--operation",
                "write",
                "--width-profile",
                "default",
                "--files",
                "400",
                "--items",
                "80",
                "--reps",
                "2",
                "--warmups",
                "1",
                "--tools",
                "yamark",
                *common,
                "--out-dir",
                str(scratch / "default"),
                "--artifact-dir",
                str(artifacts / "default"),
            ],
        ),
        (
            "big-files",
            [
                python,
                str(BENCH_DIR / "big.py"),
                "--target-bytes",
                "4000000",
                "--frontmatter-yaml-bytes",
                "200000",
                "--seed",
                "20260602",
                "--reps",
                "10",
                "--warmups",
                "2",
                "--tools",
                BIG_TOOLS,
                *common,
                "--out-dir",
                str(scratch / "big"),
                "--artifact-dir",
                str(artifacts / "big"),
            ],
        ),
        (
            "directory-yaml",
            [
                python,
                str(BENCH_DIR / "run.py"),
                "--corpus",
                "yaml",
                "--corpus-shape",
                "flow-heavy",
                "--invocation",
                "directory",
                "--operation",
                "write",
                "--width-profile",
                "default",
                "--files",
                "500",
                "--items",
                "540",
                "--reps",
                "3",
                "--warmups",
                "1",
                "--tools",
                DIRECTORY_TOOLS,
                *common,
                "--out-dir",
                str(scratch / "directory"),
                "--artifact-dir",
                str(artifacts / "directory"),
            ],
        ),
    ]


def print_steps(steps: list[tuple[str, list[str]]]) -> None:
    for label, command in steps:
        print(f"\n[{label}]")
        print(shlex.join(command))


def require_clean_worktree() -> None:
    status = git_output(["status", "--porcelain=v1"])
    if status:
        raise SystemExit("working tree must be clean before running benchmarks")


def validate_result_dir(result_dir: Path) -> None:
    if result_dir.exists():
        raise SystemExit(f"result directory already exists: {result_dir}")
    if result_dir == ROOT or result_dir == TARGET_DIR:
        raise SystemExit(f"result directory is too broad: {result_dir}")
    if path_is_within(result_dir, ROOT) and not path_is_within(result_dir, TARGET_DIR):
        raise SystemExit(
            "a result directory inside the repository must be under target/"
        )


def path_is_within(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
    except ValueError:
        return False
    return True


def preflight() -> dict[str, str]:
    missing = [
        command for command in REQUIRED_COMMANDS if shutil.which(command) is None
    ]
    if missing:
        raise SystemExit(f"missing benchmark commands: {', '.join(missing)}")
    if not hasattr(os, "wait4"):
        raise SystemExit("os.wait4 is required for benchmark RSS measurements")
    try:
        import resource  # noqa: F401
    except ImportError as error:
        raise SystemExit(
            "the Python resource module is required for benchmark RSS measurements"
        ) from error

    r_packages = subprocess.run(
        [
            "Rscript",
            "--vanilla",
            "-e",
            (
                "for (package in c('stringi', 'yaml12')) { "
                "if (!requireNamespace(package, quietly=TRUE)) quit(status=1); "
                "cat(package, '=', as.character(packageVersion(package)), "
                "'\\n', sep='') }"
            ),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if r_packages.returncode != 0:
        raise SystemExit(
            "R packages stringi and yaml12 are required for the complete benchmark suite"
        )
    versions = dict(
        line.split("=", 1) for line in r_packages.stdout.splitlines() if "=" in line
    )
    if set(versions) != {"stringi", "yaml12"}:
        raise SystemExit("could not determine stringi and yaml12 package versions")
    return versions


def run_step(label: str, command: list[str], log_path: Path) -> None:
    print(f"\n[{label}]")
    print(shlex.join(command), flush=True)
    with log_path.open("w", encoding="utf-8") as log:
        log.write(f"$ {shlex.join(command)}\n")
        log.flush()
        process = subprocess.Popen(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            errors="replace",
        )
        assert process.stdout is not None
        for line in process.stdout:
            print(line, end="", flush=True)
            log.write(line)
            log.flush()
        returncode = process.wait()
    if returncode != 0:
        raise SystemExit(f"{label} failed with exit status {returncode}: {log_path}")


def find_built_binary(target_dir: Path) -> Path:
    candidates = sorted(
        path
        for path in target_dir.glob("**/release/yamark")
        if path.is_file() and path.parent.name == "release"
    )
    if len(candidates) != 1:
        raise SystemExit(
            f"expected one Yamark release binary in {target_dir}, "
            f"found {len(candidates)}"
        )
    return candidates[0]


def validate_artifacts(
    result_dir: Path, commit: str, tree: str, *, frozen_binary: Path
) -> list[tuple[str, Path]]:
    expected = {
        "default": {
            "benchmark": "yaml-formatting",
            "formatters": ["yamark"],
            "ok": 1,
            "skipped": 0,
            "reps": 2,
            "warmups": 1,
        },
        "big": {
            "benchmark": "big-file-formatting",
            "formatters": BIG_TOOLS.split(","),
            "ok": 18,
            "skipped": 9,
            "reps": 10,
            "warmups": 2,
        },
        "directory": {
            "benchmark": "yaml-formatting",
            "formatters": DIRECTORY_TOOLS.split(","),
            "ok": 6,
            "skipped": 0,
            "reps": 3,
            "warmups": 1,
        },
    }
    artifact_root = result_dir / "artifacts"
    all_paths = sorted(artifact_root.rglob("*.json"))
    if len(all_paths) != len(expected):
        raise SystemExit(
            f"expected {len(expected)} benchmark artifacts, found {len(all_paths)}"
        )
    validated = []
    for label, contract in expected.items():
        artifact_dir = artifact_root / label
        paths = sorted(artifact_dir.glob("*.json"))
        if len(paths) != 1:
            raise SystemExit(
                f"expected one {label} artifact in {artifact_dir}, found {len(paths)}"
            )
        path = paths[0]
        data = json.loads(path.read_text(encoding="utf-8"))
        validate_artifact(
            path,
            data,
            label=label,
            contract=contract,
            commit=commit,
            tree=tree,
            frozen_binary=frozen_binary,
        )
        validated.append((label, path))
    if {path for _, path in validated} != set(all_paths):
        raise SystemExit(f"unexpected benchmark artifact under {artifact_root}")
    return validated


def validate_artifact(
    path: Path,
    data: dict[str, object],
    *,
    label: str,
    contract: dict[str, object],
    commit: str,
    tree: str,
    frozen_binary: Path,
) -> None:
    if data.get("schema_version") != 1:
        raise SystemExit(f"unexpected artifact schema in {path}")
    if data.get("benchmark") != contract["benchmark"]:
        raise SystemExit(f"unexpected benchmark in {path}")
    git = data.get("git")
    if not isinstance(git, dict) or git.get("commit") != commit:
        raise SystemExit(f"artifact commit does not match the benchmark commit: {path}")
    if git.get("tree") != tree or git.get("dirty") is not False:
        raise SystemExit(
            f"artifact was not produced from the clean benchmark tree: {path}"
        )
    if data.get("selected_formatters") != contract["formatters"]:
        raise SystemExit(f"unexpected formatter roster in {path}")
    versions = data.get("tool_versions")
    if not isinstance(versions, dict) or set(versions) != set(contract["formatters"]):
        raise SystemExit(f"incomplete formatter versions in {path}")
    if any(not versions[formatter] for formatter in contract["formatters"]):
        raise SystemExit(f"missing formatter version in {path}")

    validate_workload(path, data, label)

    results = data.get("results")
    if not isinstance(results, list):
        raise SystemExit(f"missing benchmark results in {path}")
    if any(not isinstance(result, dict) for result in results):
        raise SystemExit(f"invalid benchmark result in {path}")
    result_rows = [result for result in results if isinstance(result, dict)]
    validate_result_matrix(path, result_rows, label, contract["formatters"])

    ok = [result for result in result_rows if result.get("status") == "ok"]
    skipped = [result for result in result_rows if result.get("status") == "skipped"]
    if len(ok) != contract["ok"] or len(skipped) != contract["skipped"]:
        raise SystemExit(
            f"incomplete benchmark results in {path}: "
            f"{len(ok)} ok, {len(skipped)} skipped"
        )
    if len(ok) + len(skipped) != len(results):
        raise SystemExit(f"failed or unknown benchmark result in {path}")
    for result in ok:
        if result.get("reps") != contract["reps"]:
            raise SystemExit(f"unexpected repetition count in {path}")
        if result.get("warmups") != contract["warmups"]:
            raise SystemExit(f"unexpected warmup count in {path}")
        output_hash = result.get("output_hash")
        if not isinstance(output_hash, str) or not SHA256_PATTERN.fullmatch(
            output_hash
        ):
            raise SystemExit(f"invalid output hash in {path}")
        sample_fields = ["repetitions", "user_seconds", "sys_seconds"]
        if label == "big":
            sample_fields.append("peak_rss_bytes")
        for field in sample_fields:
            samples = result.get(field)
            if not isinstance(samples, list) or len(samples) != contract["reps"]:
                raise SystemExit(f"invalid {field} samples in {path}")
        if result.get("formatter") == "yamark":
            validate_yamark_command(path, result, frozen_binary)
    if any("does not support" not in str(result.get("reason")) for result in skipped):
        raise SystemExit(f"formatter unavailable or failed its probe in {path}")


def validate_workload(path: Path, data: dict[str, object], label: str) -> None:
    corpus = require_mapping(path, data, "corpus")
    if label == "big":
        require_fields(
            path,
            corpus,
            {
                "kind": "big-file",
                "files": 3,
                "targets": ["big.md", "big.yaml", "big-with-frontmatter.md"],
                "requested_bytes": {
                    "markdown": 4_000_000,
                    "yaml": 4_000_000,
                    "frontmatter": 4_000_000,
                    "frontmatter_yaml": 200_000,
                },
                "seed": 20260602,
            },
        )
        return

    files = 400 if label == "default" else 500
    items = 80 if label == "default" else 540
    invocation_unit = "per-file" if label == "default" else "directory"
    require_fields(
        path,
        corpus,
        {
            "kind": "yaml",
            "shape": "flow-heavy",
            "files": files,
            "items_per_file": items,
        },
    )
    invocation = require_mapping(path, data, "invocation")
    require_fields(path, invocation, {"unit": invocation_unit})
    targets = invocation.get("targets")
    if not isinstance(targets, list) or len(targets) != files:
        raise SystemExit(f"unexpected target list in {path}")
    require_fields(path, data, {"operation": "write"})
    formatting_options = require_mapping(path, data, "formatting_options")
    require_fields(path, formatting_options, {"width_profile": "default"})


def validate_result_matrix(
    path: Path,
    results: list[dict[str, object]],
    label: str,
    formatters: object,
) -> None:
    if not isinstance(formatters, list):
        raise SystemExit(f"invalid formatter contract for {path}")
    if label != "big":
        expected = set(formatters)
        actual = [result.get("formatter") for result in results]
        if len(actual) != len(set(actual)) or set(actual) != expected:
            raise SystemExit(f"incomplete or duplicate formatter results in {path}")
        return

    targets = ("big.md", "big.yaml", "big-with-frontmatter.md")
    markdown_formatters = {
        "yamark",
        "panache",
        "mdformat",
        "prettier",
        "dprint-markdown",
        "deno-fmt",
    }
    yaml_formatters = {
        "yamark",
        "prettier",
        "deno-fmt",
        "yamlfmt",
        "yamlfix",
        "dprint-yaml",
    }
    expected = {}
    for formatter in formatters:
        for target in targets:
            supported = (
                formatter in yaml_formatters
                if target.endswith(".yaml")
                else formatter in markdown_formatters
            )
            expected[(formatter, target)] = "ok" if supported else "skipped"

    actual = {}
    for result in results:
        key = (result.get("formatter"), result.get("file"))
        if key in actual:
            raise SystemExit(f"duplicate formatter/file result in {path}: {key}")
        actual[key] = result.get("status")
    if actual != expected:
        raise SystemExit(f"unexpected formatter/file result matrix in {path}")


def validate_yamark_command(
    path: Path, result: dict[str, object], frozen_binary: Path
) -> None:
    command = result.get("command")
    if not isinstance(command, str):
        raise SystemExit(f"missing Yamark command in {path}")
    argv = shlex.split(command)
    if len(argv) < 2 or argv[:2] != [str(frozen_binary), "format"]:
        raise SystemExit(f"artifact did not measure the frozen Yamark binary: {path}")


def require_mapping(
    path: Path, data: dict[str, object], field: str
) -> dict[str, object]:
    value = data.get(field)
    if not isinstance(value, dict):
        raise SystemExit(f"missing {field} metadata in {path}")
    return value


def require_fields(
    path: Path, data: dict[str, object], expected: dict[str, object]
) -> None:
    for field, value in expected.items():
        if data.get(field) != value:
            raise SystemExit(f"unexpected {field} metadata in {path}")


def corpus_records(result_dir: Path) -> list[dict[str, object]]:
    records = []
    for label, expected_files in (("default", 400), ("big", 3), ("directory", 500)):
        roots = sorted((result_dir / "scratch" / label).glob("*/corpus"))
        roots = [root for root in roots if root.is_dir()]
        if len(roots) != 1:
            raise SystemExit(
                f"expected one retained {label} corpus, found {len(roots)}"
            )
        record = corpus_record(roots[0], result_dir)
        if record["files"] != expected_files:
            raise SystemExit(
                f"expected {expected_files} files in the {label} corpus, "
                f"found {record['files']}"
            )
        records.append({"label": label, **record})
    return records


def corpus_record(corpus_dir: Path, result_dir: Path) -> dict[str, object]:
    files = sorted(path for path in corpus_dir.rglob("*") if path.is_file())
    digest = hashlib.sha256()
    total_bytes = 0
    for path in files:
        relative = path.relative_to(corpus_dir).as_posix().encode("utf-8")
        size = path.stat().st_size
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update(size.to_bytes(8, "big"))
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        total_bytes += size
    return {
        "path": corpus_dir.relative_to(result_dir).as_posix(),
        "files": len(files),
        "bytes": total_bytes,
        "sha256": digest.hexdigest(),
    }


def require_unchanged_worktree(commit: str, tree: str) -> None:
    if git_output(["rev-parse", "HEAD"]) != commit:
        raise SystemExit("HEAD changed during the benchmark run")
    if git_output(["rev-parse", "HEAD^{tree}"]) != tree:
        raise SystemExit("the Git tree changed during the benchmark run")
    require_clean_worktree()


def write_manifest(
    *,
    result_dir: Path,
    started_at: str,
    commit: str,
    tree: str,
    steps: list[tuple[str, list[str]]],
    frozen_binary: Path,
    artifacts: list[tuple[str, Path]],
    corpora: list[dict[str, object]],
    r_packages: dict[str, str],
) -> None:
    manifest = {
        "schema_version": 1,
        "status": "complete",
        "started_at": started_at,
        "completed_at": utc_now(),
        "git": {"commit": commit, "tree": tree, "dirty": False},
        "host": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "python": platform.python_version(),
            "rustc": command_version(["rustc", "--version"]),
            "cargo": command_version(["cargo", "--version"]),
            "r": command_version(["Rscript", "--version"]),
            "r_packages": r_packages,
        },
        "binary": file_record(frozen_binary, result_dir),
        "corpora": corpora,
        "steps": [{"label": label, "argv": command} for label, command in steps],
        "artifacts": [
            {"label": label, **file_record(path, result_dir)}
            for label, path in artifacts
        ],
    }
    path = result_dir / "manifest.json"
    temporary = result_dir / ".manifest.json.tmp"
    temporary.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    temporary.replace(path)


def file_record(path: Path, result_dir: Path) -> dict[str, object]:
    return {
        "path": path.relative_to(result_dir).as_posix(),
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_version(command: list[str]) -> str:
    proc = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(f"version command failed: {shlex.join(command)}")
    return proc.stdout.strip().splitlines()[0]


def git_output(args: list[str]) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise SystemExit(f"git {shlex.join(args)} failed: {proc.stderr.strip()}")
    return proc.stdout.strip()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


if __name__ == "__main__":
    sys.exit(main())
