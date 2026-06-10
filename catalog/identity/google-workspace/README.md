# Google Workspace — Admin SDK Reports API activities

## Description

Normalise les activites de l'Admin SDK Reports API de Google Workspace. La classe
OCSF est determinee au runtime selon `id.applicationName` :

- `login` vers OCSF 3002 Authentication (index `ocsf-identity`) ;
- `admin` / `user_accounts` / `groups` vers OCSF 3001 Account Change (index `ocsf-identity`) ;
- toute autre application (`drive`, `token`, `calendar`...) vers OCSF 6003 API Activity (index `ocsf-audit`).

Le parser expose un endpoint HTTP sur `${LISTEN_PORT:-8523}`. Un fetcher externe
interroge la Reports API et POST chaque objet `activity` JSON sur cet endpoint.

## Format d'entree

Objet JSON `admin#reports#activity` de la Reports API :

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

Le tableau `events[]` peut contenir plusieurs entrees. Le parser mappe la premiere
pour `name`/`type` ; les suivantes sont conservees dans `unmapped.extra_events`.

### Mapping OCSF

- `time` derive de `id.time` (RFC 3339) en epoch millisecondes.
- `actor.user.email_addr` / `actor.user.name` depuis `actor.email`, `actor.user.uid` depuis `actor.profileId`.
- `src_endpoint.ip` depuis `ipAddress`.
- Authentication (`login`) : `activity_id` 1 (Logon) pour `login_success`/`login`, 2 (Logoff) pour `logout`, 99 sinon. `status_id` 2 (Failure) pour `login_failure`, 1 (Success) sinon.
- Account Change (`admin`/`user_accounts`/`groups`) : `activity_id` 99 (verbes trop varies pour un mapping fiable).
- API Activity (autres) : `activity_id` deduit du verbe (`create`/`add` 1, `view`/`download`/`list` 2, `edit`/`update`/`rename` 3, `delete`/`remove` 4, sinon 99).
- `severity_id` 1 (Informational) par defaut, 3 (Medium) sur `login_failure`.

Toute activite sans `id.applicationName` ou sans `events[]` est rejetee vers
`raw-logs` avec `parse_error = "google_workspace_activity_shape_invalid"`.

## Fetcher attendu

Le fetcher appelle l'Admin SDK Reports API avec :

- le scope `https://www.googleapis.com/auth/admin.reports.audit.readonly` ;
- un compte de service avec delegation domain-wide, ou un admin OAuth.

Il pagine via `nextPageToken`, garde un curseur sur `startTime`, et POST chaque
event de `items[]` sur `http://<kolektor-host>:${LISTEN_PORT:-8523}`. La pagination,
l'OAuth et le curseur restent dans le fetcher — jamais dans le VRL.

## Variables

| Variable            | Default | Description                 |
|---------------------|---------|-----------------------------|
| `LISTEN_PORT`       | `8523`  | Port d'ecoute HTTP          |
| `TENANT_ID`         | —       | Injecte runtime             |
| `DATASOURCE_ID`     | —       | Injecte runtime             |
| `QUICKWIT_ENDPOINT` | —       | Endpoint Quickwit           |

## Liens

- [Admin SDK Reports API — Activities](https://developers.google.com/admin-sdk/reports/reference/rest/v1/activities)
- [Activities.list](https://developers.google.com/admin-sdk/reports/reference/rest/v1/activities/list)
- [Application names (login, admin, drive, token...)](https://developers.google.com/admin-sdk/reports/v1/appendix/activity)
