# CrowdStrike Falcon — SIEM Connector

## Description
Collects CrowdStrike Falcon events via the SIEM Connector (JSON syslog).
Normalises to OCSF depending on the event type:
- DetectionSummaryEvent → class 2001 (Security Finding)
- ProcessRollup2 → class 1007 (Process Activity)
- NetworkDetection → class 4001 (Network Activity)

## Expected format
JSON wrapped in syslog:
```json
{"metadata":{"eventType":"DetectionSummaryEvent","eventCreationTime":1713091425000,...},"SeverityName":"High","ComputerName":"WS01",...}
```

## Source-side configuration

### SIEM Connector
```bash
# Falcon SIEM Connector config
output:
  syslog:
    host: "<vector-host>"
    port: 1514
    protocol: tcp
    format: json
```

### Streaming API
Configure the CrowdStrike Streaming API to send over TCP syslog.

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 1514   | TCP syslog port    |

## Links
- [CrowdStrike SIEM Connector](https://falcon.crowdstrike.com/documentation/9/falcon-siem-connector)
- [CrowdStrike Event Streams](https://falcon.crowdstrike.com/documentation/89/event-streams)
