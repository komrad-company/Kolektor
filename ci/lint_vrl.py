#!/usr/bin/env python3
"""Static VRL antipattern linter for the Kolektor catalogue.

Catches the bug classes the 2026-06-09 review found in merged parsers —
classes that `vector validate` does not reject (a config can compile and
still silently lose events or drop a mandatory OCSF field).

Exit codes: 0 clean, 1 errors found. Warnings never fail CI.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "catalog"

# Every classified OCSF event must carry these. A missing field is invisible
# to `vector validate` but pollutes Quickwit (null tenant attribution, etc.).
MANDATORY_FIELDS = [
    "class_uid",
    "category_uid",
    "severity_id",
    "time",
    "metadata",
    "tenant_id",
    "datasource_id",
    "raw",
    "uid",
]

# `"key": if cond { .. }` inside an object literal — invalid VRL, yet an easy
# slip; this exact shape shipped in microsoft-365-audit and broke the build.
INLINE_IF_IN_OBJECT = re.compile(r'"[^"]+"\s*:\s*if\b')

# `to_int(a) ?? to_int(b)` — a null/empty first arg coerces silently (no
# error), so the second cast is dead: this produced time=0 in crowdstrike and
# routed NXLog events to raw-logs in windows-evtx. A single `to_x(.f) ?? null`
# is only "" / 0 pollution (minor) and deliberately not flagged — too noisy.
DEAD_CAST_CHAIN = re.compile(r"to_(?:int|string|float|bool)\([^()]*\)\s*\?\?\s*to_")


def lint_source(vector_toml: Path) -> tuple[list[str], list[str]]:
    text = vector_toml.read_text(encoding="utf-8")
    rel = vector_toml.relative_to(ROOT)
    errors: list[str] = []
    warnings: list[str] = []

    for lineno, line in enumerate(text.splitlines(), start=1):
        if INLINE_IF_IN_OBJECT.search(line):
            errors.append(f"{rel}:{lineno}: inline `if` inside object literal — pre-compute the value first")
        if DEAD_CAST_CHAIN.search(line):
            warnings.append(f"{rel}:{lineno}: dead cast chain — first `to_x` coerces null silently, the `?? to_x` branch never runs; guard with `if x != null`")

    missing = [field for field in MANDATORY_FIELDS if field not in text]
    if missing:
        errors.append(f"{rel}: mandatory OCSF field(s) never set: {', '.join(missing)}")

    return errors, warnings


def main() -> int:
    sources = sorted(CATALOG.glob("*/*/vector.toml"))
    if not sources:
        print(f"no parsers found under {CATALOG.relative_to(ROOT)}/", file=sys.stderr)
        return 1

    all_errors: list[str] = []
    all_warnings: list[str] = []
    for source in sources:
        errors, warnings = lint_source(source)
        all_errors.extend(errors)
        all_warnings.extend(warnings)

    for warning in all_warnings:
        print(f"WARN  {warning}")
    for error in all_errors:
        print(f"ERROR {error}", file=sys.stderr)

    print(f"\nlint_vrl: {len(sources)} parsers, {len(all_errors)} errors, {len(all_warnings)} warnings")
    return 1 if all_errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
