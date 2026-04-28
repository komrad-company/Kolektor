# CI — Architecture d'exécution

Deux workflows GitHub Actions indépendants, déclenchés en parallèle sur `komrad-runners` (ARC K8s self-hosted).

## 1. `ci.yml` — Build & Publish

Chemin critique : si un maillon casse, pas d'image publiée.

```
validate (Vector) ──┐
test (Vector)       ├── publish (Docker push GHCR) [main only]
coverage (Vector)   │
rust-checks ────────┘
                    └── report (rapport markdown Vector)
```

- **Jobs Vector** (`validate`, `test`, `coverage`, `report`) : scripts `ci/*.sh`, conteneur `timberio/vector:0.54.0-debian` (via `container:` job). `secrets: inherit` requis pour l'auth Docker Hub sur le pull de l'image.
- **rust-checks** : `cargo fmt --check` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` + `cargo build --release` (Rust 1.94, edition 2024, cache `Swatinem/rust-cache`)
- **publish** : `docker/build-push-action@v6` → `ghcr.io/komrad-company/kolektor:{sha,latest}`, déclenché uniquement sur `main`

## 2. `security.yml` — SAST

Jobs **indépendants** (pas de `needs:` entre eux), tournent en parallèle. `cancel-in-progress: false` (les scans sont longs, on ne les interrompt pas).

| Job | Workflow réutilisable | Gate |
|---|---|---|
| `secrets` | `security-secrets.yml` (gitleaks) | exit-code 1 |
| `rust` | `security-rust.yml` (cargo-audit + cargo-deny) | RustSec fail / licences + bans |
| `docker` | `security-docker.yml` (hadolint + grype) | hadolint error / grype HIGH+ only-fixed |

`security-docker.yml` : hadolint v2.12.0 (binaire direct) + grype v0.87.0 (binaire direct) sur l'image buildée depuis `Dockerfile`. Auth Docker Hub via `secrets: inherit` (`DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN`).

Exceptions hadolint documentées dans `.hadolint.yaml` à la racine.

Déclencheurs : `push main`, `pull_request`, cron `0 3 * * 1` (lundi 3h UTC), `workflow_dispatch`.

## Conventions

- `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"` : force l'exécution des actions JS sur Node 24 (warning cosmétique tant que les actions n'ont pas mis à jour leur `action.yml`)
- `actions/checkout@v5`, `actions/upload-artifact@v5`
- Runners : `komrad-runners` (ARC, DinD manuel, `--mtu=1380` pour Cilium WireGuard)

## Ignores advisories documentés

`api/.cargo/audit.toml` et `api/deny.toml` contiennent les `ignore` justifiés (commentaires). Revoir à chaque bump sqlx major.

## Gating côté branch protection

Les jobs CI + SAST exposent des status checks que la branch protection rule de `main` peut exiger avant merge. Les workflows sont séparés pour permettre de re-run un SAST sans rebuilder l'image.
