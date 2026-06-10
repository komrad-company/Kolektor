# Traefik — access log JSON

## Description
Collects Traefik access logs (structured JSON format).
Normalized to OCSF class 4002 (HTTP Activity).

## Expected format
One JSON line per request, written by Traefik with `format=json`:
```json
{
  "ClientHost": "10.0.0.5",
  "ClientUsername": "-",
  "DownstreamStatus": 200,
  "DownstreamContentSize": 1234,
  "Duration": 12345678,
  "RequestHost": "app.example.com",
  "RequestMethod": "GET",
  "RequestPath": "/api/v1/users",
  "RequestProtocol": "HTTP/2.0",
  "RequestScheme": "https",
  "RouterName": "websecure@docker",
  "ServiceName": "api@docker",
  "StartUTC": "2024-04-14T08:23:45.123456Z",
  "request_User-Agent": "Mozilla/5.0 ..."
}
```

## Source-side configuration

### Traefik static config (YAML)
```yaml
accessLog:
  filePath: /var/log/traefik/access.log
  format: json
  fields:
    defaultMode: keep
    headers:
      defaultMode: drop
      names:
        User-Agent: keep
        Referer: keep
```

### Traefik static config (CLI)
```
--accesslog=true
--accesslog.filepath=/var/log/traefik/access.log
--accesslog.format=json
--accesslog.fields.headers.names.User-Agent=keep
--accesslog.fields.headers.names.Referer=keep
```

## Variables
| Variable              | Default                          | Description                    |
|-----------------------|----------------------------------|--------------------------------|
| `TRAEFIK_ACCESS_LOG`  | `/var/log/traefik/access.log`    | Access log file path           |
| `TENANT_ID`           | -                                | Injected at runtime            |
| `DATASOURCE_ID`       | -                                | Injected at runtime            |
| `QUICKWIT_ENDPOINT`   | -                                | Injected at runtime            |

## OCSF mapping
| Traefik field           | OCSF field                     |
|-------------------------|--------------------------------|
| `RequestMethod`         | `http_request.http_method`     |
| `RequestPath`           | `http_request.url.path`        |
| `RequestHost`           | `http_request.url.hostname`    |
| `RequestScheme`         | `http_request.url.scheme`      |
| `RequestProtocol`       | `http_request.version`         |
| `request_User-Agent`    | `http_request.user_agent`      |
| `request_Referer`       | `http_request.referrer`        |
| `DownstreamStatus`      | `http_response.code`           |
| `ClientHost`            | `src_endpoint.ip`              |
| `ClientUsername`        | `actor.user.name`              |
| `DownstreamContentSize` | `traffic.bytes_out`            |
| `RequestContentSize`    | `traffic.bytes_in`             |
| `StartUTC`              | `time`                         |
| `RouterName`            | `unmapped.router_name`         |
| `ServiceName`           | `unmapped.service_name`        |
| `Duration`              | `unmapped.duration_ns`         |

## Links
- [Traefik access log docs](https://doc.traefik.io/traefik/observability/access-logs/)
- [OCSF 4002 HTTP Activity](https://schema.ocsf.io/classes/http_activity)
