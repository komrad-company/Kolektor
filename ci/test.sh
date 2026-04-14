#!/usr/bin/env bash
# Lance vector test sur chaque source du catalog
set -euo pipefail

# Dummy env vars pour que Vector ne plante pas sur les ${VAR} dans les configs
export TENANT_ID="ci-test"
export DATASOURCE_ID="ci-test"
export QUICKWIT_ENDPOINT="http://localhost:7280"
export LISTEN_PORT="5140"
export AUTH_LOG_PATH="/dev/null"
export AUDIT_LOG_PATH="/dev/null"
export NGINX_ACCESS_LOG="/dev/null"

RESULTS_DIR="ci/results"
mkdir -p "$RESULTS_DIR"

TOTAL=0
PASSED=0
FAILED=0
JUNIT_CASES=""

for source_dir in catalog/*/*/; do
  CONFIG="${source_dir}vector.toml"
  [ -f "$CONFIG" ] || continue

  # Chercher les fichiers de test
  TEST_FILES=$(find "${source_dir}tests/" -name '*.toml' 2>/dev/null | sort)
  [ -z "$TEST_FILES" ] && continue

  TOTAL=$((TOTAL + 1))
  SOURCE_NAME=$(echo "$source_dir" | sed 's|catalog/||;s|/$||;s|/|.|')

  echo "--- Testing: $SOURCE_NAME"

  # Merger config + tests dans un fichier temporaire
  MERGED=$(mktemp /tmp/vector-test-XXXXXX.toml)
  cat "$CONFIG" > "$MERGED"
  echo "" >> "$MERGED"
  for tf in $TEST_FILES; do
    echo "" >> "$MERGED"
    cat "$tf" >> "$MERGED"
  done

  if OUTPUT=$(vector test "$MERGED" 2>&1); then
    echo "  OK"
    PASSED=$((PASSED + 1))
    JUNIT_CASES="${JUNIT_CASES}    <testcase classname=\"test\" name=\"${SOURCE_NAME}\" />\n"
  else
    echo "  FAIL"
    echo "$OUTPUT"
    FAILED=$((FAILED + 1))
    ESCAPED_OUTPUT=$(echo "$OUTPUT" | sed 's/&/\&amp;/g;s/</\&lt;/g;s/>/\&gt;/g;s/"/\&quot;/g')
    JUNIT_CASES="${JUNIT_CASES}    <testcase classname=\"test\" name=\"${SOURCE_NAME}\">\n      <failure message=\"test failed\">${ESCAPED_OUTPUT}</failure>\n    </testcase>\n"
  fi

  rm -f "$MERGED"
done

# Generer le JUnit XML
cat > "$RESULTS_DIR/test-junit.xml" <<XMLEOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="test" tests="$TOTAL" failures="$FAILED">
$(echo -e "$JUNIT_CASES")
  </testsuite>
</testsuites>
XMLEOF

echo ""
echo "=== Tests: $PASSED/$TOTAL sources passed, $FAILED failed ==="

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
