#!/usr/bin/env bash
set -euo pipefail

CATALOG_DIR="/etc/vector/catalog"

# Soit SOURCE_TYPE="network/opnsense" (1 parser, backward-compat)
# Soit SOURCE_TYPES="network/opnsense,linux/auth-log,linux/syslog" (multi-parsers)
# Les composants Vector sont namespaces par source (opnsense_input, authlog_input, etc.)
# donc il n'y a pas de collision quand on charge plusieurs vector.toml dans un meme process.
SOURCES_CSV="${SOURCE_TYPES:-${SOURCE_TYPE:-}}"

if [ -z "$SOURCES_CSV" ]; then
  echo "FATAL: SOURCE_TYPE (single) or SOURCE_TYPES (csv) is required"
  echo "Available sources:"
  find "$CATALOG_DIR" -name "vector.toml" | sed "s|$CATALOG_DIR/||;s|/vector.toml||" | sort
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
    echo "FATAL: Config not found: $cfg"
    echo "Available sources:"
    find "$CATALOG_DIR" -name "vector.toml" | sed "s|$CATALOG_DIR/||;s|/vector.toml||" | sort
    exit 1
  fi
  CONFIG_ARGS+=(--config "$cfg")
  SELECTED_SOURCES+=("$src")
done

# Mode multi-parsers : un meme process Vector charge plusieurs configs, mais
# l'expansion ${DATASOURCE_ID} est globale => tous les parsers auraient le meme
# datasource_id. On contourne en reecrivant chaque config dans /tmp avec un
# datasource_id derive de "<TENANT>-<source-path>" (ex: bibihome-network-opnsense).
# En mode single (1 source), on conserve le ${DATASOURCE_ID} utilisateur tel quel.
if [ ${#SELECTED_SOURCES[@]} -gt 1 ]; then
  TMP_DIR="$(mktemp -d)"
  REWRITTEN_ARGS=()
  for src in "${SELECTED_SOURCES[@]}"; do
    ds="${DATASOURCE_ID}-$(echo "$src" | tr '/' '-')"
    safe_name="$(echo "$src" | tr '/' '-')"
    out="$TMP_DIR/$safe_name.toml"
    # Expand seulement DATASOURCE_ID ; les autres ${VAR} restent interpretes par Vector.
    sed "s|\${DATASOURCE_ID}|$ds|g" "$CATALOG_DIR/$src/vector.toml" > "$out"
    REWRITTEN_ARGS+=(--config "$out")
    echo "  $src -> datasource_id=$ds"
  done
  CONFIG_ARGS=("${REWRITTEN_ARGS[@]}")
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

echo "Starting Vector with ${#SELECTED_SOURCES[@]} source(s): ${SELECTED_SOURCES[*]}"
echo "  Tenant:     $TENANT_ID"
echo "  Datasource: $DATASOURCE_ID"
echo "  Quickwit:   $QUICKWIT_ENDPOINT"

exec vector "${CONFIG_ARGS[@]}" "$@"
