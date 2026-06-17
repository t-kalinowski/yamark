#!/usr/bin/env python3

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import platform
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import textwrap
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

try:
    import resource
except ImportError:
    resource = None


ROOT = Path(__file__).resolve().parents[2]
BENCH_DIR = Path(__file__).resolve().parent
DEFAULT_TOOLS = "yamark"
DEFAULT_FILES = 400
DEFAULT_ITEMS = 80
DEFAULT_REPS = 2
DEFAULT_INVOCATION = "per-file"
DEFAULT_CORPUS_SHAPE = "flow-heavy"
INVOCATIONS = ("directory", "per-file")
OPERATIONS = ("write", "check")
CORPUS_SHAPES = ("flow-heavy", "block-heavy", "mixed-node")
WIDTH_PROFILES = ("default", "flow-preserve-wide", "no-prose-wrap")
PROBE_YAML = "probe:    [one,two]\n"
DPRINT_YAML_CONFIG = BENCH_DIR / "dprint-yaml.json"
PRETTY_YAML_DRIVER_ENV = "YAMARK_PRETTY_YAML_DRIVER"
PRETTY_YAML_DRIVER_DIR = ROOT / "target" / "bench-tools" / "pretty-yaml-driver"
PRETTY_YAML_DRIVER_TARGET = ROOT / "target" / "bench-tools" / "pretty-yaml-driver-target"
PRETTY_YAML_CRATE_VERSION = "0.6.0"
AGENT_LOG_TAIL_LINES = 20


@dataclass(frozen=True)
class Tool:
    name: str
    command: list[str]
    directory_command: list[str] | None = None
    check_command: list[str] | None = None
    check_directory_command: list[str] | None = None
    availability: list[str] | None = None
    version_command: list[str] | None = None
    version: str | None = None
    prepare: str | None = None
    prepare_command: list[str] | None = None
    cwd: str = "repo"
    # Redirect the tool's cache into the benchmark work dir via this env var,
    # so timed runs never read state left by earlier runs or by the user's
    # own usage of the tool. Entries not named in cache_keep are removed
    # before every repetition (cache_keep preserves one-time downloads such
    # as dprint plugins).
    cache_env: str | None = None
    cache_keep: tuple[str, ...] = ()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Benchmark YAML formatting throughput across formatter CLIs."
    )
    parser.add_argument("--corpus", choices=["yaml"], default="yaml")
    parser.add_argument("--files", type=int, default=DEFAULT_FILES)
    parser.add_argument("--items", type=int, default=DEFAULT_ITEMS)
    parser.add_argument(
        "--invocation",
        choices=INVOCATIONS,
        default=DEFAULT_INVOCATION,
        help=(
            "How to invoke formatter CLIs. 'directory' passes the copied "
            "corpus root once; 'per-file' invokes once for each YAML file."
        ),
    )
    parser.add_argument(
        "--operation",
        choices=OPERATIONS,
        default="write",
        help="'write' formats files in place; 'check' measures no-write check mode.",
    )
    parser.add_argument(
        "--corpus-shape",
        choices=CORPUS_SHAPES,
        default=DEFAULT_CORPUS_SHAPE,
        help="Generated YAML shape to benchmark.",
    )
    parser.add_argument(
        "--width-profile",
        choices=WIDTH_PROFILES,
        default="default",
        help="Formatting width options for tools that support them.",
    )
    parser.add_argument("--reps", type=int, default=DEFAULT_REPS)
    parser.add_argument("--warmups", type=int, default=1)
    parser.add_argument("--tools", default=DEFAULT_TOOLS)
    parser.add_argument(
        "--python",
        default=sys.executable,
        help="Python interpreter for benchmark wrapper scripts.",
    )
    parser.add_argument("--dprint-bin", default="dprint")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT / "target" / "bench-yaml",
        help="Directory for generated corpora and transient raw benchmark inputs.",
    )
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        default=ROOT / "docs" / "benchmarks" / "yaml",
        help="Directory for commit-scoped JSON result artifacts.",
    )
    parser.add_argument(
        "--yamark-bin",
        type=Path,
        default=ROOT / "target" / "release" / "yamark",
    )
    parser.add_argument("--skip-yamark-build", action="store_true")
    parser.add_argument("--keep-workdirs", action="store_true")
    parser.add_argument("--keep-corpus", action="store_true")
    parser.add_argument(
        "--agent-summary",
        action="store_true",
        help="Print a bounded model-friendly summary and write the run log to a file.",
    )
    args = parser.parse_args()

    return run_benchmark(args)


