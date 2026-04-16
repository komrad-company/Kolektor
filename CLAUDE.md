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
├── ci/                 # Scripts CI (validate, test, coverage, report)
└── .github/workflows/  # GitHub Actions → ghcr.io/komrad-company/kolektor
```

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
