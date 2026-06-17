#!/usr/bin/env python3

from __future__ import annotations

import argparse
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
GENERATOR = BENCH_DIR / "big.R"
# Native formatter CLIs only, each invoked the way its docs say to format a
# file, with no formatting options. Lint fixers (pymarkdown,
# markdownlint-cli2) and library shims (panache-yaml, pretty-yaml, py-yaml12)
# stay available via --tools but are not part of the default comparison.
DEFAULT_TOOLS = (
    "yamark,panache,mdformat,prettier,dprint-markdown,deno-fmt,"
    "yamlfmt,yamlfix,dprint-yaml"
)
# 4 MB keeps every comparison tool in play: panache refuses inputs larger
# than 4 MiB (PreallocationSizeLimit).
DEFAULT_MARKDOWN_BYTES = 4_000_000
DEFAULT_YAML_BYTES = 4_000_000
DEFAULT_FRONTMATTER_BYTES = 4_000_000
DEFAULT_FRONTMATTER_YAML_RATIO = 0.05
DEFAULT_SEED = 20260602
DEFAULT_REPS = 10
DEFAULT_WARMUPS = 2
TARGETS = (
    "big.md",
    "big.yaml",
    "big-with-frontmatter.md",
)
YAML_SUFFIXES = (".yaml", ".yml")
MARKDOWN_SUFFIXES = (".md", ".qmd", ".rmd", ".Rmd")
DPRINT_YAML_CONFIG = BENCH_DIR / "dprint-yaml.json"
DPRINT_MARKDOWN_CONFIG = BENCH_DIR / "dprint-markdown.json"
PRETTY_YAML_DRIVER_ENV = "YAMARK_PRETTY_YAML_DRIVER"
PRETTY_YAML_DRIVER_DIR = ROOT / "target" / "bench-tools" / "pretty-yaml-driver"
PRETTY_YAML_DRIVER_TARGET = ROOT / "target" / "bench-tools" / "pretty-yaml-driver-target"
PRETTY_YAML_CRATE_VERSION = "0.6.0"


@dataclass(frozen=True)
class Tool:
    name: str
    command: list[str]
    suffixes: tuple[str, ...]
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


@dataclass(frozen=True)
class MeasuredProcess:
    proc: subprocess.CompletedProcess[str]
    user_seconds: float
    sys_seconds: float
    max_rss_bytes: int


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate and benchmark the big single-file corpora."
    )
    parser.add_argument("--target-bytes", type=int, default=None)
    parser.add_argument("--markdown-bytes", type=int, default=DEFAULT_MARKDOWN_BYTES)
    parser.add_argument("--yaml-bytes", type=int, default=DEFAULT_YAML_BYTES)
    parser.add_argument("--frontmatter-bytes", type=int, default=DEFAULT_FRONTMATTER_BYTES)
    parser.add_argument(
        "--frontmatter-yaml-bytes",
        type=int,
        default=None,
        help="Size of the front matter block within the frontmatter target.",
    )
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--reps", type=int, default=DEFAULT_REPS)
    parser.add_argument("--warmups", type=int, default=DEFAULT_WARMUPS)
    parser.add_argument("--tools", default=DEFAULT_TOOLS)
    parser.add_argument("--python", default=sys.executable)
    parser.add_argument("--rscript", default="Rscript")
    parser.add_argument("--dprint-bin", default="dprint")
    parser.add_argument("--yamark-bin", type=Path, default=ROOT / "target" / "release" / "yamark")
    parser.add_argument("--skip-yamark-build", action="store_true")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT / "target" / "bench-big",
    )
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        default=ROOT / "docs" / "benchmarks" / "big",
    )
    parser.add_argument("--keep-corpus", action="store_true")
    parser.add_argument("--keep-workdirs", action="store_true")
    args = parser.parse_args()

    if args.target_bytes is not None:
        args.markdown_bytes = args.target_bytes
        args.yaml_bytes = args.target_bytes
        args.frontmatter_bytes = args.target_bytes
    if args.frontmatter_yaml_bytes is None:
        args.frontmatter_yaml_bytes = default_frontmatter_yaml_bytes(
            args.frontmatter_bytes
        )

    return run_big_benchmark(args)


