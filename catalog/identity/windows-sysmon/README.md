# Windows Sysmon

## Description
Collecte les evenements Sysmon via Winlogbeat ou NXLog (format JSON).
Normalise en OCSF selon le type d'event :
- Event 1 (Process Create) → classe 1007 (Process Activity)
- Event 7 (Image/Module Loaded) → classe 1005 (Module Activity)
- Event 3 (Network Connection) → classe 4001 (Network Activity)
- Event 22 (DNS Query) → classe 4003 (DNS Activity)

## Events couverts
| Event ID | Description              | OCSF Class | activity_id |
|----------|--------------------------|------------|-------------|
| 1        | Process Create           | 1007       | 1 (Launch)  |
| 7        | Image/Module Loaded      | 1005       | 99 (Other)  |
| 3        | Network Connection       | 4001       | 6 (Traffic) |
| 22       | DNS Query                | 4003       | 1 (Query)   |

Tout autre Event ID est non supporte et route vers `raw-logs` avec
`parse_error = "sysmon_event_id_unsupported"`.

## Temps de l'evenement
`.time` utilise le temps de l'evenement (`.winlog.event_data.UtcTime`, ou
`.@timestamp` a defaut), jamais le timestamp d'ingestion HTTP. Le timestamp
d'ingestion ne sert que de dernier recours si aucun temps d'evenement n'est
present.

## Format attendu
JSON Winlogbeat (`.winlog.event_id`, `.winlog.event_data.*`) ou NXLog
(`.event_id`, `.event_data.*` au niveau racine).

## Configuration cote source

### Winlogbeat (winlogbeat.yml)
```yaml
winlogbeat.event_logs:
  - name: Microsoft-Windows-Sysmon/Operational
    event_id: 1, 3, 7, 22

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