def run_benchmark(args: argparse.Namespace) -> int:
    stopifnot(args.files > 0, "--files must be positive")
    stopifnot(args.items > 0, "--items must be positive")
    stopifnot(args.reps > 0, "--reps must be positive")
    stopifnot(args.warmups >= 0, "--warmups must be >= 0")

    selected = selected_tools(args.tools)
    tools = tool_specs(args)
    unknown = [name for name in selected if name not in tools]
    if unknown:
        raise SystemExit(f"unknown formatter: {', '.join(unknown)}")
    validate_selected_modes(selected, tools, args.operation, args.width_profile)

    if "yamark" in selected and not args.skip_yamark_build:
        run_checked(["cargo", "build", "--release", "--bin", "yamark"], cwd=ROOT)

    tool_versions = {name: tool_version(tools[name]) for name in selected}

    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    run_dir = args.out_dir / run_id
    corpus_dir = run_dir / "corpus"
    log_path = agent_log_path(args.out_dir, run_id) if args.agent_summary else None
    log_lines: list[str] = []
    generate_yaml_corpus(corpus_dir, args.files, args.items, args.corpus_shape)
    generated = sorted(corpus_dir.rglob("*.yaml"))
    target_relatives = [path.relative_to(corpus_dir) for path in generated]
    corpus_bytes = sum(path.stat().st_size for path in generated)
    work_root: Path | None = None

    try:
        work_root = Path(
            tempfile.mkdtemp(prefix=f"yamark-yaml-bench-{run_id}-", dir="/tmp")
        ).resolve()
        results = []
        for name in selected:
            result = run_tool(
                tool=tools[name],
                corpus_dir=corpus_dir,
                target_relatives=target_relatives,
                invocation=args.invocation,
                operation=args.operation,
                width_profile=args.width_profile,
                work_root=work_root,
                files=len(generated),
                corpus_bytes=corpus_bytes,
                reps=args.reps,
                warmups=args.warmups,
            )
            results.append(result)
            report_line(format_result(result), args, log_lines)

        artifact = benchmark_document(
            run_id=run_id,
            args=args,
            files=len(generated),
            corpus_bytes=corpus_bytes,
            target_relatives=target_relatives,
            invocation=args.invocation,
            operation=args.operation,
            width_profile=args.width_profile,
            selected=selected,
            tool_versions=tool_versions,
            results=results,
        )
        artifact_path = write_artifact(args.artifact_dir, artifact)
        report_line(f"artifact: {display_path(artifact_path)}", args, log_lines)
        if args.agent_summary:
            assert log_path is not None
            write_agent_log(log_path, log_lines)
            return print_agent_summary(
                artifact_dir=args.artifact_dir,
                artifact=artifact,
                artifact_path=artifact_path,
                results=results,
                log_path=log_path,
            )
        return 0
    finally:
        if work_root is not None and not args.keep_workdirs and work_root.exists():
            shutil.rmtree(work_root)
        if not args.keep_corpus and run_dir.exists():
            shutil.rmtree(run_dir)


def stopifnot(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def selected_tools(tools: str) -> list[str]:
    selected = [tool.strip() for tool in tools.split(",") if tool.strip()]
    if not selected:
        raise SystemExit("at least one formatter must be selected")
    return selected


def validate_selected_modes(
    selected: list[str],
    tools: dict[str, Tool],
    operation: str,
    width_profile: str,
) -> None:
    if operation == "check":
        unsupported = [
            name
            for name in selected
            if tools[name].check_command is None
            and tools[name].check_directory_command is None
        ]
        if unsupported:
            raise SystemExit(
                "--operation check is unsupported by: " + ", ".join(unsupported)
            )
    if width_profile != "default":
        unsupported = [name for name in selected if name != "yamark"]
        if unsupported:
            raise SystemExit(
                f"--width-profile {width_profile} is unsupported by: "
                + ", ".join(unsupported)
            )


def tool_specs(args: argparse.Namespace) -> dict[str, Tool]:
    yamark_bin = str(args.yamark_bin)
    python = str(args.python)
    dprint_bin = str(args.dprint_bin)
    yamark_options = yamark_width_options(args.width_profile)
    return {
        "yamark": Tool(
            name="yamark",
            command=[yamark_bin, "format", *yamark_options, "{target}"],
            check_command=[yamark_bin, "format", "--check", *yamark_options, "{target}"],
            version=yamark_cargo_version(),
        ),
        "yamlfmt": Tool(
            name="yamlfmt",
            command=["yamlfmt", "{target}"],
            availability=["yamlfmt", "--version"],
            version_command=["yamlfmt", "--version"],
        ),
        "prettier": Tool(
            name="prettier",
            command=["prettier", "--write", "{target}"],
            check_command=["prettier", "--check", "{target}"],
            availability=["prettier", "--version"],
            version_command=["prettier", "--version"],
        ),
        "yamlfix": Tool(
            name="yamlfix",
            command=["yamlfix", "{target}"],
            availability=["yamlfix", "--version"],
            version_command=["yamlfix", "--version"],
        ),
        "panache-yaml": Tool(
            name="panache-yaml",
            command=[python, str(BENCH_DIR / "format_panache_yaml.py"), "{target}"],
            availability=["panache", "--version"],
            version_command=["panache", "--version"],
        ),
        "dprint-yaml": Tool(
            name="dprint-yaml",
            # dprint has no default plugins; the config file names the YAML
            # plugin and nothing else. The cache (plugins plus incremental
            # state) is redirected and the incremental state is wiped
            # before every repetition.
            command=[
                dprint_bin,
                "fmt",
                "--config",
                str(DPRINT_YAML_CONFIG),
                "{target}",
            ],
            directory_command=[
                dprint_bin,
                "fmt",
                "--config",
                str(DPRINT_YAML_CONFIG),
                ".",
            ],
            check_command=[
                dprint_bin,
                "check",
                "--config",
                str(DPRINT_YAML_CONFIG),
                "{target}",
            ],
            check_directory_command=[
                dprint_bin,
                "check",
                "--config",
                str(DPRINT_YAML_CONFIG),
                ".",
            ],
            availability=[dprint_bin, "--version"],
            version_command=[dprint_bin, "--version"],
            prepare_command=[
                dprint_bin,
                "output-resolved-config",
                "--config",
                str(DPRINT_YAML_CONFIG),
            ],
            cwd="work",
            cache_env="DPRINT_CACHE_DIR",
            cache_keep=("plugins", "plugin-cache-manifest.json", "locks"),
        ),
        "deno-fmt": Tool(
            name="deno-fmt",
            command=["deno", "fmt", "--no-config", "--quiet", "{target}"],
            check_command=[
                "deno",
                "fmt",
                "--check",
                "--no-config",
                "--quiet",
                "{target}",
            ],
            availability=["deno", "--version"],
            version_command=["deno", "--version"],
        ),
        "pretty-yaml": Tool(
            name="pretty-yaml",
            command=[str(pretty_yaml_driver_path()), "{target}"],
            prepare="pretty-yaml-driver",
            version=f"pretty_yaml crate {PRETTY_YAML_CRATE_VERSION}",
        ),
        "py-yaml12": Tool(
            name="py-yaml12",
            command=[python, str(BENCH_DIR / "format_yaml12.py"), "{target}"],
            availability=[python, "-c", "import yaml12"],
            version_command=[
                python,
                "-c",
                "import yaml12; print('yaml12', getattr(yaml12, '__version__', ''))",
            ],
        ),
    }


def yamark_width_options(width_profile: str) -> list[str]:
    if width_profile == "default":
        return []
    if width_profile == "flow-preserve-wide":
        return ["--line-width", "1000", "--prose-width", "1000"]
    if width_profile == "no-prose-wrap":
        return ["--prose-width", "1000"]
    raise AssertionError(f"unsupported width profile: {width_profile}")


def run_checked(command: list[str], *, cwd: Path) -> None:
    proc = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"{shlex.join(command)} failed\n{proc.stdout}{proc.stderr}"
        )


