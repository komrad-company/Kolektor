# CI — Architecture d'exécution

Trois workflows GitHub Actions indépendants, déclenchés en parallèle.

## 1. `ci.yml` — Build & Publish

Chemin critique : si un maillon casse, pas d'image publiée.

```
validate (Vector) ──┐
test (Vector)       ├── build (Docker push GHCR) [main only]
coverage (Vector)   │
rust-checks ────────┘
                    └── report (rapport markdown Vector)
```

- **Jobs Vector** (`validate`, `test`, `coverage`, `report`) : scripts `ci/*.sh`, image `timberio/vector:0.54.0-debian`
- **rust-checks** : `cargo fmt --check` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` + `cargo build --release` (Rust 1.94, edition 2024, cache `Swatinem/rust-cache`)
- **build** : `docker/build-push-action@v6` → `ghcr.io/komrad-company/kolektor:{sha,latest}`, déclenché uniquement sur `main`

## 2. `security.yml` — SAST

Jobs **indépendants** (pas de `needs:` entre eux), tournent en parallèle.

| Job | Gate | Config |
|---|---|---|
| `gitleaks` | exit-code 1 | — |
| `hadolint` | failure-threshold: error | `Dockerfile` racine |
| `grype image` | HIGH+, only-fixed | `anchore/scan-action@v7.4.0` |
| `cargo-audit` | RustSec fail | `api/.cargo/audit.toml` |
| `cargo-deny` | licences + bans + advisories | `api/deny.toml` |

Déclencheurs : `push main`, `pull_request`, cron `0 3 * * 1` (lundi 3h UTC), `workflow_dispatch`.

Chaque job uploade un artifact de reporting (SARIF + JSON), rétention 30j.

## 3. `codeql.yml` (géré GitHub)

Analyse statique Rust + JS, indépendante. Upload SARIF dans l'onglet Security.

## Conventions partagées avec Kontrol

- `RUST_VERSION: "1.94"` (env var)
- `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"` (contourne deprecation Node 20 des actions GA)
- `actions/checkout@v5`, `actions/upload-artifact@v5`
- Même version Grype (`anchore/scan-action@v7.4.0`), même format hadolint (SARIF + JSON)

## Ignores advisories documentés

`api/.cargo/audit.toml` et `api/deny.toml` contiennent les `ignore` justifiés (commentaires). Revoir à chaque bump sqlx major.

## Gating côté branch protection

Les jobs CI + SAST exposent des status checks que la branch protection rule de `main` peut exiger avant merge. Les workflows sont séparés pour permettre de re-run un SAST sans rebuilder l'image.
