# CI — Architecture d'execution

Deux workflows GitHub Actions independants.

## Branching model

- `develop` is the working branch. Feature branches are opened from `develop` and merged back into `develop`.
- `main` is the release branch. Merging `develop` into `main` publishes the Docker image.
- CI and SAST run on every pushed branch and on every pull request.

## 1. `ci.yml` — Build & Publish

Chemin critique : si un maillon casse, pas d'image publiee.

```
catalog index ─────┐
validate (Vector) ─┤
test (Vector)      ├── publish (Docker push GHCR) [main only]
coverage (Vector)  │
                   └── report (rapport markdown Vector)
```

- **catalog** : `python3 ci/catalog_index.py --check` verifie que `catalog/index.json` est synchronise avec `catalog/*/*/manifest.yaml` et `vector.toml`.
- **Jobs Vector** (`validate`, `test`, `coverage`, `report`) : scripts `ci/*.sh`, conteneur `timberio/vector:0.54.0-debian`.
- **publish** : reusable Docker publish workflow -> `ghcr.io/komrad-company/kolektor:{sha,latest}`, declenche uniquement sur `main`.

## 2. `security.yml` — SAST

Jobs independants, sans `needs:`.

| Job | Workflow reutilisable | Gate |
|---|---|---|
| `secrets` | `security-secrets.yml` (gitleaks) | exit-code 1 |
| `docker` | `security-docker.yml` (hadolint + grype) | hadolint error / grype HIGH+ only-fixed |

`security-docker.yml` : hadolint + grype sur l'image construite depuis `Dockerfile`.

Declencheurs : `push` sur toutes les branches, `pull_request`, cron `0 3 * * 1` (lundi 3h UTC), `workflow_dispatch`.

## Conventions

- `actions/checkout@v5`, `actions/upload-artifact@v5`.
- Runners : `komrad-runners` quand les workflows reutilisables les utilisent.

## Gating cote branch protection

Les jobs CI + SAST exposent des status checks que les branch protection rules de `develop` et `main` peuvent exiger avant merge. Les workflows sont separes pour permettre de re-run un SAST sans rebuilder l'image.
