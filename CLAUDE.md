# CLAUDE.md — Kolektor

## Projet
Catalogue de configurations Vector.dev pour la normalisation de logs en OCSF.
Matiere premiere du SOC UI : chaque config est une source de donnees deployable par tenant.

## Structure
```
kolektor/
├── _schema/            # Template + guide contribution
├── catalog/
│   ├── network/        # Firewalls (OCSF 4001)
│   ├── endpoint/       # EDR (OCSF 1001/1003/2001)
│   ├── identity/       # Windows EVTX / Sysmon (OCSF 3001/3002)
│   ├── linux/          # syslog, auditd, auth-log
│   ├── cloud/          # CloudTrail (OCSF 6001)
│   └── web/            # nginx (OCSF 4001)
├── api/                # Workspace Cargo : kolektor-api (bin), kolektor-common, kolektor-seed
│   ├── Cargo.toml
│   ├── crates/
│   └── migrations/     # sqlx migrations (schema PG `kolektor`)
├── ci/                 # Scripts CI Vector (validate, test, coverage, report)
├── Dockerfile          # Multi-stage : build Rust puis runtime Vector + kolektor-api
└── .github/workflows/  # GitHub Actions : Vector CI + Rust CI → ghcr.io/komrad-company/kolektor
```

## API REST — architecture
- **Binaire unique** `kolektor-api` avec 3 subcommands : `init`, `serve`, `token`.
- **1 pod** dans la namespace `kolektor` (voir argocd repo) :
  - `initContainer` (`kolektor-api init`) : migrate + seed catalog → DB + ecrit le fichier Vector initial sur emptyDir partage.
  - `api` (`kolektor-api serve`) : Axum sur :8080, auth Bearer token bcrypt. Sur `PUT /v1/parsers/{cat}/{name}/enabled`, reecrit atomiquement `/etc/vector/kolektor/sources.toml`.
  - `vector` : `--watch-config` sur `/etc/vector/kolektor/sources.toml`, ports 5140-5143.
- **Source de verite** : PostgreSQL (schema `kolektor` sur l'instance partagee Kontrol). Tables `parsers`, `api_tokens`, `sync_events`.
- **Catalog Git = seed** : les `catalog/*/*/vector.toml` sont importes en DB au premier `init` (UPSERT preserve `enabled`, incremente `version` si le TOML change).
- **Endpoints v1** :
  - `GET  /v1/health` (no auth)
  - `GET  /v1/status` · `GET /v1/parsers` · `GET /v1/parsers/{cat}/{name}` · `PUT /v1/parsers/{cat}/{name}/enabled` (Bearer token)
- **Idempotence** : `PUT enabled=true` repete = meme reponse, un seul `sync_event`, pas de reload Vector parasite (compat Terraform future).

## Bootstrap token API
Apres le premier deploiement :
```bash
kubectl -n kolektor exec -it deploy/kolektor -c api -- \
  kolektor-api token create --name bootstrap --tenant-id bibihome
```
Le secret est affiche une seule fois ; a stocker cote client (ex: Kontrol secret store).
Usage : `curl -H "Authorization: Bearer <token>" https://kolektor-api.bibihome.lan/v1/parsers`.

## Conventions
- **Format** : TOML pour les configs Vector, VRL inline dans les transforms
- **Tests** : minimum 3 par source (nominal, optional_missing, malformed)
- **Logs de test** : bruts reels, pas inventes
- **Variables runtime** : `${TENANT_ID}`, `${DATASOURCE_ID}`, `${LISTEN_PORT}`, `${QUICKWIT_ENDPOINT}`
- **Champs OCSF obligatoires** : `class_uid`, `category_uid`, `severity_id`, `time`, `metadata`, `tenant_id`, `datasource_id`, `raw`
- **Commits** : `feat:`, `fix:`, `test:`, `ci:`, `docs:`
- **Langue** : francais pour docs et commentaires

## Index Quickwit
| Index            | Classe OCSF         | category_uid |
|------------------|----------------------|--------------|
| `ocsf-network`   | 4001 Network Activity | 4           |
| `ocsf-endpoint`  | 1001/1003 File/Proc  | 1           |
| `ocsf-identity`  | 3001/3002 Acct/Auth  | 3           |
| `ocsf-audit`     | 6001 API Activity    | 6           |

## CI
- `vector validate` sur chaque `vector.toml` (avec dummy env vars injectes par `ci/validate.sh`)
- `vector test` sur chaque source (merge vector.toml + tests/*.toml)
- Rapport markdown en artifact
- Image CI : `timberio/vector:0.54.0-debian`

## Pieges connus
- `vector test` necessite que les tests soient dans le meme fichier ou passes en argument avec le config
- Vector 0.54 `--no-environment` desactive le reload sur changement d'env, il n'ignore PAS l'expansion `${VAR}` : ci/validate.sh injecte donc des dummy vars
- VRL : pas d'`if` inline dans les object literals, pre-calculer les valeurs
- Les sources `syslog` Vector parsent automatiquement le header syslog (timestamp, hostname, appname)
- Pour les tests, utiliser `type = "log"` avec `insert_at` sur le transform
- Quand class_uid est dynamique (auditd, windows-evtx, crowdstrike), utiliser un transform `route` + un sink par index OCSF
