#!/usr/bin/env python3

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

VERSION = re.compile(
    r"^(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:\+dev)?$"
)


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def read_prefixed_version(path: Path, prefix: str) -> str:
    pattern = re.compile(rf'(?m)^{re.escape(prefix)}\nversion = "([^"]+)"$')
    matches = pattern.findall(path.read_text())
    if len(matches) != 1:
        fail(f"expected exactly one Yamark version in {path}")
    return matches[0]


def read_versions(root: Path) -> dict[Path, str]:
    cargo_path = root / "Cargo.toml"
    cargo_lock_path = root / "Cargo.lock"
    pyproject_path = root / "pyproject.toml"
    uv_lock_path = root / "uv.lock"
    extension_path = root / "editors/vscode/package.json"

    extension = json.loads(extension_path.read_text())
    if extension.get("name") != "yamark" or not isinstance(
        extension.get("version"), str
    ):
        fail(f"expected the Yamark extension version in {extension_path}")

    return {
        cargo_path: read_prefixed_version(cargo_path, '[package]\nname = "yamark"'),
        cargo_lock_path: read_prefixed_version(
            cargo_lock_path, '[[package]]\nname = "yamark"'
        ),
        pyproject_path: read_prefixed_version(
            pyproject_path, '[project]\nname = "yamark"'
        ),
        uv_lock_path: read_prefixed_version(
            uv_lock_path, '[[package]]\nname = "yamark"'
        ),
        extension_path: extension["version"],
    }


def replace_once(contents: str, before: str, after: str, path: Path) -> str:
    if contents.count(before) != 1:
        fail(f"expected exactly one Yamark version field in {path}")
    return contents.replace(before, after, 1)


def stable_parts(version: str) -> tuple[int, int, int]:
    major, minor, patch = version.split(".")
    return int(major), int(minor), int(patch)


def main() -> None:
    if len(sys.argv) != 2 or not VERSION.fullmatch(sys.argv[1]):
        fail("provide one version like 0.4.0 or 0.4.0+dev")
    target = sys.argv[1]
    root = Path(__file__).resolve().parents[1]
    versions = read_versions(root)
    current_versions = set(versions.values())
    if len(current_versions) != 1:
        details = ", ".join(
            f"{path.relative_to(root)}={version}" for path, version in versions.items()
        )
        fail(f"versions are out of sync: {details}")
    current = current_versions.pop()
    if current == target:
        fail(f"version is already {target}")
    if not VERSION.fullmatch(current):
        fail(f"current version is invalid: {current}")
    if current.endswith("+dev"):
        stable_current = current[: -len("+dev")]
        if target.endswith("+dev") or stable_parts(target) <= stable_parts(
            stable_current
        ):
            fail("development version must advance to a later stable version")
    else:
        expected = f"{current}+dev"
        if target != expected:
            fail(f"stable version can only move to {expected}")

    replacements = {
        root / "Cargo.toml": (
            f'[package]\nname = "yamark"\nversion = "{current}"',
            f'[package]\nname = "yamark"\nversion = "{target}"',
        ),
        root / "Cargo.lock": (
            f'[[package]]\nname = "yamark"\nversion = "{current}"',
            f'[[package]]\nname = "yamark"\nversion = "{target}"',
        ),
        root / "pyproject.toml": (
            f'[project]\nname = "yamark"\nversion = "{current}"',
            f'[project]\nname = "yamark"\nversion = "{target}"',
        ),
        root / "uv.lock": (
            f'[[package]]\nname = "yamark"\nversion = "{current}"',
            f'[[package]]\nname = "yamark"\nversion = "{target}"',
        ),
        root / "editors/vscode/package.json": (
            f'  "version": "{current}"',
            f'  "version": "{target}"',
        ),
    }
    updated = {
        path: replace_once(path.read_text(), before, after, path)
        for path, (before, after) in replacements.items()
    }
    for path, contents in updated.items():
        path.write_text(contents)

    actual = read_versions(root)
    assert set(actual.values()) == {target}
    print(f"Updated Yamark from {current} to {target} in five version files.")


if __name__ == "__main__":
    main()
