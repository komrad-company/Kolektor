# Linux Syslog — RFC 3164 / RFC 5424

## Description
Collects Linux syslog logs (rsyslog, syslog-ng, systemd-journald) over TCP/UDP.
A generic syslog line carries no intrinsic OCSF activity: it is emitted as an
OCSF Base Event (class_uid 0 / category_uid 0, decree D2) with
`parse_status = "parsed"` and routed to the `raw-logs` index. Fallback parser:
the specific parsers (auth-log, service parsers) are preferred when the source
is known.

## Expected format
- RFC 3164: `<PRI>Mmm dd hh:mm:ss hostname app[pid]: message`
- RFC 5424: `<PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID STRUCTURED-DATA MSG`

Vector automatically parses both formats via the `syslog` source.

## Source-side configuration

### rsyslog
```
*.* @@<vector-host>:<port>
```

### syslog-ng
```
destination d_vector { tcp("<vector-host>" port(<port>)); };
log { source(s_sys); destination(d_vector); };
```

## Variables
| Variable     | Default | Description        |
|-------------|---------|---------------------|
| LISTEN_PORT | 514     | TCP listen port    |

## OCSF mapping (Base Event)
| syslog field                | OCSF field                |
|-----------------------------|---------------------------|
| severity                    | severity_id (remapped 0-7 → 1-5) |
| facility                    | metadata.log_provider     |
| header + appname/procid/message | raw                  |

The complete syslog line (header + message) is preserved in `raw`. No
`activity_id` is emitted: a Base Event (class 0) carries none.

## Links
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
- [RFC 3164](https://datatracker.ietf.org/doc/html/rfc3164)
- [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424)
