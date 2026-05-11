# Microsoft Entra ID — Graph API audit logs

## Description

Normalise les logs Microsoft Graph Entra ID :

- `/auditLogs/signIns` vers OCSF 3002 Authentication ;
- `/auditLogs/directoryAudits` vers OCSF 6001 API Activity.

Le parser expose un endpoint HTTP sur `${LISTEN_PORT:-8518}`. Le fetcher externe
POST du NDJSON (un objet JSON par ligne) sur cet endpoint. Les deux types d'events
peuvent transiter sur le meme port : la classe OCSF est determinee au runtime selon
la presence de `createdDateTime` (sign-in) ou `activityDateTime` (directory audit).

## Format d'entree

JSON object par ligne (NDJSON), schema Microsoft Graph API. Champs discriminants :

- Sign-in : `createdDateTime` obligatoire.
- Directory audit : `activityDateTime` obligatoire.

Tout event sans l'un ou l'autre est rejete vers `raw-logs` avec
`parse_error = "microsoft_entra_event_shape_invalid"`.

## Fetcher attendu

Le fetcher appelle Microsoft Graph avec les permissions :

- `AuditLog.Read.All` (application permission)
- Licence Entra ID P1/P2 necessaire pour les sign-in logs selon le tenant.

Le fetcher POSTe chaque event JSON sur `http://<kolektor-host>:${LISTEN_PORT:-8518}`.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8518`  | Port d'ecoute HTTP          |
| `TENANT_ID`         | —       | Injecte runtime             |
| `DATASOURCE_ID`     | —       | Injecte runtime             |
| `QUICKWIT_ENDPOINT` | —       | Endpoint Quickwit           |

## Liens

- [Microsoft Entra activity logs with Microsoft Graph](https://learn.microsoft.com/en-us/entra/identity/monitoring-health/howto-analyze-activity-logs-with-microsoft-graph)
- [Graph signIns API](https://learn.microsoft.com/en-us/graph/api/signin-list)
- [Graph directoryAudits API](https://learn.microsoft.com/en-us/graph/api/directoryaudit-list)