def child_usage() -> tuple[float, float]:
    if resource is None:
        return (0.0, 0.0)
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return (usage.ru_utime, usage.ru_stime)


def generate_yaml_corpus(root: Path, files: int, items: int, corpus_shape: str) -> None:
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True)
    for file_idx in range(files):
        subdir = root / f"service-{file_idx % 20:02d}"
        subdir.mkdir(exist_ok=True)
        (subdir / f"config-{file_idx:04d}.yaml").write_text(
            render_yaml_file(file_idx, items, corpus_shape),
            encoding="utf-8",
        )


def render_yaml_file(file_idx: int, items: int, corpus_shape: str) -> str:
    if corpus_shape == "mixed-node":
        return render_mixed_node_file(file_idx, items)

    lines = [
        "# generated benchmark fixture",
        f"name:    service-{file_idx:04d}",
        "enabled: true",
        f"version: {1 + file_idx % 7}.{file_idx % 11}.{file_idx % 13}",
        f"labels: {{team: team-{file_idx % 8},region: us-{file_idx % 4},tier: backend}}",
        "defaults: &defaults",
        "  retries: 3",
        "  timeout: 30s",
        "  env: {LOG_LEVEL: info,FEATURE_FLAG: false}",
        "service:",
        "  <<: *defaults",
        f"  image: \"registry.example.com/service-{file_idx % 17}:{file_idx % 29}\"",
        f"  ports: [{8000 + file_idx % 100},{9000 + file_idx % 100}]",
        "description: >",
        (
            f"  Service service-{file_idx:04d} has a folded prose summary that "
            "exercises realistic configuration text without becoming huge."
        ),
        "script: |",
        f"  echo \"service-{file_idx:04d}\"",
        "  cargo test --package yamark",
        "routes:",
        f"  - {{path: /api/service-{file_idx:04d},methods: [GET,POST]}}",
        "settings:",
    ]
    for item_idx in range(items):
        name = f"worker-{item_idx:03d}"
        replicas = 1 + (file_idx + item_idx) % 9
        port_a = 8000 + item_idx % 100
        port_b = 9000 + (file_idx + item_idx) % 100
        log_level = "debug" if item_idx % 5 == 0 else "info"
        feature = "true" if (file_idx + item_idx) % 3 == 0 else "false"
        cpu = 100 + item_idx % 20
        memory = 128 + (item_idx % 12) * 32
        dep_a = (file_idx + item_idx + 1) % 50
        dep_b = (file_idx + item_idx + 7) % 50
        if corpus_shape == "flow-heavy":
            lines.append(
                (
                    f"  item_{item_idx:03d}: {{name: {name},"
                    f"replicas: {replicas},"
                    f"ports: [{port_a},{port_b}],"
                    f"env: {{LOG_LEVEL: {log_level},"
                    f"FEATURE_FLAG: {feature}}},"
                    f"resources: {{cpu: {cpu}m,memory: {memory}Mi}},"
                    f"dependencies: [service-{dep_a:04d},"
                    f"service-{dep_b:04d}]}}"
                )
            )
        elif corpus_shape == "block-heavy":
            lines.extend(
                [
                    f"  item_{item_idx:03d}:",
                    f"    name: {name}",
                    f"    replicas: {replicas}",
                    "    ports:",
                    f"      - {port_a}",
                    f"      - {port_b}",
                    "    env:",
                    f"      LOG_LEVEL: {log_level}",
                    f"      FEATURE_FLAG: {feature}",
                    "    resources:",
                    f"      cpu: {cpu}m",
                    f"      memory: {memory}Mi",
                    "    dependencies:",
                    f"      - service-{dep_a:04d}",
                    f"      - service-{dep_b:04d}",
                ]
            )
        else:
            raise AssertionError(f"unsupported corpus shape: {corpus_shape}")
    lines.extend(
        [
            f"metadata: {{owner: user-{file_idx % 25},description: generated benchmark fixture {file_idx}}}",
            "",
        ]
    )
    return "\n".join(lines)


