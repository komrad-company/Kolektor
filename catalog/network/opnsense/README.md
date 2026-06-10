# OPNsense Firewall — filterlog

## Description
Collects OPNsense filterlog logs over syslog.
Normalized to OCSF class 4001 (Network Activity).

## Expected format
CSV over syslog:
```
rulenr,subrulenr,anchorname,label,interface,reason,action,direction,ip_version,tos,ecn,ttl,id,offset,flags,proto_id,proto,length,src_ip,dst_ip,src_port,dst_port,...
```

## Source-side configuration

### OPNsense (System > Settings > Logging)
- Remote log servers: `<vector-host>:<port>` (TCP)
- Check "Firewall Events" in the facilities

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 514    | TCP syslog port    |

## OCSF mapping
| filterlog  | OCSF                          |
|-----------|-------------------------------|
| pass      | action=Allow, action_id=1     |
| block     | action=Deny, action_id=2      |
| interface | unmapped.interface            |
| src/dst   | src_endpoint/dst_endpoint     |
| proto     | connection_info.protocol_name |

## Links
- [OPNsense filterlog format](https://docs.opnsense.org/development/frontend/diagnostics_log.html)
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
