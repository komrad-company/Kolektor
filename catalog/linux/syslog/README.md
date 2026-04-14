# Linux Syslog — RFC 3164 / RFC 5424

## Description
Collecte les logs syslog Linux (rsyslog, syslog-ng, systemd-journald) via TCP/UDP.
Normalise en OCSF classe 6001 (API Activity) pour les evenements systeme generiques.

## Format attendu
- RFC 3164 : `<PRI>Mmm dd hh:mm:ss hostname app[pid]: message`
- RFC 5424 : `<PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID STRUCTURED-DATA MSG`

Vector parse automatiquement les deux formats via la source `syslog`.

## Configuration cote source

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
| LISTEN_PORT | 514     | Port TCP d'ecoute  |

## Mapping OCSF
| Champ syslog | Champ OCSF                |
|-------------|---------------------------|
| appname     | actor.process.name        |
| procid      | actor.process.pid         |
| hostname    | src_endpoint.hostname     |
| severity    | severity_id (remappe 0-7 → 1-5) |
| facility    | metadata.log_provider     |

## Liens
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
- [RFC 3164](https://datatracker.ietf.org/doc/html/rfc3164)
- [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424)