def render_mixed_node_file(file_idx: int, items: int) -> str:
    nodes = []
    for item_idx in range(items):
        nodes.append(
            {
                "id": f"service-{file_idx:04d}-node-{item_idx:05d}",
                "str": [
                    "Lorem ipsum dolor sit amet.",
                    "Ut enim ad minim veniam.",
                    "Duis aute irure dolor in reprehenderit.",
                    "Excepteur sint occaecat cupidatat non proident.",
                ],
                "block_str": "Lorem ipsum dolor sit amet.\nUt enim ad minim veniam.\n",
                "bools": [True, False],
                "ints": [123, -123],
                "floats": [123.456, -123.456],
                "null": None,
                "nested": {
                    "seq": ["alpha", "beta", "gamma"],
                    "map": {
                        "enabled": True,
                        "count": 123,
                        "ratio": 0.125,
                    },
                },
            }
        )
    return json.dumps(nodes, separators=(",", ":"), ensure_ascii=True)


def run_tool(
    *,
    tool: Tool,
    corpus_dir: Path,
    target_relatives: list[Path],
    invocation: str,
    operation: str,
    width_profile: str,
    work_root: Path,
    files: int,
    corpus_bytes: int,
    reps: int,
    warmups: int,
) -> dict[str, object]:
    if tool.availability is not None and not command_available(tool.availability):
        return {
            "formatter": tool.name,
            "status": "skipped",
            "invocation": invocation,
            "operation": operation,
            "reason": f"{tool.availability[0]} is not available on PATH",
        }

    prepared = prepare_tool(tool, work_root)
    if prepared is not None:
        return {
            "formatter": tool.name,
            "status": "skipped",
            "invocation": invocation,
            "operation": operation,
            "reason": prepared,
        }

    probe = run_probe(tool, work_root)
    if probe is not None:
        return {
            "formatter": tool.name,
            "status": "skipped",
            "invocation": invocation,
            "operation": operation,
            "reason": probe,
        }

    repetitions = []
    user_repetitions = []
    sys_repetitions = []
    metrics = None
    for run_idx in range(warmups + reps):
        run_root = work_root / tool.name / f"run-{run_idx:02d}"
        if run_root.exists():
            shutil.rmtree(run_root)
        shutil.copytree(corpus_dir, run_root)
        wipe_tool_cache(tool, work_root)
        cwd = run_root if tool.cwd == "work" else ROOT
        env = tool_env(tool, work_root)
        usage_before = child_usage()
        started = time.perf_counter()
        proc = run_invocation(
            tool, run_root, target_relatives, invocation, operation, cwd, env
        )
        elapsed = time.perf_counter() - started
        usage_after = child_usage()
        if operation == "write" and proc.returncode != 0:
            return {
                "formatter": tool.name,
                "status": "failed",
                "invocation": invocation,
                "operation": operation,
                "command": display_command(tool, invocation, operation),
                "exit_code": proc.returncode,
                "reason": stderr_tail(proc.stderr),
            }
        if operation == "check" and proc.returncode not in (0, 1):
            return {
                "formatter": tool.name,
                "status": "failed",
                "invocation": invocation,
                "operation": operation,
                "command": display_command(tool, invocation, operation),
                "exit_code": proc.returncode,
                "reason": stderr_tail(proc.stderr),
            }
        if run_idx >= warmups:
            run_metrics = output_metrics(corpus_dir, run_root)
            measured_idx = len(repetitions) + 1
            if operation == "write" and run_metrics["changed_files"] != files:
                return {
                    "formatter": tool.name,
                    "status": "failed",
                    "invocation": invocation,
                    "operation": operation,
                    "command": display_command(tool, invocation, operation),
                    "reason": (
                        f"repetition {measured_idx} changed "
                        f"{run_metrics['changed_files']} of {files} files"
                    ),
                }
            if operation == "check" and run_metrics["changed_files"] != 0:
                return {
                    "formatter": tool.name,
                    "status": "failed",
                    "invocation": invocation,
                    "operation": operation,
                    "command": display_command(tool, invocation, operation),
                    "reason": (
                        f"repetition {measured_idx} wrote "
                        f"{run_metrics['changed_files']} files in check mode"
                    ),
                }
            if metrics is None:
                metrics = run_metrics
            elif run_metrics != metrics:
                return {
                    "formatter": tool.name,
                    "status": "failed",
                    "invocation": invocation,
                    "operation": operation,
                    "command": display_command(tool, invocation, operation),
                    "reason": (
                        f"repetition {measured_idx} output metrics differ "
                        "from repetition 1"
                    ),
                }
            repetitions.append(elapsed)
            user_repetitions.append(usage_after[0] - usage_before[0])
            sys_repetitions.append(usage_after[1] - usage_before[1])

    median_seconds = statistics.median(repetitions)
    mean_seconds = statistics.fmean(repetitions)
    assert metrics is not None
    return {
        "formatter": tool.name,
        "status": "ok",
        "command": display_command(tool, invocation, operation),
        "invocation": invocation,
        "operation": operation,
        "width_profile": width_profile,
        "files": files,
        "bytes": corpus_bytes,
        "output_files": metrics["output_files"],
        "output_bytes": metrics["output_bytes"],
        "changed_files": metrics["changed_files"],
        "would_change_files": files if operation == "check" and proc.returncode != 0 else 0,
        "output_hash": metrics["output_hash"],
        "warmups": warmups,
        "reps": reps,
        "repetitions": repetitions,
        "user_seconds": user_repetitions,
        "sys_seconds": sys_repetitions,
        "median_seconds": median_seconds,
        "mean_seconds": mean_seconds,
        "median_user_seconds": statistics.median(user_repetitions),
        "median_sys_seconds": statistics.median(sys_repetitions),
        "mb_per_second": corpus_bytes / 1_000_000 / median_seconds,
    }