def default_frontmatter_yaml_bytes(frontmatter_bytes: int) -> int:
    # 5% of bytes lands at roughly a third of the document's lines in YAML,
    # since generated YAML lines are much shorter than generated Markdown
    # prose lines.
    return max(
        1,
        min(
            frontmatter_bytes - 1,
            int(round(frontmatter_bytes * DEFAULT_FRONTMATTER_YAML_RATIO)),
        ),
    )


def run_big_benchmark(args: argparse.Namespace) -> int:
    stopifnot(args.markdown_bytes > 0, "--markdown-bytes must be positive")
    stopifnot(args.yaml_bytes > 0, "--yaml-bytes must be positive")
    stopifnot(args.frontmatter_bytes > 0, "--frontmatter-bytes must be positive")
    stopifnot(
        0 < args.frontmatter_yaml_bytes < args.frontmatter_bytes,
        "--frontmatter-yaml-bytes must be positive and below --frontmatter-bytes",
    )
    stopifnot(args.reps > 0, "--reps must be positive")
    stopifnot(args.warmups >= 0, "--warmups must be >= 0")

    selected = selected_tools(args.tools)
    tools = tool_specs(args)
    unknown = [name for name in selected if name not in tools]
    if unknown:
        raise SystemExit(f"unknown formatter: {', '.join(unknown)}")

    if "yamark" in selected and not args.skip_yamark_build:
        run_checked(["cargo", "build", "--release", "--bin", "yamark"], cwd=ROOT)

    tool_versions = {name: tool_version(tools[name]) for name in selected}

    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    run_dir = args.out_dir / run_id
    corpus_dir = run_dir / "corpus"
    generate_corpus(args, corpus_dir)
    target_relatives = [Path(target) for target in TARGETS]
    target_bytes = {
        target.as_posix(): (corpus_dir / target).stat().st_size
        for target in target_relatives
    }
    work_root: Path | None = None

    try:
        work_root = Path(
            tempfile.mkdtemp(prefix=f"yamark-big-bench-{run_id}-", dir="/tmp")
        ).resolve()
        prepared = prepare_tools(selected, tools, work_root)
        results: list[dict[str, object]] = []
        for relative in target_relatives:
            for name in selected:
                tool = tools[name]
                result = run_tool_on_file(
                    tool=tool,
                    source=corpus_dir / relative,
                    relative=relative,
                    work_root=work_root,
                    reps=args.reps,
                    warmups=args.warmups,
                    prepared_reason=prepared.get(name),
                )
                results.append(result)

        artifact = benchmark_document(
            run_id=run_id,
            args=args,
            target_relatives=target_relatives,
            target_bytes=target_bytes,
            selected=selected,
            tool_versions=tool_versions,
            results=results,
        )
        artifact_path = write_artifact(args.artifact_dir, artifact)
        print_table(results)
        print(f"\nartifact: {display_path(artifact_path)}")
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


