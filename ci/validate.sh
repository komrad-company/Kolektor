#!/usr/bin/env bash
# Valide chaque vector.toml du catalog avec vector validate
set -euo pipefail

# Dummy env vars pour que Vector ne plante pas sur les ${VAR} dans les configs
export TENANT_ID="ci-validate"
export DATASOURCE_ID="ci-validate"
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

for config in catalog/*/*/vector.toml; do
  [ -f "$config" ] || continue
  TOTAL=$((TOTAL + 1))
  SOURCE_NAME=$(echo "$config" | sed 's|catalog/||;s|/vector.toml||;s|/|.|')

  echo "--- Validating: $config"
  if OUTPUT=$(vector validate "$config" 2>&1); then
    echo "  OK"
    PASSED=$((PASSED + 1))
    JUNIT_CASES="${JUNIT_CASES}    <testcase classname=\"validate\" name=\"${SOURCE_NAME}\" />\n"
  else
    echo "  FAIL"
    echo "$OUTPUT"
    FAILED=$((FAILED + 1))
    ESCAPED_OUTPUT=$(echo "$OUTPUT" | sed 's/&/\&amp;/g;s/</\&lt;/g;s/>/\&gt;/g;s/"/\&quot;/g')
    JUNIT_CASES="${JUNIT_CASES}    <testcase classname=\"validate\" name=\"${SOURCE_NAME}\">\n      <failure message=\"validation failed\">${ESCAPED_OUTPUT}</failure>\n    </testcase>\n"
  fi
done

# Generer le JUnit XML
cat > "$RESULTS_DIR/validate-junit.xml" <<XMLEOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="validate" tests="$TOTAL" failures="$FAILED">
$(echo -e "$JUNIT_CASES")
  </testsuite>
</testsuites>
XMLEOF

echo ""
echo "=== Validation: $PASSED/$TOTAL passed, $FAILED failed ==="

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi
