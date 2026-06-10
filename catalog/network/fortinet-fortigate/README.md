# Fortinet FortiGate — traffic/utm logs

## Description
Collects FortiGate logs (key=value format) over syslog.
Normalized to OCSF class 4001 (Network Activity).

## Expected format
```
date=2024-04-14 time=10:23:45 devname="FG100F" type="traffic" subtype="forward" level="notice" srcip=10.0.0.5 srcport=54321 dstip=8.8.8.8 dstport=443 action=accept proto=6 service="HTTPS" sentbyte=1234 rcvdbyte=5678 ...
```

## Source-side configuration

### FortiOS CLI
```
config log syslogd setting
  set status enable
  set server "<vector-host>"
  set port <port>
  set reliable enable
  set format default
end
```

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 514    | TCP syslog port    |

## OCSF mapping
| FortiGate     | OCSF                            |
|--------------|----------------------------------|
| accept/allow | action=Allow                     |
| deny/drop    | action=Deny                      |
| srcip/dstip  | src_endpoint.ip/dst_endpoint.ip  |
| sentbyte     | traffic.bytes_out                |
| rcvdbyte     | traffic.bytes_in                 |
| service      | connection_info.protocol_name    |
| policyid     | unmapped.policyid                |

## Links
- [FortiGate log reference](https://docs.fortinet.com/document/fortigate/7.4.0/fortios-log-message-reference)
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
