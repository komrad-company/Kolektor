#!/usr/bin/env bash
# Checks that each catalog source has at least 3 test files
set -euo pipefail

MIN_TESTS=3
TOTAL=0
PASSED=0
FAILED=0

for source_dir in catalog/*/*/; do
  CONFIG="${source_dir}vector.toml"
  [ -f "$CONFIG" ] || continue

  TOTAL=$((TOTAL + 1))
  SOURCE_NAME=$(echo "$source_dir" | sed 's|catalog/||;s|/$||;s|/|.|')

  TEST_COUNT=$(find "${source_dir}tests/" -name '*.toml' 2>/dev/null | wc -l)

  if [ "$TEST_COUNT" -ge "$MIN_TESTS" ]; then
    echo "OK   $SOURCE_NAME ($TEST_COUNT tests)"
    PASSED=$((PASSED + 1))
  else
    echo "FAIL $SOURCE_NAME ($TEST_COUNT/$MIN_TESTS tests minimum)"
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "=== Coverage: $PASSED/$TOTAL sources have >= $MIN_TESTS tests, $FAILED missing ==="

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