def tool_specs(args: argparse.Namespace) -> dict[str, Tool]:
    yamark_bin = str(args.yamark_bin)
    python = str(args.python)
    dprint_bin = str(args.dprint_bin)
    return {
        "yamark": Tool(
            name="yamark",
            command=[yamark_bin, "format", "{target}"],
            suffixes=(*YAML_SUFFIXES, *MARKDOWN_SUFFIXES),
            version=yamark_cargo_version(),
        ),
        "yamlfmt": Tool(
            name="yamlfmt",
            command=["yamlfmt", "{target}"],
            suffixes=YAML_SUFFIXES,
            availability=["yamlfmt", "--version"],
            version_command=["yamlfmt", "--version"],
        ),
        "prettier": Tool(
            name="prettier",
            command=["prettier", "--write", "{target}"],
            suffixes=(*YAML_SUFFIXES, *MARKDOWN_SUFFIXES),
            availability=["prettier", "--version"],
            version_command=["prettier", "--version"],
        ),
        "panache": Tool(
            name="panache",
            command=["panache", "format", "{target}"],
            suffixes=MARKDOWN_SUFFIXES,
            availability=["panache", "--version"],
            version_command=["panache", "--version"],
            # Plain `panache format` with the format cache redirected and
            # wiped between repetitions, so every timed run formats from
            # scratch instead of replaying a cached result.
            cache_env="PANACHE_CACHE_DIR",
        ),
        "mdformat": Tool(
            name="mdformat",
            command=["mdformat", "{target}"],
            suffixes=(".md",),
            availability=["mdformat", "--version"],
            version_command=["mdformat", "--version"],
        ),
        "pymarkdown": Tool(
            name="pymarkdown",
            command=["pymarkdown", "fix", "{target}"],
            suffixes=(".md",),
            availability=["pymarkdown", "--version"],
            version_command=["pymarkdown", "--version"],
        ),
        "markdownlint-cli2": Tool(
            name="markdownlint-cli2",
            command=["markdownlint-cli2", "--fix", "{target}"],
            suffixes=(".md",),
            availability=["markdownlint-cli2", "--version"],
            version_command=["markdownlint-cli2", "--version"],
        ),
        "dprint-markdown": Tool(
            name="dprint-markdown",
            # dprint has no default plugins; the config file names the
            # Markdown plugin and nothing else. The cache (plugins plus
            # incremental state) is redirected and the incremental state is
            # wiped between repetitions.
            command=[
                dprint_bin,
                "fmt",
                "--config",
                str(DPRINT_MARKDOWN_CONFIG),
                "{target}",
            ],
            suffixes=(".md",),
            availability=[dprint_bin, "--version"],
            version_command=[dprint_bin, "--version"],
            prepare_command=[
                dprint_bin,
                "output-resolved-config",
                "--config",
                str(DPRINT_MARKDOWN_CONFIG),
            ],
            cwd="work",
            cache_env="DPRINT_CACHE_DIR",
            cache_keep=("plugins", "plugin-cache-manifest.json", "locks"),
        ),
        "yamlfix": Tool(
            name="yamlfix",
            command=["yamlfix", "{target}"],
            suffixes=YAML_SUFFIXES,
            availability=["yamlfix", "--version"],
            version_command=["yamlfix", "--version"],
        ),
        "panache-yaml": Tool(
            name="panache-yaml",
            command=[python, str(BENCH_DIR / "format_panache_yaml.py"), "{target}"],
            suffixes=YAML_SUFFIXES,
            availability=["panache", "--version"],
            version_command=["panache", "--version"],
        ),
        "dprint-yaml": Tool(
            name="dprint-yaml",
            command=[
                dprint_bin,
                "fmt",
                "--config",
                str(DPRINT_YAML_CONFIG),
                "{target}",
            ],
            suffixes=YAML_SUFFIXES,
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
            suffixes=(*YAML_SUFFIXES, *MARKDOWN_SUFFIXES),
            availability=["deno", "--version"],
            version_command=["deno", "--version"],
        ),
        "pretty-yaml": Tool(
            name="pretty-yaml",
            command=[str(pretty_yaml_driver_path()), "{target}"],
            suffixes=YAML_SUFFIXES,
            prepare="pretty-yaml-driver",
            version=f"pretty_yaml crate {PRETTY_YAML_CRATE_VERSION}",
        ),
        "py-yaml12": Tool(
            name="py-yaml12",
            command=[python, str(BENCH_DIR / "format_yaml12.py"), "{target}"],
            suffixes=YAML_SUFFIXES,
            availability=[python, "-c", "import yaml12"],
            version_command=[
                python,
                "-c",
                "import yaml12; print('yaml12', getattr(yaml12, '__version__', ''))",
            ],
        ),
    }


