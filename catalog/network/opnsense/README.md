# OPNsense Firewall — filterlog

## Description
Collecte les logs filterlog d'OPNsense via syslog.
Normalise en OCSF classe 4001 (Network Activity).

## Format attendu
CSV via syslog :
```
rulenr,subrulenr,anchorname,label,interface,reason,action,direction,ip_version,tos,ecn,ttl,id,offset,flags,proto_id,proto,length,src_ip,dst_ip,src_port,dst_port,...
```

## Configuration cote source

### OPNsense (System > Settings > Logging)
- Remote log servers : `<vector-host>:<port>` (TCP)
- Cocher "Firewall Events" dans les facilities

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 514    | Port syslog TCP    |

## Mapping OCSF
| filterlog  | OCSF                          |
|-----------|-------------------------------|
| pass      | action=Allow, action_id=1     |
| block     | action=Deny, action_id=2      |
| interface | unmapped.interface            |
| src/dst   | src_endpoint/dst_endpoint     |
| proto     | connection_info.protocol_name |

## Liens
- [OPNsense filterlog format](https://docs.opnsense.org/development/frontend/diagnostics_log.html)
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
