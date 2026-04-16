#!/usr/bin/env bash
set -euo pipefail

CATALOG_DIR="/etc/vector/catalog"

if [ -z "${SOURCE_TYPE:-}" ]; then
  echo "FATAL: SOURCE_TYPE is required (e.g. network/opnsense, linux/auth-log)"
  echo "Available sources:"
  find "$CATALOG_DIR" -name "vector.toml" | sed "s|$CATALOG_DIR/||;s|/vector.toml||" | sort
  exit 1
fi

CONFIG_FILE="$CATALOG_DIR/$SOURCE_TYPE/vector.toml"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "FATAL: Config not found: $CONFIG_FILE"
  echo "Available sources:"
  find "$CATALOG_DIR" -name "vector.toml" | sed "s|$CATALOG_DIR/||;s|/vector.toml||" | sort
  exit 1
fi

# Valider les variables runtime obligatoires (sinon Vector produit des events avec tenant_id="" qui polluent Quickwit)
MISSING=()
[ -z "${TENANT_ID:-}" ]         && MISSING+=("TENANT_ID")
[ -z "${DATASOURCE_ID:-}" ]     && MISSING+=("DATASOURCE_ID")
[ -z "${QUICKWIT_ENDPOINT:-}" ] && MISSING+=("QUICKWIT_ENDPOINT")

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "FATAL: required environment variables missing: ${MISSING[*]}"
  exit 1
fi

echo "Starting Vector with source: $SOURCE_TYPE"
echo "  Config:     $CONFIG_FILE"
echo "  Tenant:     $TENANT_ID"
echo "  Datasource: $DATASOURCE_ID"
echo "  Quickwit:   $QUICKWIT_ENDPOINT"

exec vector --config "$CONFIG_FILE" "$@"
