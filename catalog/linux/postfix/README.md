# Postfix maillog — SMTP delivery, queue, reject

## Description
Collects Postfix mail activity from `maillog` (the `mail` syslog facility) and
normalises it to OCSF class 4009 (Email Activity). The collective routes mail
events through the same syslog block as the other Linux sources.

## Events covered
- `smtpd` client connect (`connect from host[ip]`)
- `qmgr` queue envelope (`from=<...>, size=...`)
- `smtp` delivery result (`to=<...>, relay=..., status=sent|deferred|bounced`)
- `smtpd` reject (`NOQUEUE: reject: ...`)

## Expected format
The Vector `syslog` source parses the header (timestamp, hostname, appname,
procid) and exposes the message body in `.message`. Real maillog bodies:
```
smtpd[2010]: connect from mail.example.com[1.2.3.4]
qmgr[2011]: A1B2C3: from=<sender@example.com>, size=1024, nrcpt=1 (queue active)
smtp[2012]: A1B2C3: to=<rcpt@example.org>, relay=mx.example.org[5.6.7.8]:25, delay=0.5, status=sent (250 2.0.0 OK)
smtpd[2013]: NOQUEUE: reject: RCPT from unknown[9.9.9.9]: 554 5.7.1 Service unavailable; from=<x@y.com> to=<z@w.com>
```

## Source-side configuration
Postfix writes to syslog natively. Forward the `mail` facility over TCP to the
parser. With rsyslog:
```
mail.*  @@vector-host:5146
```

## Variables
| Variable          | Default                  | Description                  |
|-------------------|--------------------------|------------------------------|
| `LISTEN_PORT`     | 5146                     | Syslog TCP listener port     |
| `TENANT_ID`       | —                        | Injected by the orchestrator |
| `DATASOURCE_ID`   | —                        | Injected by the orchestrator |
| `QUICKWIT_ENDPOINT` | —                      | Quickwit ingest base URL     |

## OCSF mapping
OCSF 4009 (Email Activity) is a category-4 (Network) class. There is **no
dedicated email index** in the catalogue (see `docs/decisions.md` D5 — the
index set is `raw-logs` + `ocsf-{network,http,dns,endpoint,identity,audit,k8s}`),
so valid events are routed to **ocsf-network**. Parse failures go to
**raw-logs** with `parse_status = "failed"`.

| maillog event   | severity_id | activity_id | fields                          |
|-----------------|-------------|-------------|---------------------------------|
| connect         | 1 (Info)    | 2 (receive) | src_endpoint host/ip            |
| qmgr from       | 1 (Info)    | 1 (send)    | email.from, email.size          |
| smtp sent       | 1 (Info)    | 1 (send)    | email.to, relay, smtp_status    |
| smtp deferred   | 2 (Low)     | 1 (send)    | email.to, relay, smtp_status    |
| smtp bounced    | 3 (Medium)  | 1 (send)    | email.to, relay, smtp_status    |
| reject          | 3 (Medium)  | 99 (Other)  | src_endpoint, email.from/to     |

`activity_id` is set on every classified event (4009 has no enum entry in the
OCSF validator, so the values above are the collective's convention, never
omitted).

## Links
- [Postfix logging](https://www.postfix.org/postconf.5.html)
- [OCSF Email Activity (4009)](https://schema.ocsf.io/classes/email_activity)
- [Vector syslog source](https://vector.dev/docs/reference/configuration/sources/syslog/)
