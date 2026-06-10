# Cloudflare HTTP Requests — Logpull / export JSONL

## Description
Collects Cloudflare `http_requests` logs exported as line-delimited JSON
by Logpull, by an object-storage export, or by a future Kolektor fetcher.
Normalized to OCSF class 4002 (HTTP Activity).

Parsed events keep the source JSON in `raw`. Invalid JSON lines or lines
missing required fields (`ClientRequestMethod`, `ClientRequestHost`)
go to `raw-logs`.

## Expected format
One JSON line per request, with the Cloudflare HTTP Requests fields:

```json
{
  "RayID": "806c30a3cec56817",
  "EdgeStartTimestamp": "2025-07-17T07:54:19Z",
  "ClientIP": "203.0.113.10",
  "ClientRequestHost": "app.example.com",
  "ClientRequestMethod": "GET",
  "ClientRequestURI": "/api/v1/users?active=true",
  "ClientRequestPath": "/api/v1/users",
  "ClientRequestProtocol": "HTTP/2",
  "ClientRequestScheme": "https",
  "EdgeResponseStatus": 200,
  "SecurityAction": "unknown"
}
```

The parser accepts RFC3339, Unix seconds, Unix milliseconds and
UnixNano timestamps for `EdgeStartTimestamp`, `EdgeEndTimestamp` or `Datetime`.

## Vector source

Line-delimited JSON file via `CLOUDFLARE_HTTP_LOG`. To avoid exposing
an inbound port, the recommended model is for a fetcher to pull the Cloudflare
logs and write the JSON lines into this file.

## Variables
| Variable                | Default                                  | Description                  |
|-------------------------|------------------------------------------|------------------------------|
| `CLOUDFLARE_HTTP_LOG`   | `/var/lib/kolektor/fetcher/cloudflare-http.jsonl` | Optional JSONL file |
| `TENANT_ID`             | -                                        | Injected at runtime          |
| `DATASOURCE_ID`         | -                                        | Injected at runtime          |
| `QUICKWIT_ENDPOINT`     | -                                        | Quickwit endpoint            |

## OCSF mapping
| Cloudflare field          | OCSF / Kolektor field              |
|---------------------------|------------------------------------|
| `ClientRequestMethod`     | `http_request.http_method`         |
| `ClientRequestHost`       | `http_request.url.hostname`        |
| `ClientRequestPath`       | `http_request.url.path`            |
| `ClientRequestURI`        | fallback `http_request.url.path`   |
| `ClientRequestScheme`     | `http_request.url.scheme`          |
| `ClientRequestProtocol`   | `http_request.version`             |
| `ClientRequestUserAgent`  | `http_request.user_agent`          |
| `ClientRequestReferer`    | `http_request.referrer`            |
| `EdgeResponseStatus`      | `http_response.code`               |
| `ClientIP`                | `src_endpoint.ip`                  |
| `ClientASN`               | `src_endpoint.asn`                 |
| `ClientCountry`           | `src_endpoint.location.country`    |
| `ClientRequestBytes`      | `traffic.bytes_in`                 |
| `EdgeResponseBytes`       | `traffic.bytes_out`                |
| `RayID`                   | `metadata.uid`, `unmapped.ray_id`  |
| `SecurityAction(s)`       | `unmapped.security_*`              |
| `BotScore`, WAF scores    | `unmapped.bot_*`, `unmapped.waf_*` |

## Severity
| Condition                              | `severity_id` |
|----------------------------------------|---------------|
| WAF/security mitigation action         | 3             |
| `EdgeResponseStatus >= 500`            | 4             |
| `EdgeResponseStatus >= 400`            | 2             |
| Other                                  | 1             |

## Links
- [Cloudflare HTTP requests fields](https://developers.cloudflare.com/logs/logpush/logpush-job/datasets/zone/http_requests/)
- [Cloudflare Logpush datasets](https://developers.cloudflare.com/logs/reference/log-fields/)
