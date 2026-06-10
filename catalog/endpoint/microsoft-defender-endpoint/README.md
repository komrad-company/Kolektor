# Microsoft Defender for Endpoint — Streaming API

## Description
Ingests Microsoft Defender for Endpoint alerts and events delivered through the
streaming API (Event Hub or HTTP forwarder) as JSON. Each event is normalised to
OCSF **Security Finding** (class 2001, category 2) and routed to the
`ocsf-endpoint` index. Events without a `Title` are not alerts and are routed to
`raw-logs` with `parse_status = "failed"`.

## Expected format
Defender Advanced Hunting schema (AlertEvidence / DeviceEvents), one JSON object
per event:
```json
{
  "Timestamp": "2024-04-14T10:23:45.1234567Z",
  "DeviceId": "abc",
  "DeviceName": "WIN-01",
  "AlertId": "da123",
  "Title": "Suspicious PowerShell",
  "Severity": "High",
  "Category": "Execution",
  "FileName": "powershell.exe",
  "FolderPath": "C:\\Windows\\System32\\WindowsPowerShell\\v1.0",
  "ProcessCommandLine": "powershell -enc ...",
  "AccountName": "jdoe",
  "AccountDomain": "CORP",
  "InitiatingProcessFileName": "cmd.exe",
  "RemoteIP": "1.2.3.4"
}
```

## Source-side configuration
Configure the Defender streaming API to export to an Event Hub, then forward
each record as a JSON POST to the Vector HTTP listener:
```
POST http://<vector-host>:8521/
Content-Type: application/json
```
The forwarder must send one JSON object per request body (or a JSON array — the
`http_server` source decodes both). No batching framing is required.

## Severity mapping
| Defender `Severity` | OCSF `severity_id` |
|---------------------|--------------------|
| Informational       | 1                  |
| Low                 | 2                  |
| Medium              | 3                  |
| High                | 4                  |
| Critical            | 5                  |
| (anything else)     | 1                  |

## OCSF mapping
| OCSF field            | Source                                   |
|-----------------------|------------------------------------------|
| `class_uid`           | 2001 (Security Finding)                  |
| `category_uid`        | 2 (Findings)                             |
| `activity_id`         | 1 (Security Finding)                     |
| `time`                | `Timestamp` (ISO 8601 → epoch ms)        |
| `finding_info.title`  | `Title`                                  |
| `finding_info.uid`    | `AlertId`                                |
| `finding_info.types`  | `[Category]`                             |
| `process.file.name`   | `FileName`                               |
| `process.file.path`   | `FolderPath`                             |
| `process.cmd_line`    | `ProcessCommandLine`                     |
| `actor.user.name`     | `AccountName`                            |
| `actor.user.domain`   | `AccountDomain`                          |
| `actor.process.name`  | `InitiatingProcessFileName`              |
| `device.uid`          | `DeviceId`                               |
| `device.hostname`     | `DeviceName`                             |
| `dst_endpoint.ip`     | `RemoteIP`                               |

## Variables
| Variable    | Default | Description                |
|-------------|---------|----------------------------|
| LISTEN_PORT | 8521    | HTTP listener port         |

## Links
- [Stream Defender for Endpoint events](https://learn.microsoft.com/en-us/defender-endpoint/raw-data-export)
- [Advanced Hunting schema reference](https://learn.microsoft.com/en-us/defender-xdr/advanced-hunting-schema-tables)