def generate_corpus(args: argparse.Namespace, corpus_dir: Path) -> None:
    if corpus_dir.exists():
        shutil.rmtree(corpus_dir)
    corpus_dir.mkdir(parents=True)
    command = [
        str(args.rscript),
        str(GENERATOR),
        "--out-dir",
        str(corpus_dir),
        "--markdown-bytes",
        str(args.markdown_bytes),
        "--yaml-bytes",
        str(args.yaml_bytes),
        "--frontmatter-bytes",
        str(args.frontmatter_bytes),
        "--frontmatter-yaml-bytes",
        str(args.frontmatter_yaml_bytes),
        "--seed",
        str(args.seed),
    ]
    run_checked(command, cwd=ROOT)


def run_checked(command: list[str], *, cwd: Path) -> None:
    proc = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        raise SystemExit(f"{shlex.join(command)} failed\n{proc.stdout}{proc.stderr}")


def prepare_tools(
    selected: list[str],
    tools: dict[str, Tool],
    work_root: Path,
) -> dict[str, str | None]:
    prepared = {}
    for name in selected:
        tool = tools[name]
        reason = unavailable_reason(tool)
        if reason is None:
            reason = prepare_tool(tool, work_root)
        if reason is None:
            reason = run_probe(tool, work_root)
        prepared[name] = reason
    return prepared


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


def unavailable_reason(tool: Tool) -> str | None:
    if tool.availability is None:
        return None
    if command_available(tool.availability):
        return None
    return f"{tool.availability[0]} is not available on PATH"


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
                let path = Path::new(&root);
                let input = fs::read_to_string(path)?;
                let output = format_text(&input, &FormatOptions::default())?;
                fs::write(path, output)?;
                Ok(())
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
    suffix = ".yaml" if ".yaml" in tool.suffixes else ".md"
    probe_root = work_root / tool.name / "probe"
    if probe_root.exists():
        shutil.rmtree(probe_root)
    probe_root.mkdir(parents=True)
    path = probe_root / f"probe{suffix}"
    path.write_text(probe_source(suffix), encoding="utf-8")
    command = materialize_command(tool.command, target=path)
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
    return None


def probe_source(suffix: str) -> str:
    if suffix in YAML_SUFFIXES:
        return "probe:    [one,two]\n"
    return "This paragraph is long enough that the markdown formatter should wrap it after the default width when the probe runs.\n"


