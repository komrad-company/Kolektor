#!/usr/bin/env python3
"""OCSF coherence validator for the Kolektor catalogue.

Checks the semantic invariants that `vector validate` (syntax) and
`lint_vrl.py` (antipatterns) cannot see: a config compiles cleanly and still
emits an event with category != class/1000, a severity_id outside the spec
enum, or a classified event with no activity_id. Enums are the D6 table of
docs/decisions.md.

Exit codes: 0 clean, 1 errors found.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "catalog"

SEVERITY_IDS = {0, 1, 2, 3, 4, 5, 6, 99}

# Known OCSF class uids the catalogue maps to (extend as parsers are added).
KNOWN_CLASSES = {
    0,  # Base Event (generic — D2)
    1003, 1005, 1007,  # system: Kernel / Module / Process Activity
    2001,  # findings: Security Finding
    3001, 3002,  # iam: Account Change / Authentication
    4001, 4002, 4003, 4009,  # network: Network / HTTP / DNS / Email Activity
    6003,  # application: API Activity
}

# activity_id enums per class (D6). A literal outside its set is an error.
ACTIVITY_IDS = {
    1007: {0, 1, 2, 3, 4, 5, 99},
    3002: {0, 1, 2, 3, 4, 5, 6, 7, 99},
    6003: {0, 1, 2, 3, 4, 99},
    4001: {0, 1, 2, 3, 4, 5, 6, 7, 99},
    4002: {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 99},
    4003: {0, 1, 2, 6, 99},
}

SEVERITY_ASSIGN = re.compile(r'(?:\.severity_id\s*=|"severity_id"\s*:)\s*(\d+)')
CLASS_ASSIGN = re.compile(r'(?:\.class_uid\s*=|"class_uid"\s*:)\s*(\d+)')
CATEGORY_ASSIGN = re.compile(r'(?:\.category_uid\s*=|"category_uid"\s*:)\s*(\d+)')


def validate_source(vector_toml: Path) -> tuple[list[str], list[str]]:
    text = vector_toml.read_text(encoding="utf-8")
    rel = vector_toml.relative_to(ROOT)
    errors: list[str] = []
    warnings: list[str] = []

    severities = {int(match) for match in SEVERITY_ASSIGN.findall(text)}
    for severity in severities - SEVERITY_IDS:
        errors.append(f"{rel}: severity_id literal {severity} outside OCSF enum {sorted(SEVERITY_IDS)}")

    classes = {int(match) for match in CLASS_ASSIGN.findall(text)}
    categories = {int(match) for match in CATEGORY_ASSIGN.findall(text)}

    for class_uid in classes:
        if class_uid == 0:
            continue
        if class_uid not in KNOWN_CLASSES:
            errors.append(f"{rel}: class_uid {class_uid} is not a known OCSF class (update KNOWN_CLASSES if intentional)")
        expected_category = class_uid // 1000
        if expected_category not in categories and categories - {0}:
            errors.append(f"{rel}: class_uid {class_uid} expects category_uid {expected_category}, file sets {sorted(categories)}")

    classifies = bool(classes - {0})
    if classifies and "activity_id" not in text:
        warnings.append(f"{rel}: classified parser sets no activity_id (mandatory per D1)")

    return errors, warnings


def main() -> int:
    sources = sorted(CATALOG.glob("*/*/vector.toml"))
    if not sources:
        print(f"no parsers found under {CATALOG.relative_to(ROOT)}/", file=sys.stderr)
        return 1

    all_errors: list[str] = []
    all_warnings: list[str] = []
    for source in sources:
        errors, warnings = validate_source(source)
        all_errors.extend(errors)
        all_warnings.extend(warnings)

    for warning in all_warnings:
        print(f"WARN  {warning}")
    for error in all_errors:
        print(f"ERROR {error}", file=sys.stderr)

    print(f"\nvalidate_ocsf: {len(sources)} parsers, {len(all_errors)} errors, {len(all_warnings)} warnings")
    return 1 if all_errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
