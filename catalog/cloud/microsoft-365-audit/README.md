# Microsoft 365 Unified Audit — Management Activity API

## Description

Normalise les evenements Microsoft 365 Unified Audit vers OCSF 6001 API Activity.

Le parser expose un endpoint HTTP sur `${LISTEN_PORT:-8517}`. Le fetcher external
POST du NDJSON (un objet JSON par ligne) sur cet endpoint. Kolektor n'impose
aucune implementation de collecteur : tout process capable de POSTer du NDJSON
est compatible.

## Format d'entree

JSON object par ligne (NDJSON), schema Management Activity API Microsoft. Champs
obligatoires : `Operation`, `Workload`. Tout event sans ces deux champs est rejete
vers `raw-logs` avec `parse_error = "microsoft365_audit_required_fields_missing"`.

## Fetcher attendu

Le fetcher appelle l'Office 365 Management Activity API avec les content types :

- `Audit.AzureActiveDirectory`
- `Audit.Exchange`
- `Audit.SharePoint`
- `Audit.General`
- `DLP.All`

Permission API requise : `ActivityFeed.Read` sur Office 365 Management APIs.

Le fetcher POSTe chaque event JSON sur `http://<kolektor-host>:${LISTEN_PORT:-8517}`.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8517`  | Port d'ecoute HTTP          |
| `TENANT_ID`         | —       | Injecte runtime             |
| `DATASOURCE_ID`     | —       | Injecte runtime             |
| `QUICKWIT_ENDPOINT` | —       | Endpoint Quickwit           |

## Liens

- [Office 365 Management Activity API reference](https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-reference)
