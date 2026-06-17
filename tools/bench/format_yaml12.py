#!/usr/bin/env python3

from __future__ import annotations

import sys
from pathlib import Path

from yaml12 import *


def main() -> int:
    target = Path(sys.argv[1])
    for yaml_path in yaml_input_paths(target):
        write_yaml(read_yaml(yaml_path), yaml_path)
    return 0


def yaml_input_paths(target: Path) -> list[Path]:
    if target.is_file():
        assert target.suffix in {".yaml", ".yml"}
        return [target]
    assert target.is_dir()
    return [*target.rglob("*.yaml"), *target.rglob("*.yml")]


if __name__ == "__main__":
    raise SystemExit(main())
