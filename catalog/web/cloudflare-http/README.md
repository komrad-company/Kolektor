# Cloudflare HTTP Requests — Logpull / export JSONL

## Description
Collecte les logs Cloudflare `http_requests` exportes en JSON line-delimited
par Logpull, par un export object storage, ou par un futur fetcher Kolektor.
Normalise en OCSF classe 4002 (HTTP Activity).

Les evenements parses conservent le JSON source dans `raw`. Les lignes JSON
invalides ou sans champs requis (`ClientRequestMethod`, `ClientRequestHost`)
partent dans `raw-logs`.

## Format attendu
Une ligne JSON par requete, avec les champs Cloudflare HTTP Requests :

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

Le parser accepte les timestamps RFC3339, Unix secondes, Unix millisecondes et
UnixNano pour `EdgeStartTimestamp`, `EdgeEndTimestamp` ou `Datetime`.

## Source Vector

Fichier JSON line-delimited via `CLOUDFLARE_HTTP_LOG`. Pour eviter d'exposer
un port entrant, le modele recommande est qu'un fetcher pull les logs Cloudflare
et ecrive les lignes JSON dans ce fichier.

## Variables
| Variable                | Default                                  | Description                  |
|-------------------------|------------------------------------------|------------------------------|
| `CLOUDFLARE_HTTP_LOG`   | `/var/log/cloudflare/http_requests.json` | Fichier JSONL optionnel      |
| `TENANT_ID`             | -                                        | Injecte runtime              |
| `DATASOURCE_ID`         | -                                        | Injecte runtime              |
| `QUICKWIT_ENDPOINT`     | -                                        | Endpoint Quickwit            |

## Mapping OCSF
| Champ Cloudflare          | Champ OCSF / Kolektor              |
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

## Severite
| Condition                              | `severity_id` |
|----------------------------------------|---------------|
| Action de mitigation WAF/security      | 3             |
| `EdgeResponseStatus >= 500`            | 4             |
| `EdgeResponseStatus >= 400`            | 2             |
| Autre                                  | 1             |

## Liens
- [Cloudflare HTTP requests fields](https://developers.cloudflare.com/logs/logpush/logpush-job/datasets/zone/http_requests/)
- [Cloudflare Logpush datasets](https://developers.cloudflare.com/logs/reference/log-fields/)
