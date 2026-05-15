# Kolektor — Vector.dev OCSF Config Catalog

![CI](https://github.com/komrad-company/Kolektor/actions/workflows/ci.yml/badge.svg) ![Release](https://img.shields.io/github/v/release/komrad-company/Kolektor) ![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue)

> *"Collection is not optional. The collective sees everything, or it sees nothing."*
> — Komrad Engineering Collective, May 2026

Catalogue of [Vector.dev](https://vector.dev) pipeline configurations for log normalisation to [OCSF](https://schema.ocsf.io). Each configuration is a deployable data source — consumed by the Komrad stack and manageable via Kontrol.

## Architecture

```
┌──────────┐     ┌──────────────────────────┐     ┌──────────┐     ┌───────────┐
│  Sources  │────→│  Vector (catalog config)  │────→│ Quickwit  │────→│ Korelator │
│ syslog,   │     │  VRL parse + OCSF norm    │     │ index by  │     │ correlation│
│ JSON, file│     │  1 pod per datasource     │     │ class     │     │ + alerts   │
└──────────┘     └──────────────────────────┘     └──────────┘     └───────────┘
```

**K8s deployment**: one Docker image contains Vector and the full catalogue.
The Vector pod is controlled by `Kolektor-kontroler`, which reads `catalog/index.json`
and applies active parsers via gRPC.

```
Kontrol-ui → Kontrol-api → gRPC Kolektor-kontroler → Vector --watch-config → Quickwit
```

## Repository structure

```
kolektor/
│
├── Dockerfile              # Vector image + embedded catalogue
├── entrypoint.sh           # Selects the config via $SOURCE_TYPE
├── .github/workflows/ci.yml # Pipeline: validate → test → coverage → build → push ghcr.io
│
├── _schema/
│   ├── template.toml       # Commented empty template (copy for new source)
│   └── README.md           # Step-by-step contribution guide
│
├── catalog/                # All Vector configs by category
│   ├── network/            # Firewalls, NDR → OCSF 4001 Network Activity
│   │   ├── opnsense/       #   filterlog CSV via syslog
│   │   ├── fortinet-fortigate/ # key=value via syslog
│   │   └── suricata-eve/   #   EVE JSON alert/dns/http/flow
│   │
│   ├── endpoint/           # EDR → OCSF 1003 Process / 2001 Finding
│   │   └── crowdstrike-falcon/ # JSON via SIEM Connector syslog
│   │
│   ├── identity/           # IdP/AD → OCSF 3002 Auth / 3001 Account
│   │   ├── microsoft-entra/       # Graph signIns/directoryAudits via HTTP push
│   │   ├── windows-security-evtx/ # Winlogbeat JSON (4624/4625/4688...)
│   │   └── windows-sysmon/        # Winlogbeat JSON (events 1/3/7/11/22)
│   │
│   ├── linux/              # Native Linux logs
│   │   ├── syslog/         #   RFC 3164/5424 via syslog TCP → OCSF 6001
│   │   ├── auditd/         #   audit.log key=value → OCSF 1003/3002
│   │   └── auth-log/       #   auth.log SSH/sudo/PAM → OCSF 3002
│   │
│   ├── cloud/              # Cloud providers → OCSF 6001 API Activity
│   │   ├── aws-cloudtrail/ # JSON via HTTP
│   │   └── microsoft-365-audit/ # Unified Audit JSONL via HTTP push
│   │
│   └── web/                # Web servers / edge → OCSF 4002 HTTP Activity
│       ├── nginx/          #   combined access log via file
│       ├── traefik/        #   access log JSON via file
│       └── cloudflare-http/ #  HTTP Requests Logpull / export JSONL
│
└── ci/                     # CI scripts
    ├── catalog_index.py    # Generates catalog/index.json
    ├── validate.sh         # vector validate on each config
    ├── test.sh             # vector test (merge config + tests/*.toml)
    ├── coverage.sh         # Enforces >= 3 tests per source
    └── report.py           # Generates markdown CI report
```

## Each source contains

```
catalog/<category>/<source>/
├── vector.toml             # Full config: source + VRL transform + sink
├── tests/
│   ├── nominal.toml        # Standard event, all fields present
│   ├── edge_case.toml      # Optional fields missing or boundary values
│   └── malformed.toml      # Invalid input → raw-logs with parse_status=failed
└── README.md               # Doc: format, source config, OCSF mapping, links
```

## Runtime environment variables

| Variable            | Required | Description                          | Example                           |
|---------------------|----------|--------------------------------------|-----------------------------------|
| `SOURCE_TYPE`       | yes      | Catalogue path (category/source)     | `network/opnsense`                |
| `SOURCE_TYPES`      | no       | CSV list for multi-parser mode       | `network/opnsense,linux/syslog`   |
| `TENANT_ID`         | yes      | Tenant identifier (multi-tenant)     | `tenant-acme`                     |
| `DATASOURCE_ID`     | yes      | Unique datasource identifier         | `ds-opnsense-hq`                  |
| `QUICKWIT_ENDPOINT` | yes      | Quickwit URL                         | `http://quickwit-searcher:7280`   |
| `LISTEN_PORT`       | no       | Listening port override              | `5140`                            |

## Docker image

```bash
# Local build
docker build -t kolektor .

# Run with a specific source
docker run -e SOURCE_TYPE=linux/syslog \
           -e TENANT_ID=acme \
           -e DATASOURCE_ID=ds-syslog-01 \
           -e QUICKWIT_ENDPOINT=http://quickwit:7280 \
           -p 5140:514 \
           kolektor

# List available sources
docker run kolektor
```

The image is built by GitHub Actions (docker/build-push-action) and pushed to `ghcr.io/komrad-company/kolektor` on every merge to `main` (tags `latest` + `<sha>`).


## Target Quickwit indexes

| Index            | OCSF class                 | category_uid | Typical sources                    |
|------------------|----------------------------|--------------|------------------------------------|
| `ocsf-network`   | 4001 Network Activity      | 4            | opnsense, fortigate, suricata flow |
| `ocsf-http`      | 4002 HTTP Activity         | 4            | nginx, traefik, cloudflare, suricata HTTP |
| `ocsf-dns`       | 4003 DNS Activity          | 4            | unbound, sysmon DNS, suricata DNS  |
| `ocsf-endpoint`  | 1001/1003 File/Process     | 1            | crowdstrike, sysmon, auditd, suricata alerts |
| `ocsf-identity`  | 3001/3002 Account/Auth     | 3            | entra sign-ins, windows-evtx, auth-log |
| `ocsf-audit`     | 6001 API Activity          | 6            | cloudtrail, m365 audit, entra directory audits, syslog |
| `ocsf-k8s`       | 6003 Kubernetes API Activity | 6          | kubernetes-audit                   |

## CI pipeline

| Stage    | Description                                      | Image                     |
|----------|--------------------------------------------------|---------------------------|
| catalog  | Validates `catalog/index.json`                   | ubuntu-latest + python    |
| validate | `vector validate` on each vector.toml            | vector:0.54.0-debian      |
| test     | `vector test` (config + tests merged)            | vector:0.54.0-debian      |
| coverage | Enforces >= 3 tests per source                   | vector:0.54.0-debian      |
| report   | Generates markdown report as artifact            | ubuntu-latest + python    |
| build    | docker/build-push-action → ghcr.io (main only)  | ubuntu-latest             |

## Contributing

1. Copy `_schema/template.toml` → `catalog/<category>/<source>/vector.toml`
2. Write the VRL parsing and OCSF normalisation logic
3. Add >= 3 tests in `tests/` using real raw log samples
4. Add a `README.md` documenting the source
5. Run `ci/catalog_index.py` + `ci/validate.sh` + `ci/test.sh` to validate locally
6. Push — CI validates automatically

### Raw / parsing convention

- A parsed event goes to its OCSF index and always retains the original log in the `raw` field.
- An unparsed event does not go to an OCSF index: it is sent to `raw-logs` with `parse_status = "failed"`, `source_type`, `parser`, `parse_error`, `raw`, `uid`, `tenant_id`, and `datasource_id`.
- Multi-class parsers declare their outputs in `manifest.yaml` via `ocsf_outputs` and route each class to its Quickwit index.

### Collector / parser convention

- The Vector parser handles normalisation only: canonical raw format in, minimal enrichment, OCSF mapping, Quickwit routing.
- Cloud/SaaS log retrieval is the responsibility of a collector: API pull with cursor, object storage + queue, Event Hub/EventBridge, or HTTP Logpush when the provider supports it.
- For cloud sources, the recommended canonical format is newline-delimited JSON. The same parser must be able to process lines from either an API collector or a pushed export, provided the source schema is identical.

See `_schema/README.md` for the full contribution guide.

## License

AGPL-3.0-or-later — the source remains open, as all things should be.
