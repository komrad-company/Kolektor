#!/usr/bin/env bash
# =============================================================================
# Kolektor — Quickwit index initialisation
# Creates OCSF indexes if they do not already exist (idempotent)
# Usage: QUICKWIT_ENDPOINT=http://... ./create-indexes.sh
# =============================================================================
set -euo pipefail

QUICKWIT_ENDPOINT="${QUICKWIT_ENDPOINT:-http://quickwit-searcher.quickwit:7280}"
INDEXES_DIR="$(dirname "$0")/indexes"

echo "Quickwit endpoint: $QUICKWIT_ENDPOINT"
echo ""

# Wait for Quickwit to be ready
echo "Waiting for Quickwit..."
for i in $(seq 1 30); do
  if curl -sf --max-time 5 "$QUICKWIT_ENDPOINT/api/v1/version" > /dev/null 2>&1; then
    echo "Quickwit ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Quickwit unreachable after 30 attempts" >&2
    exit 1
  fi
  sleep 2
done

echo ""
created=0
skipped=0
failed=0

body_file="$(mktemp)"
trap 'rm -f "$body_file"' EXIT

shopt -s nullglob
config_files=("$INDEXES_DIR"/*.json)
if [ "${#config_files[@]}" -eq 0 ]; then
  echo "ERROR: no index definitions found in $INDEXES_DIR" >&2
  exit 1
fi

for config_file in "${config_files[@]}"; do
  index_id="$(basename "$config_file" .json)"

  # Check if the index already exists — a transient error must not
  # abort the script (set -e) nor trigger a blind create attempt
  status=$(curl -s --max-time 30 -o /dev/null -w "%{http_code}" \
    "$QUICKWIT_ENDPOINT/api/v1/indexes/$index_id") || status="000"

  if [ "$status" = "200" ]; then
    echo "[$index_id] already exists — skipped"
    skipped=$((skipped + 1))
    continue
  fi
  if [ "$status" != "404" ]; then
    echo "[$index_id] ERROR: existence check returned HTTP $status" >&2
    failed=$((failed + 1))
    continue
  fi

  # Create the index
  http_code=$(curl -s --max-time 30 -o "$body_file" -w "%{http_code}" \
    -X POST "$QUICKWIT_ENDPOINT/api/v1/indexes" \
    -H "Content-Type: application/json" \
    --data-binary "@$config_file") || http_code="000"

  if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    echo "[$index_id] created"
    created=$((created + 1))
  else
    echo "[$index_id] ERROR HTTP $http_code: $(cat "$body_file")" >&2
    failed=$((failed + 1))
  fi
done

echo ""
echo "Result: $created created, $skipped skipped, $failed errors"
[ "$failed" -eq 0 ] || exit 1
