# Google Workspace — Admin SDK Reports API activities

## Description

Normalises Google Workspace Admin SDK Reports API activities. The OCSF class is
determined at runtime from `id.applicationName`:

- `login` to OCSF 3002 Authentication (index `ocsf-identity`);
- `admin` / `user_accounts` / `groups` to OCSF 3001 Account Change (index `ocsf-identity`);
- any other application (`drive`, `token`, `calendar`...) to OCSF 6003 API Activity (index `ocsf-audit`).

The parser exposes an HTTP endpoint on `${LISTEN_PORT:-8523}`. An external fetcher
queries the Reports API and POSTs each JSON `activity` object to this endpoint.

## Input format

`admin#reports#activity` JSON object from the Reports API:

```json
{
  "kind": "admin#reports#activity",
  "id": {
    "time": "2024-04-14T10:23:45.123Z",
    "uniqueQualifier": "-12345",
    "applicationName": "login",
    "customerId": "C01"
  },
  "actor": { "email": "jdoe@corp.com", "profileId": "1122" },
  "ipAddress": "1.2.3.4",
  "events": [
    {
      "type": "login",
      "name": "login_success",
      "parameters": [
        { "name": "login_type", "value": "google_password" },
        { "name": "is_suspicious", "boolValue": false }
      ]
    }
  ]
}
```

The `events[]` array may contain multiple entries. The parser maps the first one
for `name`/`type`; the rest are kept in `unmapped.extra_events`.

### OCSF mapping

- `time` derived from `id.time` (RFC 3339) as epoch milliseconds.
- `actor.user.email_addr` / `actor.user.name` from `actor.email`, `actor.user.uid` from `actor.profileId`.
- `src_endpoint.ip` from `ipAddress`.
- Authentication (`login`): `activity_id` 1 (Logon) for `login_success`/`login`, 2 (Logoff) for `logout`, 99 otherwise. `status_id` 2 (Failure) for `login_failure`, 1 (Success) otherwise.
- Account Change (`admin`/`user_accounts`/`groups`): `activity_id` 99 (verbs too varied for a reliable mapping).
- API Activity (others): `activity_id` derived from the verb (`create`/`add` 1, `view`/`download`/`list` 2, `edit`/`update`/`rename` 3, `delete`/`remove` 4, otherwise 99).
- `severity_id` 1 (Informational) by default, 3 (Medium) on `login_failure`.

Any activity missing `id.applicationName` or `events[]` is rejected to
`raw-logs` with `parse_error = "google_workspace_activity_shape_invalid"`.

## Expected fetcher

The fetcher calls the Admin SDK Reports API with:

- the `https://www.googleapis.com/auth/admin.reports.audit.readonly` scope;
- a service account with domain-wide delegation, or an OAuth admin.

It paginates via `nextPageToken`, keeps a cursor on `startTime`, and POSTs each
event from `items[]` to `http://<kolektor-host>:${LISTEN_PORT:-8523}`. Pagination,
OAuth and the cursor stay in the fetcher — never in the VRL.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8523`  | HTTP listen port            |
| `TENANT_ID`         | —       | Injected at runtime         |
| `DATASOURCE_ID`     | —       | Injected at runtime         |
| `QUICKWIT_ENDPOINT` | —       | Quickwit endpoint           |

## Links

- [Admin SDK Reports API — Activities](https://developers.google.com/admin-sdk/reports/reference/rest/v1/activities)
- [Activities.list](https://developers.google.com/admin-sdk/reports/reference/rest/v1/activities/list)
- [Application names (login, admin, drive, token...)](https://developers.google.com/admin-sdk/reports/v1/appendix/activity)
