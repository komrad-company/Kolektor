# CLAUDE.md — Kolektor

## Projet
Catalogue de configurations Vector.dev pour la normalisation de logs en OCSF.
Matiere premiere du SOC UI : chaque config est une source de donnees deployable par tenant.

## Structure
```
kolektor/
├── _schema/            # Template + guide contribution
├── catalog/            # Configs Vector + catalog/index.json genere
│   ├── network/        # Firewalls (OCSF 4001)
│   ├── endpoint/       # EDR (OCSF 1001/1003/2001)
│   ├── identity/       # Windows EVTX / Sysmon (OCSF 3001/3002)
│   ├── linux/          # syslog, auditd, auth-log
│   ├── cloud/          # CloudTrail (OCSF 6001)
│   └── web/            # nginx, traefik, cloudflare
├── ci/                 # Scripts CI Vector + generation catalog/index.json
├── Dockerfile          # Runtime Vector + catalogue embarque
└── .github/workflows/  # GitHub Actions : catalogue, Vector CI, Docker publish
```

## Architecture cible
- `Kolektor` livre Vector et le catalogue local.
- `Kolektor-kontroler`, dans le pod Vector, lit `/etc/vector/catalog/index.json`, expose le catalogue en gRPC et applique les parsers actifs.
- `Kontrol-api` orchestre l'etat desire cote BFF et appelle `Kolektor-kontroler`.
- Aucune API REST Rust ne vit dans ce repo.

## Catalogue
- `catalog/*/*/manifest.yaml` contient les metadonnees humaines.
- `catalog/*/*/vector.toml` contient le bloc Vector applique par le controleur.
- `catalog/index.json` est genere par `ci/catalog_index.py` et doit rester synchronise avec les manifests.

## Conventions
- **Format** : TOML pour les configs Vector, VRL inline dans les transforms
- **Tests** : minimum 3 par source (nominal, optional_missing, malformed)
- **Logs de test** : bruts reels, pas inventes
- **Variables runtime** : `${TENANT_ID}`, `${DATASOURCE_ID}`, `${LISTEN_PORT}`, `${QUICKWIT_ENDPOINT}`
- **Champs OCSF obligatoires** : `class_uid`, `category_uid`, `severity_id`, `time`, `metadata`, `tenant_id`, `datasource_id`, `raw`
- **Commits** : `feat:`, `fix:`, `test:`, `ci:`, `docs:`
- **Langue** : francais pour docs et commentaires

## CI
- `python3 ci/catalog_index.py --check`
- `vector validate` sur chaque `vector.toml` avec dummy env vars injectes par `ci/validate.sh`
- `vector test` sur chaque source (merge vector.toml + tests/*.toml)
- Rapport markdown en artifact
- Image CI : `timberio/vector:0.54.0-debian`
- SAST : gitleaks + hadolint + grype (voir `docs/CI.md`)

## Pieges connus
- `vector test` necessite que les tests soient dans le meme fichier ou passes en argument avec le config
- Vector 0.54 `--no-environment` desactive le reload sur changement d'env, il n'ignore PAS l'expansion `${VAR}` : ci/validate.sh injecte donc des dummy vars
- VRL : pas d'`if` inline dans les object literals, pre-calculer les valeurs
- Les sources `syslog` Vector parsent automatiquement le header syslog (timestamp, hostname, appname)
- Pour les tests, utiliser `type = "log"` avec `insert_at` sur le transform
- Quand class_uid est dynamique (auditd, windows-evtx, crowdstrike), utiliser un transform `route` + un sink par index OCSF
