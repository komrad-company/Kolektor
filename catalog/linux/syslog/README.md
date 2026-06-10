# Linux Syslog — RFC 3164 / RFC 5424

## Description
Collecte les logs syslog Linux (rsyslog, syslog-ng, systemd-journald) via TCP/UDP.
Une ligne syslog generique ne porte aucune activite OCSF intrinseque : elle est
emise en OCSF Base Event (class_uid 0 / category_uid 0, decret D2) avec
`parse_status = "parsed"` et routee vers l'index `raw-logs`. Parser de repli :
les parsers specifiques (auth-log, parsers de service) sont preferes quand la
source est connue.

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

## Mapping OCSF (Base Event)
| Champ syslog                | Champ OCSF                |
|-----------------------------|---------------------------|
| severity                    | severity_id (remappe 0-7 → 1-5) |
| facility                    | metadata.log_provider     |
| header + appname/procid/message | raw                  |

La ligne syslog complete (header + message) est conservee dans `raw`. Aucun
`activity_id` n'est emis : un Base Event (class 0) n'en porte pas.

## Liens
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
- [RFC 3164](https://datatracker.ietf.org/doc/html/rfc3164)
- [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424)
