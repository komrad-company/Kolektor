# Windows Sysmon

## Description
Collecte les evenements Sysmon via Winlogbeat (format JSON).
Normalise en OCSF selon le type d'event :
- Event 1 (Process Create), Event 7 (Image Loaded) → classe 1003
- Event 3 (Network Connection) → classe 4001
- Event 11 (File Created) → classe 1001
- Event 22 (DNS Query) → classe 4003

## Events couverts
| Event ID | Description              | OCSF Class |
|----------|--------------------------|------------|
| 1        | Process Create           | 1003       |
| 3        | Network Connection       | 4001       |
| 7        | Image Loaded (DLL)       | 1003       |
| 11       | File Created             | 1001       |
| 22       | DNS Query                | 4003       |

## Format attendu
JSON Winlogbeat/NXLog avec `.winlog.event_id`, `.winlog.event_data.*`.

## Configuration cote source

### Winlogbeat (winlogbeat.yml)
```yaml
winlogbeat.event_logs:
  - name: Microsoft-Windows-Sysmon/Operational
    event_id: 1, 3, 7, 11, 22

output.http:
  hosts: ["http://<vector-host>:8515"]
```

### Sysmon config (sysmonconfig.xml)
Utiliser [SwiftOnSecurity sysmon config](https://github.com/SwiftOnSecurity/sysmon-config)
ou [olafhartong sysmon modular](https://github.com/olafhartong/sysmon-modular).

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 8515   | Port HTTP d'ecoute |

## Liens
- [Sysmon](https://learn.microsoft.com/en-us/sysinternals/downloads/sysmon)
- [Winlogbeat](https://www.elastic.co/beats/winlogbeat)
