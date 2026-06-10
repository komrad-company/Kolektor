# Unbound DNS (OPNsense) — query log

## Description
Collects DNS queries resolved by Unbound on OPNsense, forwarded over syslog.
Normalized to OCSF class 4003 (DNS Activity), activity `Query`.

## Expected format
Native unbound format (verbosity >= 1, `log-queries: yes`):
```
[1713091425] unbound[12345:0] info: 10.0.0.5 example.com. A IN
```

The parser also accepts the variant without the unbound prefix (some syslog forwarders truncate it):
```
info: 10.0.0.5 example.com. A IN
```

One line = one observed DNS query. The `resolving`, `reply from`, `response for` lines are ignored (dropped via filter_valid).

## Source-side configuration

### OPNsense — Services > Unbound DNS > General
- **Log level verbosity**: `Level 1 (Info)` minimum
- **Log queries**: checked
- Optional: Log replies, Log tag queryreply

### OPNsense — System > Settings > Logging / targets
Add a Remote Syslog target:
- **Transport**: TCP
- **Hostname/IP**: `<vector-host>`
- **Port**: `5144` (this parser's default)
- **Facility**: `local7` (or the one used by unbound)
- **Selection**: check `DNS (Unbound)`

## Variables
| Variable          | Default | Description                                 |
|-------------------|---------|---------------------------------------------|
| `LISTEN_PORT`     | `5144`  | TCP syslog port                             |
| `TENANT_ID`       | -       | Injected at runtime                         |
| `DATASOURCE_ID`   | -       | Injected at runtime                         |
| `QUICKWIT_ENDPOINT` | -     | Injected at runtime                         |

## OCSF mapping
| Unbound field   | OCSF field             |
|-----------------|------------------------|
| client IP       | `src_endpoint.ip`      |
| QNAME           | `query.hostname`       |
| QTYPE           | `query.type`           |
| QCLASS          | `query.class`          |
| syslog ts       | `time`                 |
| -               | `activity_id = 1` (Query) |

## Known limits
- The parser captures only queries (not responses nor blocklist blocks) — a blocked query first appears as a Query then as an NXDOMAIN response (not captured here). Sufficient for threat hunting on resolved domains.
- The Unix timestamp in `[1713091425]` is ignored in favor of the syslog timestamp (more reliable after relay).

## Links
- [Unbound logging docs](https://unbound.docs.nlnetlabs.nl/en/latest/manpages/unbound.conf.html#logging)
- [OCSF 4003 DNS Activity](https://schema.ocsf.io/classes/dns_activity)
