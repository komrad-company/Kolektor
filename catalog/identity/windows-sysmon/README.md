# Windows Sysmon

## Description
Collects Sysmon events via Winlogbeat or NXLog (JSON format).
Normalises to OCSF depending on the event type:
- Event 1 (Process Create) → class 1007 (Process Activity)
- Event 7 (Image/Module Loaded) → class 1005 (Module Activity)
- Event 3 (Network Connection) → class 4001 (Network Activity)
- Event 22 (DNS Query) → class 4003 (DNS Activity)

## Covered events
| Event ID | Description              | OCSF Class | activity_id |
|----------|--------------------------|------------|-------------|
| 1        | Process Create           | 1007       | 1 (Launch)  |
| 7        | Image/Module Loaded      | 1005       | 99 (Other)  |
| 3        | Network Connection       | 4001       | 6 (Traffic) |
| 22       | DNS Query                | 4003       | 1 (Query)   |

Any other Event ID is unsupported and routed to `raw-logs` with
`parse_error = "sysmon_event_id_unsupported"`.

## Event time
`.time` uses the event time (`.winlog.event_data.UtcTime`, or
`.@timestamp` as a fallback), never the HTTP ingestion timestamp. The ingestion
timestamp is only a last resort when no event time is present.

## Expected format
Winlogbeat JSON (`.winlog.event_id`, `.winlog.event_data.*`) or NXLog
(`.event_id`, `.event_data.*` at root level).

## Source-side configuration

### Winlogbeat (winlogbeat.yml)
```yaml
winlogbeat.event_logs:
  - name: Microsoft-Windows-Sysmon/Operational
    event_id: 1, 3, 7, 22

output.http:
  hosts: ["http://<vector-host>:8515"]
```

### Sysmon config (sysmonconfig.xml)
Use [SwiftOnSecurity sysmon config](https://github.com/SwiftOnSecurity/sysmon-config)
or [olafhartong sysmon modular](https://github.com/olafhartong/sysmon-modular).

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 8515   | HTTP listen port   |

## Links
- [Sysmon](https://learn.microsoft.com/en-us/sysinternals/downloads/sysmon)
- [Winlogbeat](https://www.elastic.co/beats/winlogbeat)
