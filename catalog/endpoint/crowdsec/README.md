# CrowdSec — Security Finding

## Description
Ingests CrowdSec alerts pushed as JSON by the `http` notification plugin.
Normalises into OCSF class 2001 (Security Finding), routed to the
`ocsf-endpoint` index.

## Format attendu
```json
{
  "scenario": "crowdsecurity/ssh-bf",
  "scenario_hash": "...",
  "scenario_version": "0.1",
  "message": "Ip 1.2.3.4 performed 'crowdsecurity/ssh-bf' (6 events over 5s)",
  "events_count": 6,
  "source": {
    "scope": "Ip",
    "value": "1.2.3.4",
    "ip": "1.2.3.4",
    "cn": "US",
    "as_name": "...",
    "latitude": 0.0
  },
  "remediation": true,
  "decisions": [
    {"type": "ban", "scope": "Ip", "value": "1.2.3.4", "duration": "4h", "origin": "crowdsec", "scenario": "crowdsecurity/ssh-bf"}
  ],
  "start_at": "2024-04-14T10:23:45Z",
  "stop_at": "2024-04-14T10:23:50Z"
}
```

## Configuration cote source
CrowdSec pushes alerts through the local API plugin system. Declare an `http`
notification in `/etc/crowdsec/notifications/http.yaml` and bind it to a
profile in `/etc/crowdsec/profiles.yaml`.

```yaml
# /etc/crowdsec/notifications/http.yaml
type: http
name: kolektor_http
url: http://kolektor:8520/
method: POST
headers:
  Content-Type: application/json
```

```yaml
# /etc/crowdsec/profiles.yaml
name: default_ip_remediation
filters:
  - Alert.Remediation == true
decisions:
  - type: ban
    duration: 4h
notifications:
  - kolektor_http
```

The plugin posts each alert as a JSON object; Vector's `http_server` source
decodes the body directly.

## Variables
| Variable    | Default | Description        |
|-------------|---------|--------------------|
| LISTEN_PORT | 8520    | HTTP listen port   |

## Mapping OCSF
| CrowdSec               | OCSF                                  |
|------------------------|---------------------------------------|
| scenario               | finding_info.title                    |
| scenario_hash          | finding_info.uid                      |
| message                | finding_info.desc                     |
| source.ip              | src_endpoint.ip                       |
| source.cn              | src_endpoint.location.country         |
| source.as_name         | src_endpoint.location.isp             |
| start_at               | time (epoch ms)                       |
| scenario_version       | metadata.version                      |
| decisions[].type       | severity_id (ban/captcha -> 4 High)   |
| remediation            | severity_id (true -> 3 Medium)        |

`severity_id` falls back to 2 (Low) when no remediation is active. `activity_id`
is 1 (Generate) — a CrowdSec alert is always a generated finding.

## Liens
- [CrowdSec notification plugins](https://docs.crowdsec.net/docs/notification_plugins/intro)
- [CrowdSec HTTP plugin](https://docs.crowdsec.net/docs/notification_plugins/http)
- [Vector HTTP source](https://vector.dev/docs/reference/configuration/sources/http_server/)
