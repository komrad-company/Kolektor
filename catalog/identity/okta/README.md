# Okta — System Log API events

## Description

Normalises Okta System Log API events into OCSF. The collective routes by
`eventType` prefix:

- `user.session.*` / `user.authentication.*` → OCSF 3002 Authentication
  (`ocsf-identity`);
- `user.lifecycle.*` / `group.*` / `user.account.*` → OCSF 3001 Account Change
  (`ocsf-identity`);
- everything else with a valid `eventType` → OCSF 6003 API Activity
  (`ocsf-audit`).

The parser exposes an HTTP endpoint on `${LISTEN_PORT:-8522}`. A System Log
fetcher pulls events from `/api/v1/logs` and POSTs each LogEvent JSON object to
this endpoint. The OCSF class is decided at runtime from `eventType` — all three
classes transit the same port.

## Input format

One Okta System Log `LogEvent` JSON object per request, real System Log schema.
The discriminating field is `eventType`; an event without it is rejected to
`raw-logs` with `parse_error = "okta_event_type_missing"`.

```json
{
  "uuid": "e1a2",
  "published": "2024-04-14T10:23:45.123Z",
  "eventType": "user.session.start",
  "severity": "INFO",
  "displayMessage": "User login to Okta",
  "actor": {"id": "00u1", "type": "User", "alternateId": "jdoe@corp.com", "displayName": "J Doe"},
  "client": {"ipAddress": "1.2.3.4", "userAgent": {"rawUserAgent": "Mozilla/5.0"}, "geographicalContext": {"country": "US", "city": "NYC"}},
  "outcome": {"result": "SUCCESS", "reason": null},
  "target": [{"id": "0oa1", "type": "AppInstance", "displayName": "Salesforce"}]
}
```

## Field mapping

| OCSF field          | Okta source                                  |
|---------------------|----------------------------------------------|
| `time`              | `published` (ISO 8601 → epoch ms)            |
| `severity_id`       | `severity` (DEBUG/INFO → 1, WARN → 3, ERROR → 4, else 1) |
| `status_id`         | `outcome.result` (SUCCESS → 1, FAILURE → 2)  |
| `actor.user.name`   | `actor.alternateId`                          |
| `actor.user.full_name` | `actor.displayName`                       |
| `actor.user.uid`    | `actor.id`                                   |
| `src_endpoint.ip`   | `client.ipAddress`                           |
| `metadata.uid`      | `uuid`                                        |

`activity_id`: Authentication (3002) maps `session.start`/`authentication.*` →
1 (Logon), `session.end` → 2 (Logoff), else 99. Account Change (3001) defaults
to 99 (Other) — Okta lifecycle/group event types do not map deterministically
to the OCSF verb enum. API Activity (6003) derives 1/2/3/4 from the verb in the
`eventType` string, else 99.

## Fetcher

The fetcher reads the Okta System Log API with an API token scoped to read
events:

- API token (SSWS) or OAuth 2.0 service app with `okta.logs.read`.

It paginates `/api/v1/logs` (cursor via the `Link: rel="next"` header) and POSTs
each event JSON to `http://<kolektor-host>:${LISTEN_PORT:-8522}`. Pagination,
backoff and cursor state live in the fetcher, never in the VRL.

## Variables

| Variable            | Default | Description          |
|---------------------|---------|----------------------|
| `LISTEN_PORT`       | `8522`  | HTTP listen port     |
| `TENANT_ID`         | —       | Injected at runtime  |
| `DATASOURCE_ID`     | —       | Injected at runtime  |
| `QUICKWIT_ENDPOINT` | —       | Quickwit endpoint    |

## Links

- [Okta System Log API](https://developer.okta.com/docs/reference/api/system-log/)
- [LogEvent object reference](https://developer.okta.com/docs/reference/api/system-log/#logevent-object)
- [System Log event types](https://developer.okta.com/docs/reference/api/event-types/)
