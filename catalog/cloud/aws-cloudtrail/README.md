# AWS CloudTrail — API Activity

## Description
Collecte les logs CloudTrail AWS (JSON).
Normalise en OCSF classe 6001 (API Activity).

## Format attendu
```json
{
  "eventVersion": "1.08",
  "eventTime": "2024-04-14T10:23:45Z",
  "eventSource": "s3.amazonaws.com",
  "eventName": "GetObject",
  "awsRegion": "us-east-1",
  "sourceIPAddress": "10.0.0.5",
  "userIdentity": {"type": "IAMUser", "userName": "admin", ...},
  ...
}
```

## Configuration cote source

### Option 1 : S3 + SQS (recommande)
Configurer CloudTrail pour ecrire dans S3, puis Vector lit via source `aws_s3`.

### Option 2 : HTTP forward
Utiliser un Lambda ou un Firehose pour forwarder les events en JSON vers Vector HTTP.

### Option 3 : Fichier local
Si les logs sont telecharges/synchronises localement.

## Variables
| Variable    | Default | Description              |
|------------|---------|--------------------------|
| LISTEN_PORT | 8516   | Port HTTP d'ecoute       |

## Mapping OCSF
| CloudTrail        | OCSF                      |
|-------------------|---------------------------|
| eventName         | api.operation             |
| eventSource       | api.service.name          |
| sourceIPAddress   | src_endpoint.ip           |
| userIdentity      | actor.user.*              |
| errorCode         | api.response.error        |
| awsRegion         | cloud.region              |

## Liens
- [CloudTrail record format](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-event-reference-record-contents.html)
- [Vector HTTP source](https://vector.dev/docs/reference/configuration/sources/http_server/)
