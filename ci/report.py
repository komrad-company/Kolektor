#!/usr/bin/env python3
"""Generates a markdown report from the CI JUnit results."""

import xml.etree.ElementTree as ET
from pathlib import Path

RESULTS_DIR = Path("ci/results")


def parse_junit(path: Path) -> list[dict]:
    """Parses a JUnit XML file and returns the list of test cases."""
    if not path.exists():
        return []
    tree = ET.parse(path)
    results = []
    for tc in tree.iter("testcase"):
        failure = tc.find("failure")
        results.append({
            "name": tc.get("name", "unknown"),
            "classname": tc.get("classname", ""),
            "passed": failure is None,
            "message": failure.text if failure is not None else "",
        })
    return results


def main():
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    validate_results = parse_junit(RESULTS_DIR / "validate-junit.xml")
    test_results = parse_junit(RESULTS_DIR / "test-junit.xml")

    lines = ["# Kolektor — CI Report", ""]

    # Validation
    v_pass = sum(1 for r in validate_results if r["passed"])
    v_total = len(validate_results)
    lines.append(f"## Validation: {v_pass}/{v_total}")
    lines.append("")
    lines.append("| Source | Status |")
    lines.append("|--------|--------|")
    for r in validate_results:
        status = "OK" if r["passed"] else "FAIL"
        lines.append(f"| {r['name']} | {status} |")
    lines.append("")

    # Tests
    t_pass = sum(1 for r in test_results if r["passed"])
    t_total = len(test_results)
    lines.append(f"## Tests: {t_pass}/{t_total}")
    lines.append("")
    lines.append("| Source | Status |")
    lines.append("|--------|--------|")
    for r in test_results:
        status = "OK" if r["passed"] else "FAIL"
        lines.append(f"| {r['name']} | {status} |")
    lines.append("")

    # Errors
    failures = [r for r in validate_results + test_results if not r["passed"]]
    if failures:
        lines.append("## Errors")
        lines.append("")
        for r in failures:
            lines.append(f"### {r['name']}")
            lines.append(f"```\n{r['message']}\n```")
            lines.append("")

    report_path = RESULTS_DIR / "report.md"
    report_path.write_text("\n".join(lines))
    print(f"Report generated: {report_path}")


if __name__ == "__main__":
    main()
