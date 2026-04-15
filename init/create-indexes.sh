#!/usr/bin/env bash
# =============================================================================
# Kolektor — initialisation des index Quickwit
# Crée les 4 index OCSF si ils n'existent pas déjà (idempotent)
# Usage : QUICKWIT_ENDPOINT=http://... ./create-indexes.sh
# =============================================================================
set -euo pipefail

QUICKWIT_ENDPOINT="${QUICKWIT_ENDPOINT:-http://quickwit-searcher.quickwit:7280}"
INDEXES_DIR="$(dirname "$0")/indexes"
INDEXES="ocsf-network ocsf-endpoint ocsf-identity ocsf-audit raw-logs"

echo "Quickwit endpoint : $QUICKWIT_ENDPOINT"
echo ""

# Attendre que Quickwit soit prêt
echo "Attente de Quickwit..."
for i in $(seq 1 30); do
  if curl -sf "$QUICKWIT_ENDPOINT/api/v1/version" > /dev/null 2>&1; then
    echo "Quickwit prêt."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERREUR : Quickwit non joignable après 30 tentatives"
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
    echo "[$index_id] ERREUR : config introuvable : $config_file"
    ((failed++))
    continue
  fi

  # Vérifier si l'index existe déjà
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    "$QUICKWIT_ENDPOINT/api/v1/indexes/$index_id")

  if [ "$status" = "200" ]; then
    echo "[$index_id] déjà existant — ignoré"
    ((skipped++))
    continue
  fi

  # Créer l'index
  response=$(curl -s -w "\n%{http_code}" \
    -X POST "$QUICKWIT_ENDPOINT/api/v1/indexes" \
    -H "Content-Type: application/json" \
    --data-binary "@$config_file")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | head -n -1)

  if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
    echo "[$index_id] créé"
    ((created++))
  else
    echo "[$index_id] ERREUR HTTP $http_code : $body"
    ((failed++))
  fi
done

echo ""
echo "Résultat : $created créés, $skipped ignorés, $failed erreurs"
[ "$failed" -eq 0 ] || exit 1
