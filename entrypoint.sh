#!/usr/bin/env bash
set -euo pipefail

CATALOG_DIR="/etc/vector/catalog"

fatal() {
  echo "FATAL: $*" >&2
}

list_sources() {
  echo "Available sources:" >&2
  find "$CATALOG_DIR" -name "vector.toml" | sed "s|$CATALOG_DIR/||;s|/vector.toml||" | sort >&2
}

# Either SOURCE_TYPE="network/opnsense" (single parser, backward-compat)
# or SOURCE_TYPES="network/opnsense,linux/auth-log,linux/syslog" (multi-parser).
# Vector components are namespaced per source (opnsense_input, authlog_input...)
# so several vector.toml can be loaded in one process without collision.
SOURCES_CSV="${SOURCE_TYPES:-${SOURCE_TYPE:-}}"

if [ -z "$SOURCES_CSV" ]; then
  fatal "SOURCE_TYPE (single) or SOURCE_TYPES (csv) is required"
  list_sources
  exit 1
fi

# Required runtime variables — validated before any config rewrite so a
# missing variable is a clear FATAL, not an unbound-variable crash mid-rewrite.
MISSING=()
[ -z "${TENANT_ID:-}" ]         && MISSING+=("TENANT_ID")
[ -z "${DATASOURCE_ID:-}" ]     && MISSING+=("DATASOURCE_ID")
[ -z "${QUICKWIT_ENDPOINT:-}" ] && MISSING+=("QUICKWIT_ENDPOINT")

if [ ${#MISSING[@]} -gt 0 ]; then
  fatal "required environment variables missing: ${MISSING[*]}"
  exit 1
fi

# DATASOURCE_ID is substituted into configs with sed in multi-source mode:
# a strict charset keeps sed metacharacters (&, \, |) from silently
# corrupting the generated configs.
if ! [[ "$DATASOURCE_ID" =~ ^[A-Za-z0-9_-]+$ ]]; then
  fatal "DATASOURCE_ID must match [A-Za-z0-9_-]+ (got: $DATASOURCE_ID)"
  exit 1
fi

CONFIG_ARGS=()
SELECTED_SOURCES=()
IFS=',' read -r -a SOURCES_ARR <<< "$SOURCES_CSV"
for src in "${SOURCES_ARR[@]}"; do
  src="${src#"${src%%[![:space:]]*}"}"  # ltrim
  src="${src%"${src##*[![:space:]]}"}"  # rtrim
  [ -z "$src" ] && continue
  cfg="$CATALOG_DIR/$src/vector.toml"
  if [ ! -f "$cfg" ]; then
    fatal "Config not found: $cfg"
    list_sources
    exit 1
  fi
  CONFIG_ARGS+=(--config "$cfg")
  SELECTED_SOURCES+=("$src")
done

# Multi-parser mode: a single Vector process loads several configs, but the
# ${DATASOURCE_ID} expansion is global => every parser would share the same
# datasource_id. Each config is rewritten in /tmp with a datasource_id
# derived from "<DATASOURCE_ID>-<source-path>" (e.g. acme-network-opnsense).
# Single mode (1 source) keeps the user-supplied ${DATASOURCE_ID} untouched.
if [ ${#SELECTED_SOURCES[@]} -gt 1 ]; then
  TMP_DIR="$(mktemp -d)"
  # Cleared if the script dies before exec; a successful exec keeps the
  # files alive for Vector (the trap never fires after exec).
  trap 'rm -rf "$TMP_DIR"' EXIT
  REWRITTEN_ARGS=()
  for src in "${SELECTED_SOURCES[@]}"; do
    ds="${DATASOURCE_ID}-$(echo "$src" | tr '/' '-')"
    safe_name="$(echo "$src" | tr '/' '-')"
    out="$TMP_DIR/$safe_name.toml"
    # Expand only DATASOURCE_ID; the other ${VAR} stay for Vector to interpret.
    sed "s|\${DATASOURCE_ID}|$ds|g" "$CATALOG_DIR/$src/vector.toml" > "$out"
    REWRITTEN_ARGS+=(--config "$out")
    echo "  $src -> datasource_id=$ds"
  done
  CONFIG_ARGS=("${REWRITTEN_ARGS[@]}")
fi

echo "Starting Vector with ${#SELECTED_SOURCES[@]} source(s): ${SELECTED_SOURCES[*]}"
echo "  Tenant:     $TENANT_ID"
echo "  Datasource: $DATASOURCE_ID"
echo "  Quickwit:   $QUICKWIT_ENDPOINT"
echo "  Port:       ${LISTEN_PORT:-(parser default)}"

exec vector "${CONFIG_ARGS[@]}" "$@"
