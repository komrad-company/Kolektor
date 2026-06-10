# CrowdStrike Falcon — SIEM Connector

## Description
Collecte les events CrowdStrike Falcon via le SIEM Connector (JSON syslog).
Normalise en OCSF selon le type d'event :
- DetectionSummaryEvent → classe 2001 (Security Finding)
- ProcessRollup2 → classe 1007 (Process Activity)
- NetworkDetection → classe 4001 (Network Activity)

## Format attendu
JSON encapsule dans du syslog :
```json
{"metadata":{"eventType":"DetectionSummaryEvent","eventCreationTime":1713091425000,...},"SeverityName":"High","ComputerName":"WS01",...}
```

## Configuration cote source

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
Configurer le Streaming API CrowdStrike pour envoyer en TCP syslog.

## Variables
| Variable    | Default | Description        |
|------------|---------|---------------------|
| LISTEN_PORT | 1514   | Port syslog TCP    |

## Liens
- [CrowdStrike SIEM Connector](https://falcon.crowdstrike.com/documentation/9/falcon-siem-connector)
- [CrowdStrike Event Streams](https://falcon.crowdstrike.com/documentation/89/event-streams)
