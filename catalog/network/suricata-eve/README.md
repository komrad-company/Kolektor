# Suricata EVE — JSON events

## Description
Collects Suricata EVE JSON events from an `eve.json` file.
The parser routes dynamically:
- `alert` → OCSF 2001 Security Finding (`ocsf-endpoint`)
- `flow`, `netflow`, `tls` → OCSF 4001 Network Activity (`ocsf-network`)
- `http` → OCSF 4002 HTTP Activity (`ocsf-http`)
- `dns` → OCSF 4003 DNS Activity (`ocsf-dns`)

Invalid JSON lines or unsupported `event_type` values go to `raw-logs`
with `parse_status = "failed"`.

## Expected format
One JSON line per event:

```json
{"timestamp":"2024-04-14T10:23:45.123456+0000","event_type":"alert","src_ip":"10.0.0.5","src_port":54321,"dest_ip":"8.8.8.8","dest_port":53,"proto":"UDP","alert":{"signature_id":2024218,"signature":"ET DNS Query for suspicious domain","category":"Potentially Bad Traffic","severity":2}}
```

## Source-side configuration

### Suricata
In `suricata.yaml`, enable `eve-log` with at least the useful types:

```yaml
outputs:
  - eve-log:
      enabled: yes
      filetype: regular
      filename: eve.json
      types:
        - alert
        - dns
        - http
        - tls
        - flow
```

Mount or forward the `eve.json` file into the Kolektor pod.

## Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `SURICATA_EVE_LOG` | `/var/log/suricata/eve.json` | EVE JSON file path |
| `TENANT_ID` | - | Injected at runtime |
| `DATASOURCE_ID` | - | Injected at runtime |
| `QUICKWIT_ENDPOINT` | - | Injected at runtime |

## OCSF mapping
| Suricata field | OCSF field |
|----------------|------------|
| `src_ip`, `src_port` | `src_endpoint.ip`, `src_endpoint.port` |
| `dest_ip`, `dest_port` | `dst_endpoint.ip`, `dst_endpoint.port` |
| `proto`, `app_proto` | `connection_info` |
| `alert.signature` | `finding_info.title` |
| `alert.signature_id` | `finding_info.uid` |
| `dns.rrname`, `dns.rrtype`, `dns.rcode` | `query.*` |
| `dns.answers` | `answers` |
| `http.*` | `http_request`, `http_response` |
| `flow.*` | `traffic` |
| `tls.sni`, `tls.version` | `tls` |

## Known limits
- The OCSF categories are intentionally broad: the parser favors class-indexable routing.
- Unsupported EVE event types (`stats`, `fileinfo`, etc.) are quarantined in `raw-logs`.
- The Suricata timestamp takes priority; if its format is unexpected, the parser falls back to `now()`.

## Links
- [Suricata EVE JSON output](https://docs.suricata.io/en/latest/output/eve/eve-json-output.html)
- [OCSF Network Activity](https://schema.ocsf.io/classes/network_activity)
- [OCSF DNS Activity](https://schema.ocsf.io/classes/dns_activity)
- [OCSF HTTP Activity](https://schema.ocsf.io/classes/http_activity)
