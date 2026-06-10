# AWS CloudTrail — API Activity

## Description
Collects AWS CloudTrail logs (JSON).
Normalises to OCSF class 6003 (API Activity).

## Expected format
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

## Source-side configuration

### Option 1: S3 + SQS (recommended)
Configure CloudTrail to write to S3, then Vector reads via the `aws_s3` source.

### Option 2: HTTP forward
Use a Lambda or a Firehose to forward events as JSON to Vector HTTP.

### Option 3: Local file
If the logs are downloaded/synchronised locally.

## Variables
| Variable    | Default | Description        |
|------------|---------|--------------------|
| LISTEN_PORT | 8516   | HTTP listen port   |

## OCSF mapping
| CloudTrail        | OCSF                      |
|-------------------|---------------------------|
| eventName         | api.operation, activity_id (verb) |
| eventSource       | api.service.name          |
| sourceIPAddress   | src_endpoint.ip           |
| userIdentity      | actor.user.*              |
| errorCode         | api.response.error        |
| awsRegion         | cloud.region              |

## Links
- [CloudTrail record format](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-event-reference-record-contents.html)
- [Vector HTTP source](https://vector.dev/docs/reference/configuration/sources/http_server/)
