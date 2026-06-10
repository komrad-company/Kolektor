# Port registry — the collective allocates, no parser squats

Default listener ports for every network-fed source. A new parser claims the
next free port in its block and records it here **before** writing the source.
File-fed parsers (env-var paths, no listener) are listed for completeness.

## TCP/UDP syslog block — 5140–5159

| Port | Source | Protocol |
|---|---|---|
| 5140 | network/opnsense | syslog |
| 5141 | linux/syslog | syslog |
| 5142 | linux/auth-log | syslog |
| 5143 | linux/auditd | syslog |
| 5144 | network/fortinet-fortigate | syslog |
| 5145 | network/unbound | syslog |
| 5146 | linux/postfix | syslog |

## Endpoint block — 1514

| Port | Source | Protocol |
|---|---|---|
| 1514 | endpoint/crowdstrike-falcon | syslog (SIEM connector) |

## HTTP push block — 8514–8539

| Port | Source | Protocol |
|---|---|---|
| 8514 | identity/windows-security-evtx | HTTP (winlogbeat/NXLog) |
| 8515 | identity/windows-sysmon | HTTP (winlogbeat/NXLog) |
| 8516 | cloud/aws-cloudtrail | HTTP |
| 8517 | cloud/microsoft-365-audit | HTTP push |
| 8518 | identity/microsoft-entra | HTTP push |
| 8519 | endpoint/falco | HTTP (falcosidekick) |
| 8520 | endpoint/crowdsec | HTTP (notification plugin) |
| 8521 | endpoint/microsoft-defender-endpoint | HTTP (streaming API) |
| 8522 | identity/okta | HTTP (System Log fetcher) |
| 8523 | identity/google-workspace | HTTP (Reports API fetcher) |

## File-fed sources — no listener (env-var path)

| Env var | Source |
|---|---|
| `${K8S_AUDIT_LOG}` | cloud/kubernetes-audit |
| `${SURICATA_EVE_LOG}` | network/suricata-eve |
| `${CLOUDFLARE_HTTP_LOG}` | web/cloudflare-http |
| `${NGINX_ACCESS_LOG}` | web/nginx |
| `${TRAEFIK_ACCESS_LOG}` | web/traefik |
| `${ZEEK_LOG_DIR}` | network/zeek |
