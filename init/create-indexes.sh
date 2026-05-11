#!/usr/bin/env bash
# =============================================================================
# Kolektor — Quickwit index initialisation
# Creates OCSF indexes if they do not already exist (idempotent)
# Usage: QUICKWIT_ENDPOINT=http://... ./create-indexes.sh
# =============================================================================
set -euo pipefail

QUICKWIT_ENDPOINT="${QUICKWIT_ENDPOINT:-http://quickwit-searcher.quickwit:7280}"
INDEXES_DIR="$(dirname "$0")/indexes"
INDEXES="ocsf-network ocsf-http ocsf-dns ocsf-endpoint ocsf-identity ocsf-audit ocsf-k8s raw-logs"

echo "Quickwit endpoint: $QUICKWIT_ENDPOINT"
echo ""

# Wait for Quickwit to be ready
echo "Waiting for Quickwit..."
for i in $(seq 1 30); do
  if curl -sf "$QUICKWIT_ENDPOINT/api/v1/version" > /dev/null 2>&1; then
    echo "Quickwit ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Quickwit unreachable after 30 attempts"
    exit 1
  fi
  sleep 2
done

echo ""
created=0
skipped=0
failed=0

for index_id in $INDEXES; do
  config_file="$INDEXES_DIR/${index_id}.json"

  if [ ! -f "$config_file" ]; then
    echo "[$index_id] ERROR: config not found: $config_file"
    ((failed++))
    continue
  fi

  # Check if the index already exists
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    "$QUICKWIT_ENDPOINT/api/v1/indexes/$index_id")

  if [ "$status" = "200" ]; then
    echo "[$index_id] already exists — skipped"
    ((skipped++))
    continue
  fi

  # Create the index
  response=$(curl -s -w "\n%{http_code}" \
    -X POST "$QUICKWIT_ENDPOINT/api/v1/indexes" \
    -H "Content-Type: application/json" \
    --data-binary "@$config_file")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | head -n -1)

  if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    echo "[$index_id] created"
    ((created++))
  else
    echo "[$index_id] ERROR HTTP $http_code: $body"
    ((failed++))
  fi
done

echo ""
echo "Result: $created created, $skipped skipped, $failed errors"
[ "$failed" -eq 0 ] || exit 1
