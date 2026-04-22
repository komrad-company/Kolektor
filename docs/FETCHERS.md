# Kolektor Fetchers

Les fetchers recuperent les logs cloud/SaaS en **pull** et ecrivent du JSON
line-delimited dans des fichiers surveilles par Vector. L'objectif est de ne pas
exposer de port entrant pour recevoir des logs Internet.

## Architecture

```
UI/Kontrol -> kolektor-api /v1/fetchers -> DB
                                     |
                                     v
kolektor-fetcher run -> API/S3 pull -> fichier JSONL -> Vector file source -> OCSF -> Quickwit
```

La configuration est stockee dans `kolektor.fetchers`. Le champ
`parser_source_type` lie explicitement un fetcher au parser qui consomme son
fichier de sortie.

## API

Endpoints authentifies :

- `GET /v1/fetchers`
- `POST /v1/fetchers`
- `GET /v1/fetchers/{id}`
- `PUT /v1/fetchers/{id}`
- `PUT /v1/fetchers/{id}/enabled`
- `DELETE /v1/fetchers/{id}`

Champs principaux :

| Champ                | Description |
|----------------------|-------------|
| `provider`           | `microsoft_graph`, `microsoft365_management` ou `s3` |
| `parser_source_type` | Parser cible, ex. `identity/microsoft-entra` |
| `output_path`        | Fichier JSONL ecrit par le fetcher et lu par Vector |
| `interval_seconds`   | Frequence de polling, minimum 30 secondes |
| `config`             | Configuration provider-specific |
| `state`              | Curseurs geres par `kolektor-fetcher` |

Eviter de stocker les secrets dans `config`. Preferer les champs `*_env` qui
referencent une variable d'environnement montee dans le pod fetcher.

## Microsoft Entra ID

Provider : `microsoft_graph`

Exemple sign-ins :

```json
{
  "name": "entra-signins",
  "provider": "microsoft_graph",
  "parser_source_type": "identity/microsoft-entra",
  "enabled": true,
  "interval_seconds": 300,
  "output_path": "/var/lib/kolektor/fetcher/microsoft-entra.jsonl",
  "config": {
    "tenant_id": "00000000-0000-0000-0000-000000000000",
    "client_id": "11111111-1111-1111-1111-111111111111",
    "client_secret_env": "MSGRAPH_CLIENT_SECRET",
    "kind": "signins",
    "lookback_minutes": 15,
    "safety_lag_seconds": 120
  }
}
```

Exemple directory audits :

```json
{
  "name": "entra-directory-audits",
  "provider": "microsoft_graph",
  "parser_source_type": "identity/microsoft-entra",
  "enabled": true,
  "interval_seconds": 300,
  "output_path": "/var/lib/kolektor/fetcher/microsoft-entra.jsonl",
  "config": {
    "tenant_id": "00000000-0000-0000-0000-000000000000",
    "client_id": "11111111-1111-1111-1111-111111111111",
    "client_secret_env": "MSGRAPH_CLIENT_SECRET",
    "kind": "directory_audits"
  }
}
```

Permission Microsoft Graph : `AuditLog.Read.All` en application permission.

## Microsoft 365 Unified Audit

Provider : `microsoft365_management`

```json
{
  "name": "m365-unified-audit",
  "provider": "microsoft365_management",
  "parser_source_type": "cloud/microsoft-365-audit",
  "enabled": true,
  "interval_seconds": 300,
  "output_path": "/var/lib/kolektor/fetcher/microsoft-365-audit.jsonl",
  "config": {
    "tenant_id": "contoso.onmicrosoft.com",
    "client_id": "11111111-1111-1111-1111-111111111111",
    "client_secret_env": "M365_ACTIVITY_CLIENT_SECRET",
    "content_types": [
      "Audit.AzureActiveDirectory",
      "Audit.Exchange",
      "Audit.SharePoint",
      "Audit.General"
    ],
    "publisher_identifier": "22222222-2222-2222-2222-222222222222",
    "ensure_subscriptions": false,
    "lookback_minutes": 15,
    "safety_lag_seconds": 120
  }
}
```

Permission Office 365 Management APIs : `ActivityFeed.Read`.

## S3 / S3-Compatible

Provider : `s3`

```json
{
  "name": "cloudflare-r2-http",
  "provider": "s3",
  "parser_source_type": "web/cloudflare-http",
  "enabled": true,
  "interval_seconds": 300,
  "output_path": "/var/lib/kolektor/fetcher/cloudflare-http.jsonl",
  "config": {
    "bucket": "kolektor-logs",
    "prefix": "cloudflare/http/",
    "region": "auto",
    "endpoint": "https://accountid.r2.cloudflarestorage.com",
    "access_key_id_env": "S3_ACCESS_KEY_ID",
    "secret_access_key_env": "S3_SECRET_ACCESS_KEY",
    "force_path_style": true,
    "max_objects": 100
  }
}
```

Le fetcher lit les objets dans l'ordre lexicographique et conserve
`state.cursors["s3:last_key"]`. Il supporte les objets NDJSON, les tableaux JSON
et les objets `.gz`.

## Limites du premier increment

- Pas encore de verrou distribue : lancer un seul replica `kolektor-fetcher`.
- Le mode S3 suit `last_key`, donc il suppose des cles monotones par date/heure.
- Les connecteurs Microsoft utilisent les curseurs temporels et une marge de
  securite (`safety_lag_seconds`) pour absorber la latence d'indexation.
- Le fetcher M365 suit la pagination `NextPageUri` et limite chaque fenetre de
  polling a 24h maximum, comme recommande par l'API.
