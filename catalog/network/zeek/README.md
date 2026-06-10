# Zeek — JSON logs

## Description
Collects Zeek network logs in JSON mode from a log directory and normalises them
to OCSF. The log type is read from the `_path` field that Zeek writes on every
JSON line, and the parser routes dynamically:

- `conn` → OCSF 4001 Network Activity (`ocsf-network`)
- `ssl` → OCSF 4001 Network Activity (`ocsf-network`) — no dedicated TLS index
- `http` → OCSF 4002 HTTP Activity (`ocsf-http`)
- `dns` → OCSF 4003 DNS Activity (`ocsf-dns`)

Invalid JSON lines and unsupported `_path` values are quarantined in `raw-logs`
with `parse_status = "failed"`.

## Expected format
One JSON object per line. Zeek must run in JSON output mode so each line carries
`_path` and a float epoch `ts`:

```json
{"_path":"conn","ts":1713091425.123,"id.orig_h":"10.0.0.5","id.orig_p":54321,"id.resp_h":"1.2.3.4","id.resp_p":443,"proto":"tcp","orig_bytes":1024,"resp_bytes":2048,"conn_state":"SF","duration":1.5}
```

Zeek encodes the connection 4-tuple as dotted top-level keys (`id.orig_h`,
`id.resp_p`); these are literal keys, not nested objects.

## Source configuration

### Zeek
Enable JSON logging so each line is a self-describing JSON object. In
`local.zeek`:

```
@load policy/tuning/json-logs.zeek
```

This makes Zeek emit `conn.log`, `dns.log`, `http.log`, `ssl.log`, etc. as JSON
with a `_path` field. Mount or forward the Zeek log directory into the Kolektor
pod and point `ZEEK_LOG_DIR` at it. The `file` source ingests
`${ZEEK_LOG_DIR}/*.log`.

## Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `ZEEK_LOG_DIR` | `/var/log/zeek` | Directory of Zeek `*.log` JSON files |
| `TENANT_ID` | - | Injected at runtime |
| `DATASOURCE_ID` | - | Injected at runtime |
| `QUICKWIT_ENDPOINT` | - | Injected at runtime |

No listener port: this source is file-fed (see `_schema/ports.md`).

## OCSF mapping
| Zeek field | OCSF field |
|------------|------------|
| `_path` | route / `class_uid` selection, `unmapped.zeek_path` |
| `ts` | `time` (epoch seconds → milliseconds) |
| `id.orig_h`, `id.orig_p` | `src_endpoint.ip`, `src_endpoint.port` |
| `id.resp_h`, `id.resp_p` | `dst_endpoint.ip`, `dst_endpoint.port` |
| `proto` | `connection_info.protocol_name` |
| `orig_bytes`, `resp_bytes` | `traffic.bytes_out`, `traffic.bytes_in` |
| `duration`, `conn_state` | `duration`, `status_detail` |
| `query`, `qtype_name`, `rcode_name` | `query.hostname`, `query.type`, `query.rcode` |
| `answers` | `answers` |
| `method`, `host`, `uri`, `user_agent` | `http_request.*` |
| `status_code` | `http_response.code` |
| `server_name`, `version`, `validation_status` | `tls.sni`, `tls.version`, `tls.validation_status` |

### activity_id
| Log type | Rule | activity_id |
|----------|------|-------------|
| `conn` | `conn_state == "SF"` | 1 Open |
| `conn` | `conn_state == "S0"` | 4 Fail |
| `conn` | `conn_state == "REJ"` | 5 Refuse |
| `conn` | otherwise | 6 Traffic |
| `ssl` | always | 6 Traffic |
| `dns` | `answers` present | 2 Response |
| `dns` | otherwise | 1 Query |
| `http` | from request method (GET→3, POST→6, PUT→7, DELETE→2, HEAD→4, OPTIONS→5, PATCH→9, CONNECT→1, TRACE→8, else 99) |

## Known limits
- `severity_id` is fixed to `1` (Informational): Zeek conn/dns/http/ssl are
  telemetry, not findings. Severity weighting belongs to Korelator rules.
- `ssl` events land in `ocsf-network` — the catalogue has no dedicated TLS index.
- Zeek `_path` values outside `conn` / `dns` / `http` / `ssl` (e.g. `x509`,
  `files`, `notice`) are quarantined in `raw-logs`; add a route to extend.
- If `ts` is missing or unparsable the parser falls back to `now()`.

## Links
- [Zeek logging framework](https://docs.zeek.org/en/master/frameworks/logging.html)
- [Zeek JSON logs policy](https://docs.zeek.org/en/master/scripts/policy/tuning/json-logs.zeek.html)
- [OCSF Network Activity](https://schema.ocsf.io/classes/network_activity)
- [OCSF HTTP Activity](https://schema.ocsf.io/classes/http_activity)
- [OCSF DNS Activity](https://schema.ocsf.io/classes/dns_activity)
