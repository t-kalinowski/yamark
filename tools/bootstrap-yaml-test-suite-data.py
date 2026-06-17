#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_URL = "https://github.com/yaml/yaml-test-suite.git"


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = args.repo_root.expanduser().resolve()
    assert_repo_root(repo_root)

    if args.source is None:
        with tempfile.TemporaryDirectory(prefix="yaml-test-suite-") as tmp:
            source = download_suite(Path(tmp), args.ref)
            refresh_data(repo_root, source)
    else:
        refresh_data(repo_root, args.source.expanduser().resolve())

    return 0


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Refresh local untracked yaml-test-suite data under "
            "tests/yaml-test-suite/data."
        )
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path.cwd(),
        help="repository root to refresh (defaults to the current directory)",
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=None,
        help=(
            "existing yaml-test-suite fixture root with data/ and License, for "
            "example ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite"
        ),
    )
    parser.add_argument(
        "--ref",
        default=os.environ.get("YAML_TEST_SUITE_REF", ""),
        help="optional upstream commit-ish to checkout before running make data",
    )
    return parser.parse_args(argv)


def assert_repo_root(repo_root: Path) -> None:
    cargo_toml = repo_root / "Cargo.toml"
    assert cargo_toml.is_file(), f"Cargo.toml not found under repo root: {repo_root}"
    assert 'name = "yamark"' in cargo_toml.read_text(), f"wrong repository root: {repo_root}"


def download_suite(tmp: Path, ref: str) -> Path:
    clone_dir = tmp / "yaml-test-suite"
    run(["git", "clone", "--depth=1", "--no-single-branch", REPO_URL, os.fspath(clone_dir)])
    run(["git", "fetch", "origin", "data"], cwd=clone_dir)
    run(["git", "branch", "--track", "data", "origin/data"], cwd=clone_dir)
    if ref:
        run(["git", "checkout", ref], cwd=clone_dir)
    run(["make", "data"], cwd=clone_dir)
    return clone_dir


def refresh_data(repo_root: Path, source_root: Path) -> None:
    source_data = source_root / "data"
    source_license = source_root / "License"
    if not source_license.is_file():
        source_license = source_root / "LICENSE"

    assert source_data.is_dir(), f"yaml-test-suite data directory not found: {source_data}"
    assert source_license.is_file(), f"yaml-test-suite License not found under: {source_root}"

    dest_root = repo_root / "tests" / "yaml-test-suite"
    dest_data = dest_root / "data"
    dest_license = dest_root / "License"

    shutil.rmtree(dest_data, ignore_errors=True)
    dest_root.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source_data, dest_data, symlinks=True)
    remove_symlink_dirs(dest_data)
    shutil.copy2(source_license, dest_license)
    print(f"yaml-test-suite data refreshed at {dest_data}")


def remove_symlink_dirs(root: Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink() and path.is_dir():
            path.unlink()


def run(command: list[str], cwd: Path | None = None) -> None:
    try:
        subprocess.run(command, cwd=cwd, check=True)
    except FileNotFoundError as err:
        raise SystemExit(f"required command not found: {command[0]}") from err
    except subprocess.CalledProcessError as err:
        joined = " ".join(command)
        raise SystemExit(f"command failed: {joined}") from err


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
