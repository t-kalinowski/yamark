#!/usr/bin/env python3

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    target = Path(sys.argv[1])
    yaml_files = yaml_input_paths(target)
    if not yaml_files:
        return 0

    with tempfile.TemporaryDirectory(prefix="yamark-panache-yaml-") as tmp:
        qmd_root = Path(tmp)
        pairs = []
        for yaml_path in yaml_files:
            relative = Path(yaml_path.name) if target.is_file() else yaml_path.relative_to(target)
            qmd_path = qmd_root / relative.with_name(f"{relative.name}.qmd")
            qmd_path.parent.mkdir(parents=True, exist_ok=True)
            yaml_text = yaml_path.read_text(encoding="utf-8")
            if yaml_text and not yaml_text.endswith("\n"):
                yaml_text += "\n"
            qmd_path.write_text(f"---\n{yaml_text}---\n", encoding="utf-8")
            pairs.append((yaml_path, qmd_path))

        panache_target = pairs[0][1] if target.is_file() else qmd_root
        subprocess.run(
            ["panache", "--no-cache", "format", str(panache_target)],
            check=True,
        )

        for yaml_path, qmd_path in pairs:
            yaml_path.write_text(extract_frontmatter(qmd_path), encoding="utf-8")

    return 0


def yaml_input_paths(target: Path) -> list[Path]:
    if target.is_file():
        assert target.suffix in {".yaml", ".yml"}
        return [target]
    assert target.is_dir()
    return [*target.rglob("*.yaml"), *target.rglob("*.yml")]


def extract_frontmatter(path: Path) -> str:
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    assert lines and lines[0].strip() == "---"
    for index, line in enumerate(lines[1:], start=1):
        if line.strip() == "---":
            return "".join(lines[1:index])
    raise AssertionError(f"missing front matter closer in {path}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as err:
        raise SystemExit(err.returncode) from err
