# Contribution guide — New parser

## Add a new source

1. Copy the template:
   ```bash
   cp -r _schema/template.toml catalog/<category>/<source>/vector.toml
   ```

2. Edit `vector.toml`:
   - Adapt the source (syslog, file, http...)
   - Write the parsing VRL in the `parse_and_normalize` transform
   - Map to the mandatory OCSF fields
   - Keep the source log in `.raw` for valid OCSF events
   - Route parsing failures to `raw-logs`, not to an OCSF index

3. Create the tests in `tests/` (minimum 3):
   - `nominal.toml` — standard event, all fields present
   - `optional_missing.toml` — optional fields missing
   - `malformed.toml` — invalid input, must exit to `raw-logs` with `parse_status = "failed"` without exiting to OCSF

4. Create a `README.md` documenting:
   - Description of the source
   - Expected log format
   - Source-side configuration (how to send the logs)
   - Specific environment variables
   - Links to the official documentation

5. Validate:
   ```bash
   # Vector 0.54 --no-environment does NOT disable ${VAR} expansion:
   # use ci/validate.sh (injects dummy vars) or export them yourself.
   bash ci/validate.sh
   bash ci/test.sh
   ```

## Mandatory OCSF fields

Each normalized event must contain:

| Field           | Type   | Description                    |
|-----------------|--------|--------------------------------|
| `class_uid`     | int    | OCSF class (e.g. 4001)        |
| `category_uid`  | int    | OCSF category (e.g. 4)        |
| `severity_id`   | int    | 0=Unknown, 1=Info, 2=Low...   |
| `time`          | int    | Epoch milliseconds             |
| `metadata`      | object | `product.name`, `vendor_name`  |
| `tenant_id`     | string | Injected via `$TENANT_ID`      |
| `datasource_id` | string | Injected via `$DATASOURCE_ID`  |
| `raw`           | string | Original message preserved     |
| `uid`           | string | UUID shared with raw-logs when investigation is needed |

## Conventions

- Files in TOML
- VRL inline in the transform (no separate `.vrl` file)
- Runtime variables as `${ENV_VAR}` with defaults where applicable
- Test logs: real raw samples, not invented
- No unparsed event must be sent to an OCSF index
- Dynamic OCSF fields must be routed to the Quickwit index matching their `class_uid`

## Collector / parser convention

Kolektor separates log retrieval from normalization:

- a **collector** retrieves the logs from the source: syslog, file, cursor-based
  pull API, object storage + queue, Event Hub/EventBridge, or Logpush HTTP;
- a Vector **parser** transforms a canonical raw format into OCSF and routes to
  the Quickwit indexes.

For cloud/SaaS sources, do not bury pagination, OAuth, retry/backoff
or cursor management in the VRL. Prefer a dedicated collector that drops
line-delimited JSON or pushes JSON objects to Vector. The parser must stay
testable with raw fixtures and reusable regardless of the transport.

## raw / OCSF / raw-logs convention

Each valid OCSF event keeps a copy of the source log in `.raw`.
For text sources (`file`, syslog payload), `.raw` must be the original message
or the closest possible reconstructed line. For JSON sources already decoded
by Vector (`http_server encoding = "json"`), capture `raw_msg = encode_json(.)`
before adding `tenant_id`, `datasource_id` or the OCSF fields.

Each valid event also carries a `uid`:

```vrl
_ts  = to_string(.timestamp) ?? ""
_pid = if .procid != null { "[" + to_string!(.procid) + "]" } else { "" }
.raw = _ts + " " + (string(.hostname) ?? "") + " " + (string(.appname) ?? "") + _pid + ": " + _msg
.uid = uuid_v4()
```

Parsed events are not systematically copied into `raw-logs`:
their raw form is already in `.raw`. `raw-logs` serves to isolate parsing failures
and unsupported formats.

Expected pattern for failures:

```toml
[transforms.filter_failed]
type      = "filter"
inputs    = ["parse_and_normalize"]
condition = '.class_uid == 0'

[transforms.raw_failed]
type   = "remap"
inputs = ["filter_failed"]
source = '''
  . = {
    "uid":           .uid,
    "time":          .time,
    "received_time": .received_time,
    "tenant_id":     .tenant_id,
    "datasource_id": .datasource_id,
    "source_type":   "category/source",
    "parser":        "source",
    "parse_status":  "failed",
    "parse_error":   "reason_code",
    "raw":           .raw
  }
'''
```

The `raw_failed` sink sends to `${QUICKWIT_ENDPOINT}/api/v1/raw-logs/ingest`.

The `uid` lets Kontrol correlate a normalized OCSF event with its original raw
line when it exists in another stream, and provides a stable identifier
for quarantine events.

## Dynamic routing when `class_uid` varies

If a source produces multiple OCSF classes (e.g. auditd = 1003 + 3002,
windows-evtx = 3001/3002/1003), a `route` transform + one sink per
target Quickwit index is required. A single sink to `ocsf-endpoint` with 3002
events inside = data in the wrong place. See [catalog/linux/auditd/vector.toml](../catalog/linux/auditd/vector.toml).

The `manifest.yaml` must declare the multiple outputs with `ocsf_outputs`:

```yaml
display_name: Windows Sysmon
default_port: 8515
ocsf_outputs:
  - class_uid: 1003
    category_uid: 1
    index: ocsf-endpoint
    route: endpoint
  - class_uid: 4001
    category_uid: 4
    index: ocsf-network
    route: network
  - class_uid: 4003
    category_uid: 4
    index: ocsf-dns
    route: dns
```

For a single-class source, the legacy fields `ocsf_class_uid` and
`ocsf_category_uid` remain accepted; the seed automatically generates one output.

## Expected tests

The tests must verify:

- a nominal case through to the normalize transform;
- an optional case with missing fields;
- a malformed case with `.class_uid == 0` at the parsing transform;
- an output from the `raw_failed` transform with `parse_status = "failed"`,
  `source_type`, `parse_error`, `raw`, `uid`, `tenant_id` and `datasource_id`;
- for multi-class parsers, at least one case per route (`endpoint`,
  `identity`, `network`, `dns`, etc.).
