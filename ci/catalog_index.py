#!/usr/bin/env python3
"""Generate the Kolektor parser catalogue consumed by Kolektor-kontroler."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[1]
CATALOG_DIR = ROOT / "catalog"
INDEX_PATH = CATALOG_DIR / "index.json"
INGEST_RE = re.compile(r"\$\{QUICKWIT_ENDPOINT\}/api/v1/([^/]+)/ingest")


def first_readme_paragraph(readme: Path) -> str | None:
    if not readme.exists():
        return None

    lines: list[str] = []
    for raw in readme.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            if lines:
                break
            continue
        lines.append(line)

    return " ".join(lines) if lines else None


def output_indexes(manifest: dict[str, Any], vector_toml: str) -> list[str]:
    indexes: list[str] = []
    for output in manifest.get("ocsf_outputs", []) or []:
        index = output.get("index")
        if isinstance(index, str) and index not in indexes:
            indexes.append(index)

    for index in INGEST_RE.findall(vector_toml):
        if index != "raw-logs" and index not in indexes:
            indexes.append(index)

    return indexes


def parser_entry(source_dir: Path) -> dict[str, Any]:
    manifest_path = source_dir / "manifest.yaml"
    vector_path = source_dir / "vector.toml"
    manifest = yaml.safe_load(manifest_path.read_text(encoding="utf-8")) or {}
    vector_toml = vector_path.read_text(encoding="utf-8")
    source_type = source_dir.relative_to(CATALOG_DIR).as_posix()
    category = source_type.split("/", 1)[0]
    indexes = output_indexes(manifest, vector_toml)

    return {
        "source_type": source_type,
        "display_name": manifest["display_name"],
        "version": manifest.get("version"),
        "category": category,
        "default_port": manifest.get("default_port"),
        "ocsf_index": ", ".join(indexes) if indexes else None,
        "description": manifest.get("description")
        or first_readme_paragraph(source_dir / "README.md"),
        "vector_toml": vector_toml,
    }


def build_index() -> dict[str, Any]:
    entries = []
    for path in sorted(CATALOG_DIR.glob("*/*/manifest.yaml")):
        if not (path.parent / "vector.toml").exists():
            sys.exit(f"{path.parent.relative_to(ROOT)}: manifest.yaml without vector.toml")
        entries.append(parser_entry(path.parent))
    if not entries:
        sys.exit(f"no parsers found under {CATALOG_DIR.relative_to(ROOT)}/")
    return {"version": 1, "parsers": entries}


def encode(index: dict[str, Any]) -> str:
    return json.dumps(index, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail if index is stale")
    args = parser.parse_args()

    index = build_index()
    content = encode(index)

    if args.check:
        if not INDEX_PATH.exists():
            print(f"{INDEX_PATH} is missing", file=sys.stderr)
            return 1
        current = INDEX_PATH.read_text(encoding="utf-8")
        if current != content:
            print(f"{INDEX_PATH} is stale; run ci/catalog_index.py", file=sys.stderr)
            return 1
        print(f"{INDEX_PATH.relative_to(ROOT)} in sync — {len(index['parsers'])} parsers checked")
        return 0

    INDEX_PATH.write_text(content, encoding="utf-8")
    print(f"wrote {INDEX_PATH.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
