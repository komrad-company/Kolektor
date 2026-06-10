# Microsoft 365 Unified Audit — Management Activity API

## Description

Normalises Microsoft 365 Unified Audit events to OCSF 6003 API Activity.

The parser exposes an HTTP endpoint on `${LISTEN_PORT:-8517}`. The external fetcher
POSTs NDJSON (one JSON object per line) to this endpoint. Kolektor mandates no
specific collector implementation: any process capable of POSTing NDJSON is
compatible.

## Input format

One JSON object per line (NDJSON), Microsoft Management Activity API schema.
Required fields: `Operation`, `Workload`. Any event missing these two fields is
rejected to `raw-logs` with `parse_error = "microsoft365_audit_required_fields_missing"`.

## Expected fetcher

The fetcher calls the Office 365 Management Activity API with the content types:

- `Audit.AzureActiveDirectory`
- `Audit.Exchange`
- `Audit.SharePoint`
- `Audit.General`
- `DLP.All`

Required API permission: `ActivityFeed.Read` on Office 365 Management APIs.

The fetcher POSTs each JSON event to `http://<kolektor-host>:${LISTEN_PORT:-8517}`.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8517`  | HTTP listen port            |
| `TENANT_ID`         | —       | Injected at runtime         |
| `DATASOURCE_ID`     | —       | Injected at runtime         |
| `QUICKWIT_ENDPOINT` | —       | Quickwit endpoint           |

## Links

- [Office 365 Management Activity API reference](https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-reference)
