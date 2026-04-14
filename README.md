# Korelator Catalog — Vector.dev Config Catalog

Catalogue de configurations Vector.dev pour la normalisation de logs en OCSF.

## Architecture

```
Sources (syslog, fichiers, API) → Vector (parse + VRL) → Quickwit (index OCSF)
```

## Structure

```
catalog/
├── network/       # Firewalls, NDR (OCSF 4001 Network Activity)
├── endpoint/      # EDR, Sysmon (OCSF 1001/1003 File/Process Activity)
├── identity/      # AD, IdP (OCSF 3001/3002 Account/Authentication)
├── linux/         # syslog, auditd, auth-log
├── cloud/         # CloudTrail, GCP Audit (OCSF 6001 API Activity)
├── web/           # nginx, apache, haproxy
└── kubernetes/    # k8s audit, container logs
```

## Utilisation

Chaque source dans `catalog/<category>/<source>/` contient :
- `vector.toml` — config Vector complete (source + transform OCSF + sink Quickwit)
- `tests/*.toml` — cas de test `vector test` avec logs bruts reels
- `README.md` — documentation de la source

## Variables d'environnement runtime

| Variable           | Description                          | Exemple                     |
|--------------------|--------------------------------------|-----------------------------|
| `TENANT_ID`        | ID du tenant (multi-tenant)          | `tenant-acme`               |
| `DATASOURCE_ID`    | ID de la datasource instanciee       | `ds-opnsense-01`            |
| `LISTEN_PORT`      | Port d'ecoute (override du defaut)   | `514`                       |
| `QUICKWIT_ENDPOINT`| URL Quickwit                         | `http://quickwit:7280`      |

## CI

```bash
# Valider toutes les configs
ci/validate.sh

# Lancer tous les tests
ci/test.sh
```

## Index Quickwit cibles

| Index            | Classe OCSF                | Sources typiques          |
|------------------|----------------------------|---------------------------|
| `ocsf-network`   | 4001 Network Activity      | Firewalls, proxies, NDR   |
| `ocsf-endpoint`  | 1001/1003 File/Process     | EDR, Sysmon, auditd       |
| `ocsf-identity`  | 3001/3002 Account/Auth     | AD, FreeIPA, Okta         |
| `ocsf-audit`     | 6001 API Activity          | CloudTrail, GCP, K8s      |

## Contribuer

Voir `_schema/README.md` pour le guide de contribution.
