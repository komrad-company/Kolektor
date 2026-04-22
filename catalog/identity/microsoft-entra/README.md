# Microsoft Entra ID — Graph audit logs

## Description
Normalise les logs Microsoft Graph Entra ID produits par `kolektor-fetcher` :

- `/auditLogs/signIns` vers OCSF 3002 Authentication ;
- `/auditLogs/directoryAudits` vers OCSF 6001 API Activity.

Le parser lit du JSON line-delimited depuis `MS_ENTRA_LOG`.

## Fetcher attendu

Provider `microsoft_graph`, avec `kind = "signins"` ou
`kind = "directory_audits"`. Les deux fetchers peuvent ecrire dans le meme
fichier JSONL surveille par ce parser.

Permissions Microsoft Graph : `AuditLog.Read.All` en application permission.
Les sign-in logs peuvent aussi necessiter une licence Entra ID P1/P2 selon le
tenant.

## Variables
| Variable              | Default                                      | Description            |
|-----------------------|----------------------------------------------|------------------------|
| `MS_ENTRA_LOG`        | `/var/lib/kolektor/fetcher/microsoft-entra.jsonl` | Fichier JSONL    |
| `TENANT_ID`           | -                                            | Injecte runtime        |
| `DATASOURCE_ID`       | -                                            | Injecte runtime        |
| `QUICKWIT_ENDPOINT`   | -                                            | Endpoint Quickwit      |

## Liens
- [Microsoft Entra activity logs with Microsoft Graph](https://learn.microsoft.com/en-us/entra/identity/monitoring-health/howto-analyze-activity-logs-with-microsoft-graph)
- [Graph signIns API](https://learn.microsoft.com/en-us/graph/api/signin-list)
