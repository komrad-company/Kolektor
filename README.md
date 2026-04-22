# Kolektor — Vector.dev OCSF Config Catalog

Catalogue de configurations [Vector.dev](https://vector.dev) pour la normalisation de logs en [OCSF](https://schema.ocsf.io).
Matiere premiere du SOC UI : chaque config est une source de donnees deployable par tenant.

## Architecture

```
┌──────────┐     ┌──────────────────────────┐     ┌──────────┐     ┌───────────┐
│  Sources  │────→│  Vector (catalog config)  │────→│ Quickwit  │────→│ Kolektor  │
│ syslog,   │     │  parse VRL + OCSF norm    │     │ index par │     │ correlation│
│ JSON, file│     │  1 pod par datasource     │     │ classe    │     │ + alertes  │
└──────────┘     └──────────────────────────┘     └──────────┘     └───────────┘
```

**Deploiement K8s** : 1 image Docker contient Vector + tout le catalog.
Au runtime, `SOURCE_TYPE` selectionne la config. Chaque datasource = 1 Deployment.

```
SOC UI → API Rust → commit Git → ArgoCD → Deployment Vector (SOURCE_TYPE=xxx) → Quickwit
```

## Structure du repo

```
kolektor/
│
├── Dockerfile              # Image Vector + catalog embarque
├── entrypoint.sh           # Selectionne la config via $SOURCE_TYPE
├── .github/workflows/ci.yml # Pipeline : validate → test → coverage → build → push ghcr.io
│
├── _schema/
│   ├── template.toml       # Template vide commente (copier pour nouvelle source)
│   └── README.md           # Guide de contribution pas-a-pas
│
├── catalog/                # Toutes les configs Vector par categorie
│   ├── network/            # Firewalls, NDR → OCSF 4001 Network Activity
│   │   ├── opnsense/       #   filterlog CSV via syslog
│   │   ├── fortinet-fortigate/ # key=value via syslog
│   │   └── suricata-eve/   #   EVE JSON alert/dns/http/flow
│   │
│   ├── endpoint/           # EDR → OCSF 1003 Process / 2001 Finding
│   │   └── crowdstrike-falcon/ # JSON via SIEM Connector syslog
│   │
│   ├── identity/           # IdP/AD → OCSF 3002 Auth / 3001 Account
│   │   ├── microsoft-entra/       # Graph signIns/directoryAudits via fetcher
│   │   ├── windows-security-evtx/ # Winlogbeat JSON (4624/4625/4688...)
│   │   └── windows-sysmon/        # Winlogbeat JSON (events 1/3/7/11/22)
│   │
│   ├── linux/              # Logs Linux natifs
│   │   ├── syslog/         #   RFC 3164/5424 via syslog TCP → OCSF 6001
│   │   ├── auditd/         #   audit.log key=value → OCSF 1003/3002
│   │   └── auth-log/       #   auth.log SSH/sudo/PAM → OCSF 3002
│   │
│   ├── cloud/              # Cloud providers → OCSF 6001 API Activity
│   │   ├── aws-cloudtrail/ # JSON via HTTP
│   │   └── microsoft-365-audit/ # Unified Audit JSONL via fetcher
│   │
│   └── web/                # Serveurs web / edge → OCSF 4002 HTTP Activity
│       ├── nginx/          #   combined access log via file
│       ├── traefik/        #   access log JSON via file
│       └── cloudflare-http/ #  HTTP Requests Logpull / export JSONL
│
└── ci/                     # Scripts CI
    ├── validate.sh         # vector validate sur chaque config
    ├── test.sh             # vector test (merge config + tests/*.toml)
    ├── coverage.sh         # Verifie min 3 tests par source
    └── report.py           # Genere rapport markdown CI
```

## Chaque source contient

```
catalog/<category>/<source>/
├── vector.toml             # Config complete : source + transform VRL + sink
├── tests/
│   ├── nominal.toml        # Event standard, tous champs presents
│   ├── edge_case.toml      # Champs optionnels manquants ou valeurs limites
│   └── malformed.toml      # Input invalide → raw-logs avec parse_status=failed
└── README.md               # Doc : format, config source, mapping OCSF, liens
```

## Variables d'environnement runtime

| Variable            | Obligatoire | Description                          | Exemple                           |
|---------------------|-------------|--------------------------------------|-----------------------------------|
| `SOURCE_TYPE`       | oui         | Chemin catalog (categorie/source)    | `network/opnsense`                |
| `TENANT_ID`         | oui         | ID du tenant (multi-tenant)          | `tenant-acme`                     |
| `DATASOURCE_ID`     | oui         | ID unique de la datasource           | `ds-opnsense-hq`                  |
| `QUICKWIT_ENDPOINT` | oui         | URL Quickwit                         | `http://quickwit-searcher:7280`   |
| `LISTEN_PORT`       | non         | Override du port d'ecoute            | `5140`                            |

## Image Docker

```bash
# Build local
docker build -t kolektor .

# Run avec une source specifique
docker run -e SOURCE_TYPE=linux/syslog \
           -e TENANT_ID=acme \
           -e DATASOURCE_ID=ds-syslog-01 \
           -e QUICKWIT_ENDPOINT=http://quickwit:7280 \
           -p 5140:514 \
           kolektor

# Lister les sources disponibles
docker run kolektor
```

L'image est buildee par GitHub Actions (docker/build-push-action) et pushee sur `ghcr.io/komrad-company/kolektor` a chaque merge sur `main` (tags `latest` + `<sha>`).

## Deploiement K8s (ArgoCD)

Chaque datasource = 1 Deployment dans `infrastructure/kolektor-collector/` (repo argocd) :

```yaml
env:
  - name: SOURCE_TYPE
    value: "linux/syslog"        # ← selectionne la config du catalog
  - name: TENANT_ID
    value: "acme"
  - name: DATASOURCE_ID
    value: "ds-syslog-01"
  - name: QUICKWIT_ENDPOINT
    value: "http://quickwit-searcher.quickwit:7280"
```

ArgoCD sync automatique → pod Vector pret a recevoir.

## Index Quickwit cibles

| Index            | Classe OCSF                | category_uid | Sources typiques               |
|------------------|----------------------------|--------------|--------------------------------|
| `ocsf-network`   | 4001 Network Activity      | 4            | opnsense, fortigate, suricata flow |
| `ocsf-http`      | 4002 HTTP Activity         | 4            | nginx, traefik, cloudflare, suricata HTTP |
| `ocsf-dns`       | 4003 DNS Activity          | 4            | unbound, sysmon DNS, suricata DNS |
| `ocsf-endpoint`  | 1001/1003 File/Process     | 1            | crowdstrike, sysmon, auditd, suricata alerts |
| `ocsf-identity`  | 3001/3002 Account/Auth     | 3            | entra sign-ins, windows-evtx, auth-log |
| `ocsf-audit`     | 6001 API Activity          | 6            | cloudtrail, m365 audit, entra directory audits, syslog |
| `ocsf-k8s`       | 6003 Kubernetes API Activity | 6          | kubernetes-audit               |

## CI Pipeline

| Stage    | Description                                   | Image                     |
|----------|-----------------------------------------------|---------------------------|
| validate | `vector validate` sur chaque vector.toml      | vector:0.54.0-debian      |
| test     | `vector test` (config + tests merges)         | vector:0.54.0-debian      |
| coverage | Verifie >= 3 tests par source                 | vector:0.54.0-debian      |
| report   | Genere rapport markdown en artifact           | ubuntu-latest + python 3.12 |
| build    | docker/build-push-action → ghcr.io (main)     | ubuntu-latest             |

## Contribuer

1. Copier `_schema/template.toml` → `catalog/<category>/<source>/vector.toml`
2. Ecrire le VRL de parsing + normalisation OCSF
3. Ajouter >= 3 tests dans `tests/` avec des logs bruts reels
4. Ajouter un `README.md` documentant la source
5. `ci/validate.sh` + `ci/test.sh` pour valider localement
6. Push → CI valide automatiquement

### Convention raw / parsing

- Un evenement parse va dans son index OCSF et conserve toujours le log source dans le champ `raw`.
- Un evenement non parse ne va pas dans un index OCSF : il est envoye dans `raw-logs` avec `parse_status = "failed"`, `source_type`, `parser`, `parse_error`, `raw`, `uid`, `tenant_id` et `datasource_id`.
- Les parsers multi-classes declarent leurs sorties dans `manifest.yaml` via `ocsf_outputs`, et routent chaque classe vers son index Quickwit.

### Convention collecteur / parser

- Le parser Vector ne porte que la normalisation : format brut canonique en entree, enrichissement minimal, mapping OCSF, routage Quickwit.
- La recuperation des logs cloud/SaaS est la responsabilite d'un collecteur : API pull avec curseur, object storage + queue, Event Hub/EventBridge, ou Logpush HTTP quand le fournisseur sait pousser.
- Pour les sources cloud, le format canonique recommande est JSON line-delimited. Le meme parser doit pouvoir traiter les lignes issues d'un collecteur API ou d'un export pousse si le schema source reste identique.

Voir `docs/FETCHERS.md` pour la configuration des fetchers pull.

Voir `_schema/README.md` pour le guide complet.
