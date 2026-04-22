# Microsoft 365 Unified Audit — Management Activity API

## Description
Normalise les evenements Microsoft 365 Unified Audit recuperes par
`kolektor-fetcher` via l'Office 365 Management Activity API.

Le parser lit du JSON line-delimited depuis `MS365_AUDIT_LOG` et envoie les
evenements valides vers OCSF 6001 API Activity.

## Fetcher attendu

Provider `microsoft365_management`, avec les content types souhaites :

- `Audit.AzureActiveDirectory`
- `Audit.Exchange`
- `Audit.SharePoint`
- `Audit.General`
- `DLP.All`

Permission API : `ActivityFeed.Read` sur Office 365 Management APIs.

## Variables
| Variable              | Default                                             | Description       |
|-----------------------|-----------------------------------------------------|-------------------|
| `MS365_AUDIT_LOG`     | `/var/lib/kolektor/fetcher/microsoft-365-audit.jsonl` | Fichier JSONL |
| `TENANT_ID`           | -                                                   | Injecte runtime   |
| `DATASOURCE_ID`       | -                                                   | Injecte runtime   |
| `QUICKWIT_ENDPOINT`   | -                                                   | Endpoint Quickwit |

## Liens
- [Office 365 Management Activity API reference](https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-reference)
