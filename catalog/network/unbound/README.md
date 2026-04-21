# Unbound DNS (OPNsense) — query log

## Description
Collecte les requetes DNS resolvees par Unbound sur OPNsense, forwardees via syslog.
Normalise en OCSF classe 4003 (DNS Activity), activity `Query`.

## Format attendu
Format natif unbound (verbosity >= 1, `log-queries: yes`) :
```
[1713091425] unbound[12345:0] info: 10.0.0.5 example.com. A IN
```

Le parseur accepte aussi la variante sans prefix unbound (certains forwarders syslog tronquent) :
```
info: 10.0.0.5 example.com. A IN
```

Une ligne = une requete DNS observee. Les lignes `resolving`, `reply from`, `response for` sont ignorees (drop via filter_valid).

## Configuration cote source

### OPNsense — Services > Unbound DNS > General
- **Log level verbosity** : `Level 1 (Info)` minimum
- **Log queries** : coche
- Optionnel : Log replies, Log tag queryreply

### OPNsense — System > Settings > Logging / targets
Ajouter une cible Remote Syslog :
- **Transport** : TCP
- **Hostname/IP** : `<vector-host>`
- **Port** : `5144` (par defaut de ce parser)
- **Facility** : `local7` (ou celui utilise par unbound)
- **Selection** : cocher `DNS (Unbound)`

## Variables
| Variable          | Default | Description                                 |
|-------------------|---------|---------------------------------------------|
| `LISTEN_PORT`     | `5144`  | Port TCP syslog                             |
| `TENANT_ID`       | -       | Injecte runtime                             |
| `DATASOURCE_ID`   | -       | Injecte runtime                             |
| `QUICKWIT_ENDPOINT` | -     | Injecte runtime                             |

## Mapping OCSF
| Champ Unbound   | Champ OCSF             |
|-----------------|------------------------|
| client IP       | `src_endpoint.ip`      |
| QNAME           | `query.hostname`       |
| QTYPE           | `query.type`           |
| QCLASS          | `query.class`          |
| syslog ts       | `time`                 |
| -               | `activity_id = 1` (Query) |

## Limites connues
- Le parser ne capture que les requetes (pas les reponses ni les blocages de blocklist) — une requete bloquee apparait d'abord comme Query puis comme reponse NXDOMAIN (non capturee ici). Suffisant pour le threat hunting de domaines resolus.
- Le timestamp Unix dans `[1713091425]` est ignore au profit du timestamp syslog (plus fiable apres relay).

## Liens
- [Unbound logging docs](https://unbound.docs.nlnetlabs.nl/en/latest/manpages/unbound.conf.html#logging)
- [OCSF 4003 DNS Activity](https://schema.ocsf.io/classes/dns_activity)
