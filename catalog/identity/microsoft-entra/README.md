# Microsoft Entra ID — Graph API audit logs

## Description

Normalises Microsoft Graph Entra ID logs:

- `/auditLogs/signIns` to OCSF 3002 Authentication;
- `/auditLogs/directoryAudits` to OCSF 6003 API Activity.

The parser exposes an HTTP endpoint on `${LISTEN_PORT:-8518}`. The external fetcher
POSTs NDJSON (one JSON object per line) to this endpoint. Both event types can
transit on the same port: the OCSF class is determined at runtime from the
presence of `createdDateTime` (sign-in) or `activityDateTime` (directory audit).

## Input format

One JSON object per line (NDJSON), Microsoft Graph API schema. Discriminating fields:

- Sign-in: `createdDateTime` required.
- Directory audit: `activityDateTime` required.

Any event missing either is rejected to `raw-logs` with
`parse_error = "microsoft_entra_event_shape_invalid"`.

## Expected fetcher

The fetcher calls Microsoft Graph with the permissions:

- `AuditLog.Read.All` (application permission)
- An Entra ID P1/P2 license is required for sign-in logs depending on the tenant.

The fetcher POSTs each JSON event to `http://<kolektor-host>:${LISTEN_PORT:-8518}`.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8518`  | HTTP listen port            |
| `TENANT_ID`         | —       | Injected at runtime         |
| `DATASOURCE_ID`     | —       | Injected at runtime         |
| `QUICKWIT_ENDPOINT` | —       | Quickwit endpoint           |

## Links

- [Microsoft Entra activity logs with Microsoft Graph](https://learn.microsoft.com/en-us/entra/identity/monitoring-health/howto-analyze-activity-logs-with-microsoft-graph)
- [Graph signIns API](https://learn.microsoft.com/en-us/graph/api/signin-list)
- [Graph directoryAudits API](https://learn.microsoft.com/en-us/graph/api/directoryaudit-list)