def run_tool_on_file(
    *,
    tool: Tool,
    source: Path,
    relative: Path,
    work_root: Path,
    reps: int,
    warmups: int,
    prepared_reason: str | None,
) -> dict[str, object]:
    base = base_result(tool, relative, source)
    if relative.suffix not in tool.suffixes:
        return {
            **base,
            "status": "skipped",
            "reason": f"{tool.name} does not support {relative.suffix} inputs",
        }
    if prepared_reason is not None:
        return {
            **base,
            "status": "skipped",
            "reason": prepared_reason,
        }

    repetitions: list[float] = []
    user_repetitions: list[float] = []
    sys_repetitions: list[float] = []
    peak_rss_repetitions: list[int] = []
    metrics = None

    for run_idx in range(warmups + reps):
        run_root = work_root / tool.name / relative.name / f"run-{run_idx:02d}"
        if run_root.exists():
            shutil.rmtree(run_root)
        run_root.mkdir(parents=True)
        target = run_root / relative.name
        shutil.copy2(source, target)
        wipe_tool_cache(tool, work_root)

        started = time.perf_counter()
        measured = run_measured(
            materialize_command(tool.command, target=target),
            cwd=run_root if tool.cwd == "work" else ROOT,
            env=tool_env(tool, work_root),
        )
        proc = measured.proc
        elapsed = time.perf_counter() - started
        if proc.returncode != 0:
            return {
                **base,
                "status": "failed",
                "command": display_command(tool),
                "exit_code": proc.returncode,
                "reason": stderr_tail(proc.stderr),
            }

        if run_idx >= warmups:
            run_metrics = output_metrics(source, target)
            measured_idx = len(repetitions) + 1
            if metrics is None:
                metrics = run_metrics
            elif run_metrics != metrics:
                return {
                    **base,
                    "status": "failed",
                    "command": display_command(tool),
                    "reason": (
                        f"repetition {measured_idx} output metrics differ "
                        "from repetition 1"
                    ),
                }
            repetitions.append(elapsed)
            user_repetitions.append(measured.user_seconds)
            sys_repetitions.append(measured.sys_seconds)
            peak_rss_repetitions.append(measured.max_rss_bytes)

    assert metrics is not None
    median_seconds = statistics.median(repetitions)
    median_peak_rss_bytes = int(statistics.median(peak_rss_repetitions))
    input_bytes = int(base["bytes"])
    front_matter = (
        {"front_matter": metrics["front_matter"]}
        if "front_matter" in metrics
        else {}
    )
    return {
        **base,
        "status": "ok",
        "command": display_command(tool),
        "output_bytes": metrics["output_bytes"],
        "changed": metrics["changed"],
        "output_hash": metrics["output_hash"],
        **front_matter,
        "warmups": warmups,
        "reps": reps,
        "repetitions": repetitions,
        "user_seconds": user_repetitions,
        "sys_seconds": sys_repetitions,
        "peak_rss_bytes": peak_rss_repetitions,
        "median_seconds": median_seconds,
        "mean_seconds": statistics.fmean(repetitions),
        "median_user_seconds": statistics.median(user_repetitions),
        "median_sys_seconds": statistics.median(sys_repetitions),
        "median_peak_rss_bytes": median_peak_rss_bytes,
        "mb_per_second": input_bytes / 1_000_000 / median_seconds,
    }


def base_result(tool: Tool, relative: Path, source: Path) -> dict[str, object]:
    return {
        "file": relative.as_posix(),
        "formatter": tool.name,
        "bytes": source.stat().st_size,
    }


def run_measured(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
) -> MeasuredProcess:
    stopifnot(resource is not None, "resource module is required for benchmark RSS")
    stopifnot(hasattr(os, "wait4"), "os.wait4 is required for benchmark RSS")
    with tempfile.TemporaryFile() as stdout_file, tempfile.TemporaryFile() as stderr_file:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdout=stdout_file,
            stderr=stderr_file,
        )
        _pid, status, usage = os.wait4(process.pid, 0)
        process.returncode = os.waitstatus_to_exitcode(status)
        stdout_file.seek(0)
        stderr_file.seek(0)
        proc = subprocess.CompletedProcess(
            args=command,
            returncode=process.returncode,
            stdout=stdout_file.read().decode("utf-8", errors="replace"),
            stderr=stderr_file.read().decode("utf-8", errors="replace"),
        )
    return MeasuredProcess(
        proc=proc,
        user_seconds=float(usage.ru_utime),
        sys_seconds=float(usage.ru_stime),
        max_rss_bytes=max_rss_bytes(usage),
    )


def max_rss_bytes(usage: object) -> int:
    rss = int(getattr(usage, "ru_maxrss"))
    if platform.system() == "Darwin":
        return rss
    return rss * 1024


def output_metrics(source: Path, output: Path) -> dict[str, object]:
    before = source.read_bytes()
    after = output.read_bytes()
    digest = hashlib.sha256()
    digest.update(after)
    metrics: dict[str, object] = {
        "output_bytes": len(after),
        "changed": before != after,
        "output_hash": digest.hexdigest(),
    }
    outcome = front_matter_outcome(before, after)
    if outcome is not None:
        metrics["front_matter"] = outcome
    return metrics


