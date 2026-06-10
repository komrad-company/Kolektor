# Windows Security Event Log (EVTX)

## Description
Collecte les evenements de securite Windows via Winlogbeat (format JSON).
Normalise en OCSF selon le type d'event :
- 4624/4625/4634/4647/4648 → classe 3002 (Authentication)
- 4720/4732 → classe 3001 (Account Change)
- 4688/4698 → classe 1007 (Process Activity)

## Events couverts
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

## Format attendu
JSON Winlogbeat avec structure `.winlog.event_id`, `.winlog.event_data.*`, `.winlog.computer_name`.

## Configuration cote source

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
| Variable    | Default | Description              |
|------------|---------|--------------------------|
| LISTEN_PORT | 8514   | Port HTTP d'ecoute       |

## Liens
- [Windows Security Event IDs](https://learn.microsoft.com/en-us/windows/security/threat-protection/auditing/security-auditing-overview)
- [Winlogbeat](https://www.elastic.co/beats/winlogbeat)