def run_invocation(
    tool: Tool,
    run_root: Path,
    target_relatives: list[Path],
    invocation: str,
    operation: str,
    cwd: Path,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    assert invocation in INVOCATIONS
    assert operation in OPERATIONS
    if invocation == "directory":
        command = materialize_command(
            tool_command(tool, invocation, operation),
            target=run_root,
            root=run_root,
        )
        return subprocess.run(
            command,
            cwd=cwd,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    stdout = []
    stderr = []
    returncode = 0
    for relative in target_relatives:
        command = materialize_command(
            tool_command(tool, invocation, operation),
            target=run_root / relative,
            root=run_root,
        )
        proc = subprocess.run(
            command,
            cwd=cwd,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        stdout.append(proc.stdout)
        stderr.append(proc.stderr)
        if operation == "write" and proc.returncode != 0:
            return subprocess.CompletedProcess(
                args=command,
                returncode=proc.returncode,
                stdout="".join(stdout),
                stderr="".join(stderr),
            )
        if operation == "check":
            if proc.returncode not in (0, 1):
                return subprocess.CompletedProcess(
                    args=command,
                    returncode=proc.returncode,
                    stdout="".join(stdout),
                    stderr="".join(stderr),
                )
            returncode = max(returncode, proc.returncode)
    return subprocess.CompletedProcess(
        args=[],
        returncode=returncode,
        stdout="".join(stdout),
        stderr="".join(stderr),
    )


def display_command(tool: Tool, invocation: str, operation: str) -> str:
    if invocation == "directory":
        command = materialize_command(
            tool_command(tool, invocation, operation),
            target=Path("ROOT"),
            root=Path("ROOT"),
        )
    else:
        command = materialize_command(
            tool_command(tool, invocation, operation),
            target=Path("TARGET"),
            root=Path("ROOT"),
        )
    return shlex.join(command)


def tool_command(tool: Tool, invocation: str, operation: str) -> list[str]:
    assert operation in OPERATIONS
    if operation == "check":
        if invocation == "directory" and tool.check_directory_command is not None:
            return tool.check_directory_command
        if tool.check_command is not None:
            return tool.check_command
        raise SystemExit(f"{tool.name} does not support --operation check")
    if invocation == "directory" and tool.directory_command is not None:
        return tool.directory_command
    return tool.command


def output_metrics(corpus_dir: Path, run_root: Path) -> dict[str, object]:
    output_files = 0
    output_bytes = 0
    changed_files = 0
    digest = hashlib.sha256()

    for source in sorted(corpus_dir.rglob("*.yaml")):
        relative = source.relative_to(corpus_dir)
        output = run_root / relative
        if not output.exists():
            raise AssertionError(f"formatter removed YAML file: {relative}")
        before = source.read_bytes()
        after = output.read_bytes()
        output_files += 1
        output_bytes += len(after)
        changed_files += int(before != after)
        digest.update(relative.as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(after)
        digest.update(b"\0")

    return {
        "output_files": output_files,
        "output_bytes": output_bytes,
        "changed_files": changed_files,
        "output_hash": digest.hexdigest(),
    }


def command_available(command: list[str]) -> bool:
    try:
        proc = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError:
        return False
    return proc.returncode == 0


def tool_version(tool: Tool) -> str | None:
    if tool.version is not None:
        return tool.version
    if tool.version_command is None:
        return None
    try:
        proc = subprocess.run(
            tool.version_command,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError:
        return None
    if proc.returncode != 0:
        return None
    return version_line(proc.stdout + proc.stderr)


def version_line(output: str) -> str | None:
    # First line carrying a digit, so banner rules (yamlfix) are skipped.
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    for line in lines:
        if any(ch.isdigit() for ch in line):
            return line
    return lines[0] if lines else None


def yamark_cargo_version() -> str:
    for line in (ROOT / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("version"):
            return "yamark " + line.split('"')[1]
    raise SystemExit("could not find yamark version in Cargo.toml")


def prepare_tool(tool: Tool, work_root: Path) -> str | None:
    if tool.prepare == "pretty-yaml-driver":
        reason = prepare_pretty_yaml_driver()
        if reason is not None:
            return reason
    elif tool.prepare is not None:
        raise AssertionError(f"unsupported tool preparation: {tool.prepare}")

    if tool.prepare_command is None:
        return None

    try:
        proc = subprocess.run(
            tool.prepare_command,
            cwd=ROOT,
            env=tool_env(tool, work_root),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError:
        return f"{tool.prepare_command[0]} is not available on PATH"
    if proc.returncode != 0:
        return f"prepare command failed: {stderr_tail(proc.stderr)}"
    return None


def tool_cache_dir(tool: Tool, work_root: Path) -> Path:
    return work_root / tool.name / "cache"


def tool_env(tool: Tool, work_root: Path) -> dict[str, str] | None:
    if tool.cache_env is None:
        return None
    cache_dir = tool_cache_dir(tool, work_root)
    cache_dir.mkdir(parents=True, exist_ok=True)
    return {**os.environ, tool.cache_env: str(cache_dir)}


def wipe_tool_cache(tool: Tool, work_root: Path) -> None:
    if tool.cache_env is None:
        return
    cache_dir = tool_cache_dir(tool, work_root)
    if not cache_dir.exists():
        return
    for entry in cache_dir.iterdir():
        if entry.name in tool.cache_keep:
            continue
        if entry.is_dir() and not entry.is_symlink():
            shutil.rmtree(entry)
        else:
            entry.unlink()


def pretty_yaml_driver_path() -> Path:
    if override := os.environ.get(PRETTY_YAML_DRIVER_ENV):
        return Path(override)
    return PRETTY_YAML_DRIVER_TARGET / "release" / "format-pretty-yaml"


def prepare_pretty_yaml_driver() -> str | None:
    if os.environ.get(PRETTY_YAML_DRIVER_ENV):
        return None
    if shutil.which("cargo") is None:
        return "cargo is not available on PATH"

    src_dir = PRETTY_YAML_DRIVER_DIR / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    (PRETTY_YAML_DRIVER_DIR / "Cargo.toml").write_text(
        textwrap.dedent(
            """\
            [package]
            name = "format-pretty-yaml"
            version = "0.0.0"
            edition = "2024"

            [dependencies]
            pretty_yaml = "{PRETTY_YAML_CRATE_VERSION}"
            """
        ).replace("{PRETTY_YAML_CRATE_VERSION}", PRETTY_YAML_CRATE_VERSION),
        encoding="utf-8",
    )
    (src_dir / "main.rs").write_text(
        textwrap.dedent(
            """\
            use pretty_yaml::{config::FormatOptions, format_text};
            use std::env;
            use std::error::Error;
            use std::fs;
            use std::path::Path;

            fn main() -> Result<(), Box<dyn Error>> {
                let root = env::args().nth(1).expect("missing root path");
                format_root(Path::new(&root))
            }

            fn format_root(path: &Path) -> Result<(), Box<dyn Error>> {
                assert!(path.exists(), "root path does not exist");
                visit(path)
            }

            fn visit(path: &Path) -> Result<(), Box<dyn Error>> {
                if path.is_dir() {
                    for entry in fs::read_dir(path)? {
                        visit(&entry?.path())?;
                    }
                    return Ok(());
                }
                if !is_yaml(path) {
                    return Ok(());
                }
                let input = fs::read_to_string(path)?;
                let output = format_text(&input, &FormatOptions::default())?;
                fs::write(path, output)?;
                Ok(())
            }

            fn is_yaml(path: &Path) -> bool {
                matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("yaml" | "yml")
                )
            }
            """
        ),
        encoding="utf-8",
    )

    proc = subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            str(PRETTY_YAML_DRIVER_DIR / "Cargo.toml"),
            "--target-dir",
            str(PRETTY_YAML_DRIVER_TARGET),
        ],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        return f"pretty-yaml driver build failed: {stderr_tail(proc.stderr)}"
    return None


def run_probe(tool: Tool, work_root: Path) -> str | None:
    probe_root = work_root / tool.name / "probe"
    if probe_root.exists():
        shutil.rmtree(probe_root)
    probe_root.mkdir(parents=True)
    path = probe_root / "probe.yaml"
    path.write_text(PROBE_YAML, encoding="utf-8")
    command = materialize_command(tool.command, target=path, root=probe_root)
    try:
        proc = subprocess.run(
            command,
            cwd=probe_root if tool.cwd == "work" else ROOT,
            env=tool_env(tool, work_root),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except FileNotFoundError:
        return f"{command[0]} is not available on PATH"
    if proc.returncode != 0:
        return f"probe command failed: {stderr_tail(proc.stderr)}"
    if path.read_text(encoding="utf-8") == PROBE_YAML:
        return "probe file was not modified"
    return None


def materialize_command(command: list[str], *, target: Path, root: Path) -> list[str]:
    out = []
    for part in command:
        if part == "{target}":
            out.append(str(target))
        elif part == "{root}":
            out.append(str(root))
        else:
            out.append(part)
    return out


def stderr_tail(stderr: str) -> str:
    lines = [line for line in stderr.splitlines() if line.strip()]
    return "\n".join(lines[-8:]) if lines else "command failed"


def report_line(line: str, args: argparse.Namespace, log_lines: list[str]) -> None:
    if args.agent_summary:
        log_lines.append(line)
    else:
        print(line)


def format_result(result: dict[str, object]) -> str:
    formatter = result["formatter"]
    status = result["status"]
    if status == "ok":
        return (
            f"{formatter}: {result['median_seconds']:.4f}s median, "
            f"{result['mb_per_second']:.2f} MB/s"
        )
    return f"{formatter}: {status} ({result.get('reason', '')})"


def agent_log_path(out_dir: Path, run_id: str) -> Path:
    return out_dir / "logs" / f"{run_id}.log"


def write_agent_log(path: Path, lines: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def print_agent_summary(
    *,
    artifact_dir: Path,
    artifact: dict[str, object],
    artifact_path: Path,
    results: list[dict[str, object]],
    log_path: Path,
) -> int:
    failed = [result for result in results if result.get("status") == "failed"]
    if failed:
        print("status: failed")
        print(f"artifact: {display_path(artifact_path)}")
        print(f"failure: {format_agent_failure(failed[0])}")
        print(f"log: {display_path(log_path)}")
        for line in nonempty_tail(log_path, AGENT_LOG_TAIL_LINES):
            print(f"tail: {line}")
        return 1

    current = next(
        (result for result in results if result.get("status") == "ok"),
        None,
    )
    if current is None:
        print("status: skipped")
        print(f"artifact: {display_path(artifact_path)}")
        print("current: none")
        print("previous: none")
        print("delta: n/a")
        print(f"log: {display_path(log_path)}")
        return 0

    previous = previous_benchmark_result(artifact_dir, artifact, current)
    print("status: ok")
    print(f"artifact: {display_path(artifact_path)}")
    print(f"current: {format_agent_result(artifact, current)}")
    if previous is None:
        print("previous: none")
        print("delta: n/a")
    else:
        previous_artifact, previous_result = previous
        print(f"previous: {format_agent_result(previous_artifact, previous_result)}")
        print(f"delta: {format_agent_delta(current, previous_result)}")
    print(f"log: {display_path(log_path)}")
    return 0


def format_agent_failure(result: dict[str, object]) -> str:
    reason = str(result.get("reason", "")).splitlines()
    reason_text = reason[-1] if reason else "command failed"
    return f"{result.get('formatter', '')} {reason_text}"


def previous_benchmark_result(
    artifact_dir: Path,
    current_artifact: dict[str, object],
    current_result: dict[str, object],
) -> tuple[dict[str, object], dict[str, object]] | None:
    current_commit = current_artifact["git"]["commit"]
    candidates: list[tuple[str, dict[str, object], dict[str, object]]] = []
    for path in sorted(artifact_dir.glob("*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        if data.get("benchmark") != "yaml-formatting":
            continue
        if data.get("corpus") != current_artifact.get("corpus"):
            continue
        git = data.get("git", {})
        if git.get("commit") == current_commit:
            continue
        for result in data.get("results", []):
            if result.get("status") != "ok":
                continue
            if result_key(result) != result_key(current_result):
                continue
            candidates.append((git.get("commit_time", ""), data, result))
    if not candidates:
        return None
    candidates.sort(key=lambda candidate: candidate[0], reverse=True)
    return (candidates[0][1], candidates[0][2])


def format_agent_result(
    artifact: dict[str, object],
    result: dict[str, object],
) -> str:
    git = artifact["git"]
    dirty = str(bool(git.get("dirty"))).lower()
    return (
        f"{git['short_commit']} dirty={dirty} "
        f"formatter={result.get('formatter', '')} "
        f"invocation={result.get('invocation', '')} "
        f"operation={result.get('operation', 'write')} "
        f"width_profile={result.get('width_profile', 'default')} "
        f"median={result['median_seconds']:.4f}s "
        f"mbps={result['mb_per_second']:.3f}"
    )


def format_agent_delta(
    current: dict[str, object],
    previous: dict[str, object],
) -> str:
    current_seconds = float(current["median_seconds"])
    previous_seconds = float(previous["median_seconds"])
    seconds = current_seconds - previous_seconds
    if previous_seconds == 0:
        return f"{seconds:+.4f}s"
    percent = seconds / previous_seconds * 100
    return f"{seconds:+.4f}s ({percent:+.1f}%)"


def nonempty_tail(path: Path, lines: int) -> list[str]:
    text = path.read_text(encoding="utf-8")
    nonempty = [line for line in text.splitlines() if line.strip()]
    return nonempty[-lines:]


def benchmark_document(
    *,
    run_id: str,
    args: argparse.Namespace,
    files: int,
    corpus_bytes: int,
    target_relatives: list[Path],
    invocation: str,
    operation: str,
    width_profile: str,
    selected: list[str],
    tool_versions: dict[str, str | None],
    results: list[dict[str, object]],
) -> dict[str, object]:
    git = git_metadata()
    return {
        "schema_version": 1,
        "benchmark": "yaml-formatting",
        "run_id": run_id,
        "created_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "git": git,
        "host": {
            "system": platform.system(),
            "machine": platform.machine(),
            "cpu": cpu_brand(),
            "python": platform.python_version(),
        },
        "corpus": {
            "kind": args.corpus,
            "shape": args.corpus_shape,
            "files": files,
            "items_per_file": args.items,
            "bytes": corpus_bytes,
        },
        "invocation": {
            "unit": invocation,
            "targets": [path.as_posix() for path in target_relatives],
        },
        "operation": operation,
        "formatting_options": {
            "width_profile": width_profile,
            "yamark_options": yamark_width_options(width_profile),
        },
        "selected_formatters": selected,
        "tool_versions": tool_versions,
        "results": results,
    }


def cpu_brand() -> str | None:
    if platform.system() != "Darwin":
        return platform.processor() or None
    proc = subprocess.run(
        ["sysctl", "-n", "machdep.cpu.brand_string"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        return None
    return proc.stdout.strip() or None


def git_metadata() -> dict[str, object]:
    status = git_output(["status", "--porcelain=v1"])
    commit = git_output(["rev-parse", "HEAD"])
    return {
        "commit": commit,
        "short_commit": git_output(["rev-parse", "--short=12", "HEAD"]),
        "commit_time": git_output(["show", "-s", "--format=%cI", "HEAD"]),
        "tree": git_output(["rev-parse", "HEAD^{tree}"]),
        "dirty": bool(status),
    }


def git_output(args: list[str]) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        raise SystemExit(f"git {' '.join(args)} failed: {proc.stderr.strip()}")
    return proc.stdout.strip()


def write_artifact(artifact_dir: Path, artifact: dict[str, object]) -> Path:
    artifact_dir.mkdir(parents=True, exist_ok=True)
    path = artifact_dir / artifact_file_name(artifact)
    if path.exists():
        artifact = merge_artifact(json.loads(path.read_text(encoding="utf-8")), artifact)
    path.write_text(
        json.dumps(artifact, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return path


def artifact_file_name(artifact: dict[str, object]) -> str:
    commit = artifact["git"]["commit"]
    shape = artifact["corpus"]["shape"]
    if shape == DEFAULT_CORPUS_SHAPE:
        return f"{commit}.json"
    return f"{commit}-{shape}.json"


def merge_artifact(existing: dict[str, object], new: dict[str, object]) -> dict[str, object]:
    if existing.get("benchmark") != new.get("benchmark"):
        raise SystemExit("existing benchmark artifact has a different benchmark name")
    if existing.get("git", {}).get("commit") != new.get("git", {}).get("commit"):
        raise SystemExit("existing benchmark artifact has a different git commit")
    if existing.get("corpus") != new.get("corpus"):
        raise SystemExit("existing benchmark artifact has different corpus settings")

    merged = copy.deepcopy(existing)
    new_results = list(new["results"])
    replacement_keys = {
        result_key(row)
        for row in new_results
    }
    old_results = [
        row for row in merged.get("results", [])
        if result_key(row) not in replacement_keys
    ]
    merged["results"] = sorted(
        [*old_results, *new_results],
        key=lambda row: (row.get("invocation", ""), row.get("formatter", "")),
    )
    merged["run_id"] = new["run_id"]
    merged["created_at"] = new["created_at"]
    merged["selected_formatters"] = sorted(
        set(merged.get("selected_formatters", []))
        | set(new.get("selected_formatters", []))
    )
    merged["tool_versions"] = {
        **merged.get("tool_versions", {}),
        **new.get("tool_versions", {}),
    }
    merged["invocation"] = {
        "units": sorted(
            {
                row.get("invocation", "")
                for row in merged["results"]
                if row.get("invocation")
            }
        ),
        "targets": new["invocation"]["targets"],
    }
    merged["operation"] = {
        "values": sorted(
            {
                row.get("operation", "")
                for row in merged["results"]
                if row.get("operation")
            }
        )
    }
    merged["formatting_options"] = {
        "width_profiles": sorted(
            {
                row.get("width_profile", "default")
                for row in merged["results"]
                if row.get("width_profile")
            }
        )
    }
    return merged


def result_key(row: dict[str, object]) -> tuple[object, object, object, object]:
    return (
        row.get("formatter"),
        row.get("invocation", ""),
        row.get("operation", "write"),
        row.get("width_profile", "default"),
    )


def display_path(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT))
    except ValueError:
        return str(path)


if __name__ == "__main__":
    sys.exit(main())
