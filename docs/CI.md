# CI — Execution architecture

One workflow: `.github/workflows/ci.yml`. Security gates first, as decreed
by the collective — a failed audit blocks everything downstream.

## Branching model

- `develop` is the working branch. Feature branches are opened from `develop` and merged back into `develop`.
- `main` is the release branch.
- CI runs on pushes to `main`/`develop` and `v*` tags, on every pull request, weekly (Monday 03:00 UTC) and on `workflow_dispatch`.

## Pipeline

```
security (gitleaks) ──┬── container_sast (hadolint + grype)──┐
                      ├── catalog (index check)              ├── publish (container-pipeline)
                      └── vector (validate + test            │
                            + coverage + report)─────────────┘
```

| Job | What it does | Where it lives |
|---|---|---|
| `security` | gitleaks secret detection | reusable — `Kontinuous-integration/security-pipeline.yml@main` |
| `container_sast` | hadolint on `Dockerfile`, grype (`--fail-on high --only-fixed`) on the built image | inline |
| `catalog` | `python3 ci/catalog_index.py --check` — `catalog/index.json` in sync with manifests | inline |
| `vector` | `ci/validate.sh`, `ci/test.sh`, `ci/coverage.sh` in `timberio/vector:0.54.0-debian`, then `ci/report.py` markdown artifact | inline |
| `publish` | buildah build + push to `ghcr.io/komrad-company/kolektor` — PR: build only, `develop`: `sha-*` tag, `v*` tags: semver + `latest` | reusable — `Kontinuous-integration/container-pipeline.yml@main` |

Vector-specific logic stays inline in this repository: it serves no other
repo and does not belong in shared workflows. Shared concerns (secret
detection, container build/publish) are consumed from
`komrad-company/Kontinuous-integration`.

## Branch protection gating

All jobs expose status checks that the branch protection rules of `develop`
and `main` require before merge. The weekly schedule re-runs the full
pipeline so a newly published CVE fails `container_sast` without waiting
for a push.