def front_matter_outcome(before: bytes, after: bytes) -> str | None:
    # The corpus front matter is deliberately unformatted, so a tool that
    # formats front matter must rewrite the block. Trailing-whitespace
    # trimming alone does not count as rewriting, and an output without a
    # parseable front matter block means the tool broke the document.
    before_block = front_matter_block(before)
    if before_block is None:
        return None
    after_block = front_matter_block(after)
    if after_block is None:
        return "removed"
    if strip_trailing_ws(before_block) != strip_trailing_ws(after_block):
        return "rewritten"
    return "preserved"


def front_matter_block(data: bytes) -> bytes | None:
    if not data.startswith(b"---\n"):
        return None
    end = data.find(b"\n---", len(b"---\n"))
    if end == -1:
        return None
    return data[len(b"---\n") : end + 1]


def strip_trailing_ws(block: bytes) -> bytes:
    return b"\n".join(line.rstrip() for line in block.split(b"\n"))


def materialize_command(command: list[str], *, target: Path) -> list[str]:
    return [str(target) if part == "{target}" else part for part in command]


def display_command(tool: Tool) -> str:
    return shlex.join(materialize_command(tool.command, target=Path("TARGET")))


def stderr_tail(stderr: str) -> str:
    lines = [line for line in stderr.splitlines() if line.strip()]
    return "\n".join(lines[-8:]) if lines else "command failed"


def print_table(results: list[dict[str, object]]) -> None:
    headers = [
        "file",
        "formatter",
        "status",
        "median_seconds",
        "peak_rss_mb",
        "mb_per_second",
        "output_mb",
        "changed",
        "reason",
    ]
    print("| " + " | ".join(headers) + " |")
    print("| --- | --- | --- | ---: | ---: | ---: | --- | --- |")
    for row in results:
        print("| " + " | ".join(table_value(row, header) for header in headers) + " |")


def table_value(row: dict[str, object], header: str) -> str:
    if header == "median_seconds":
        value = row.get(header)
        return "" if value is None else f"{float(value):.4f}"
    if header == "peak_rss_mb":
        value = row.get("median_peak_rss_bytes")
        return "" if value is None else f"{float(value) / 1_000_000:.1f}"
    if header == "mb_per_second":
        value = row.get(header)
        return "" if value is None else f"{float(value):.2f}"
    if header == "output_mb":
        value = row.get("output_bytes")
        return "" if value is None else f"{float(value) / 1_000_000:.2f}"
    if header == "changed":
        value = row.get(header)
        return "" if value is None else str(bool(value)).lower()
    value = str(row.get(header, ""))
    return value.replace("|", "\\|").replace("\n", " ")


def benchmark_document(
    *,
    run_id: str,
    args: argparse.Namespace,
    target_relatives: list[Path],
    target_bytes: dict[str, int],
    selected: list[str],
    tool_versions: dict[str, str | None],
    results: list[dict[str, object]],
) -> dict[str, object]:
    return {
        "schema_version": 1,
        "benchmark": "big-file-formatting",
        "run_id": run_id,
        "created_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "git": git_metadata(),
        "host": {
            "system": platform.system(),
            "machine": platform.machine(),
            "cpu": cpu_brand(),
            "python": platform.python_version(),
        },
        "corpus": {
            "kind": "big-file",
            "files": len(target_relatives),
            "targets": [path.as_posix() for path in target_relatives],
            "bytes": sum(target_bytes.values()),
            "bytes_by_target": target_bytes,
            "requested_bytes": {
                "markdown": args.markdown_bytes,
                "yaml": args.yaml_bytes,
                "frontmatter": args.frontmatter_bytes,
                "frontmatter_yaml": args.frontmatter_yaml_bytes,
            },
            "seed": args.seed,
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
    commit = artifact["git"]["commit"]
    path = artifact_dir / f"{commit}.json"
    path.write_text(json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def display_path(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT))
    except ValueError:
        return str(path)


if __name__ == "__main__":
    sys.exit(main())
