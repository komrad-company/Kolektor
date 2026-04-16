# Guide de contribution — Nouveau parser

## Ajouter une nouvelle source

1. Copier le template :
   ```bash
   cp -r _schema/template.toml catalog/<category>/<source>/vector.toml
   ```

2. Editer `vector.toml` :
   - Adapter la source (syslog, file, http...)
   - Ecrire le VRL de parsing dans le transform `parse_and_normalize`
   - Mapper vers les champs OCSF obligatoires

3. Creer les tests dans `tests/` (minimum 3) :
   - `nominal.toml` — event standard, tous les champs presents
   - `optional_missing.toml` — champs optionnels manquants
   - `malformed.toml` — input invalide, doit etre droppe sans crash

4. Creer un `README.md` documentant :
   - Description de la source
   - Format de log attendu
   - Configuration cote source (comment envoyer les logs)
   - Variables d'environnement specifiques
   - Liens vers la doc officielle

5. Valider :
   ```bash
   # Vector 0.54 --no-environment ne desactive PAS l'expansion ${VAR} :
   # passer par ci/validate.sh (injecte des dummy vars) ou exporter soi-meme.
   bash ci/validate.sh
   bash ci/test.sh
   ```

## Champs OCSF obligatoires

Chaque event normalise doit contenir :

| Champ           | Type   | Description                    |
|-----------------|--------|--------------------------------|
| `class_uid`     | int    | Classe OCSF (ex: 4001)        |
| `category_uid`  | int    | Categorie OCSF (ex: 4)        |
| `severity_id`   | int    | 0=Unknown, 1=Info, 2=Low...   |
| `time`          | int    | Epoch milliseconds             |
| `metadata`      | object | `product.name`, `vendor_name`  |
| `tenant_id`     | string | Injecte via `$TENANT_ID`       |
| `datasource_id` | string | Injecte via `$DATASOURCE_ID`   |
| `raw`           | string | Message original conserve      |

## Conventions

- Fichiers en TOML
- VRL inline dans le transform (pas de fichier `.vrl` separe)
- Variables runtime en `${ENV_VAR}` avec defaults si applicable
- Logs de test : bruts reels, pas inventes

## Pattern `raw` + `uid` + corr\u00e9lation raw-logs

Pour les sources syslog (opnsense, auditd, auth-log, syslog linux), on reconstruit
la ligne brute et on ajoute un `uid` UUID partage entre l'index OCSF et `raw-logs` :

```vrl
_ts  = to_string(.timestamp) ?? ""
_pid = if .procid != null { "[" + to_string!(.procid) + "]" } else { "" }
.raw = _ts + " " + (string(.hostname) ?? "") + " " + (string(.appname) ?? "") + _pid + ": " + _msg
.uid = uuid_v4()
```

Puis un transform `raw_only` extrait `{uid, time, tenant_id, datasource_id, raw}`
et un sink separe l'envoie vers `${QUICKWIT_ENDPOINT}/api/v1/raw-logs/ingest`.

Le `uid` permet a Kontrol de correler un event OCSF normalise avec sa ligne brute
originale (utile pour l'investigation : "afficher le log brut de cette detection").

## Routage dynamique quand `class_uid` varie

Si une source produit plusieurs classes OCSF (ex: auditd = 1003 + 3002,
windows-evtx = 3001/3002/1003), il faut un transform `route` + un sink par
index Quickwit cible. Un sink unique vers `ocsf-endpoint` avec des events 3002
dedans = donnees au mauvais endroit. Voir [catalog/linux/auditd/vector.toml](../catalog/linux/auditd/vector.toml).
