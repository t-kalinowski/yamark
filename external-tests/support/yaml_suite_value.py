import json
import sys
from pathlib import Path

try:
    import yaml12
except ImportError as err:
    raise SystemExit("missing Python package: yaml12 (install py-yaml12)") from err


def normalize(value):
    if isinstance(value, yaml12.Yaml):
        if value.tag == "!":
            return str(value.value)
        return normalize(value.value)
    if isinstance(value, float) and value.is_integer():
        return int(value)
    if isinstance(value, dict):
        entries = [[normalize(key), normalize(val)] for key, val in value.items()]
        entries.sort(key=lambda item: json.dumps(item[0], sort_keys=True, separators=(",", ":")))
        return {"__map__": entries}
    if isinstance(value, list):
        return [normalize(item) for item in value]
    return value


def parse_json_stream(text):
    decoder = json.JSONDecoder()
    pos = 0
    values = []
    while True:
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text):
            return values
        value, pos = decoder.raw_decode(text, pos)
        values.append(value)


def main():
    mode, path = sys.argv[1], Path(sys.argv[2])
    text = path.read_text(encoding="utf-8")
    if mode == "yaml":
        value = yaml12.parse_yaml(text, multi=True)
    elif mode == "json":
        value = parse_json_stream(text)
    else:
        raise SystemExit(f"unknown mode: {mode}")

    print(json.dumps(normalize(value), sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
