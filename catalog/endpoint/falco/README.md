# Falco — Security Finding

## Description
Collects Falco runtime-security alerts (JSON), pushed over HTTP by
falcosidekick. Normalises to OCSF class 2001 (Security Finding), routed to the
`ocsf-endpoint` index.

## Expected format
```json
{
  "output": "10:23:45.123456789: Warning A shell was spawned in a container ...",
  "priority": "Warning",
  "rule": "Terminal shell in container",
  "time": "2024-04-14T10:23:45.123456789Z",
  "output_fields": {
    "container.id": "a1b2c3d4e5f6",
    "proc.cmdline": "bash",
    "user.name": "root",
    "k8s.pod.name": "nginx-7d8b49557c-2xk4p"
  },
  "source": "syscall",
  "tags": ["container", "shell", "mitre_execution"],
  "hostname": "node1"
}
```

## Source-side configuration
Falco does not push over HTTP on its own. Deploy
[falcosidekick](https://github.com/falcosecurity/falcosidekick) alongside Falco
and enable its generic webhook output pointing at this parser:

```yaml
# falco.yaml
json_output: true
http_output:
  enabled: true
  url: "http://falcosidekick:2801/"
```

```yaml
# falcosidekick env / values
WEBHOOK_ADDRESS: "http://<vector-host>:8519/"
```

falcosidekick forwards each Falco alert as a single JSON object via HTTP POST.

## Variables
| Variable    | Default | Description        |
|-------------|---------|--------------------|
| LISTEN_PORT | 8519    | HTTP listener port |

## OCSF mapping
| Falco          | OCSF                                |
|----------------|-------------------------------------|
| rule           | finding_info.title                  |
| tags           | finding_info.types                  |
| output         | finding_info.desc                   |
| priority       | severity_id, unmapped.priority      |
| time           | time (epoch ms)                     |
| hostname       | src_endpoint.hostname               |
| source         | unmapped.source                     |
| output_fields  | unmapped.output_fields              |

`severity_id` from Falco priority: Emergency/Alert/Critical → 5, Error → 4,
Warning → 3, Notice → 2, Informational/Debug → 1, otherwise → 1.
`activity_id` is 1 (Generate) — a fired finding.

Events missing the `rule` field are routed to `raw-logs` with
`parse_status = "failed"` and `parse_error = "falco_rule_missing"`; they never
reach an OCSF index.

## Links
- [Falco output fields / JSON output](https://falco.org/docs/reference/rules/supported-fields/)
- [falcosidekick](https://github.com/falcosecurity/falcosidekick)
- [OCSF Security Finding (2001)](https://schema.ocsf.io/classes/security_finding)
- [Vector HTTP source](https://vector.dev/docs/reference/configuration/sources/http_server/)
