# Suricata EVE — JSON events

## Description
Collecte les evenements Suricata EVE JSON depuis un fichier `eve.json`.
Le parser route dynamiquement :
- `alert` → OCSF 2001 Security Finding (`ocsf-endpoint`)
- `flow`, `netflow`, `tls` → OCSF 4001 Network Activity (`ocsf-network`)
- `http` → OCSF 4002 HTTP Activity (`ocsf-http`)
- `dns` → OCSF 4003 DNS Activity (`ocsf-dns`)

Les lignes JSON invalides ou les `event_type` non supportes partent dans `raw-logs`
avec `parse_status = "failed"`.

## Format attendu
Une ligne JSON par evenement :

```json
{"timestamp":"2024-04-14T10:23:45.123456+0000","event_type":"alert","src_ip":"10.0.0.5","src_port":54321,"dest_ip":"8.8.8.8","dest_port":53,"proto":"UDP","alert":{"signature_id":2024218,"signature":"ET DNS Query for suspicious domain","category":"Potentially Bad Traffic","severity":2}}
```

## Configuration cote source

### Suricata
Dans `suricata.yaml`, activer `eve-log` avec au minimum les types utiles :

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

Monter ou forwarder le fichier `eve.json` dans le pod Kolektor.

## Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `SURICATA_EVE_LOG` | `/var/log/suricata/eve.json` | Chemin du fichier EVE JSON |
| `TENANT_ID` | - | Injecte runtime |
| `DATASOURCE_ID` | - | Injecte runtime |
| `QUICKWIT_ENDPOINT` | - | Injecte runtime |

## Mapping OCSF
| Champ Suricata | Champ OCSF |
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

## Limites connues
- Les categories OCSF sont volontairement larges : le parser privilegie le routage indexable par classe.
- Les event types EVE non supportes (`stats`, `fileinfo`, etc.) sont quarantaines dans `raw-logs`.
- Le timestamp Suricata est prioritaire ; si son format est inattendu, le parser utilise `now()`.

## Liens
- [Suricata EVE JSON output](https://docs.suricata.io/en/latest/output/eve/eve-json-output.html)
- [OCSF Network Activity](https://schema.ocsf.io/classes/network_activity)
- [OCSF DNS Activity](https://schema.ocsf.io/classes/dns_activity)
- [OCSF HTTP Activity](https://schema.ocsf.io/classes/http_activity)
