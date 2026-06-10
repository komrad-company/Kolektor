# Windows Security Event Log (EVTX)

## Description
Collects Windows security events via Winlogbeat (JSON format).
Normalises to OCSF depending on the event type:
- 4624/4625/4634/4647/4648 → class 3002 (Authentication)
- 4720/4732 → class 3001 (Account Change)
- 4688/4698 → class 1007 (Process Activity)

## Covered events
| Event ID | Description                           | OCSF Class | activity_id |
|----------|---------------------------------------|------------|-------------|
| 4624     | Successful logon                      | 3002       | 1 Logon     |
| 4625     | Failed logon                          | 3002       | 1 Logon     |
| 4634     | Logoff                                | 3002       | 2 Logoff    |
| 4647     | User initiated logoff                 | 3002       | 2 Logoff    |
| 4648     | Logon with explicit credentials       | 3002       | 1 Logon     |
| 4688     | New process created                   | 1007       | 1 Launch    |
| 4698     | Scheduled task created                | 1007       | 99 Other    |
| 4720     | User account created                  | 3001       | 99 Other    |
| 4732     | Member added to security group        | 3001       | 99 Other    |

## Expected format
Winlogbeat JSON with structure `.winlog.event_id`, `.winlog.event_data.*`, `.winlog.computer_name`.

## Source-side configuration

### Winlogbeat (winlogbeat.yml)
```yaml
winlogbeat.event_logs:
  - name: Security
    event_id: 4624, 4625, 4634, 4647, 4648, 4688, 4698, 4720, 4732

output.http:
  hosts: ["http://<vector-host>:8514"]
  codec.json: {}
```

## Variables
| Variable    | Default | Description        |
|------------|---------|--------------------|
| LISTEN_PORT | 8514   | HTTP listen port   |

## Links
- [Windows Security Event IDs](https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/security-auditing-overview)
- [Winlogbeat](https://www.elastic.co/beats/winlogbeat)
