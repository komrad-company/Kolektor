# Catalogue decisions — the collective rules, the parsers obey

> Decision log for the Kolektor catalogue. Each entry is a decree: what was
> chosen, why, and what it breaks. Ratified 2026-06-10.

## D1 — OCSF taxonomy: strict alignment on the specification

The catalogue aligns on the published OCSF class/category uids. Internal
divergence is debt, not a convention.

| Concern | Was | Now | Affected parsers |
|---|---|---|---|
| API Activity | 6001 (Web Resources Activity) | **6003** | microsoft-365-audit, microsoft-entra (audit events) |
| Process Activity | 1003 (Kernel Activity) | **1007** | auditd, windows-sysmon, crowdstrike-falcon, windows-security-evtx (process events) |
| Module Activity (sysmon EventID 7) | 1003 | **1005** | windows-sysmon |
| Generic syslog | 6001 | **0 (Base Event)** | syslog — see D2 |
| `activity_id` | absent on most | **mandatory** on every classified event | all |
| "Other" enum | mapped to `0` (Unknown) | **`99`** | auth-log, windows-security-evtx |

Category is always `floor(class_uid / 1000)` (except 6-digit extension
classes). Reference enums captured below in D6.

**Breaks:** Korelator rules filtering on `class_uid` must follow (6001→6003,
1003→1007). Quickwit history keeps the old uids — queries spanning the cutover
must `OR` both. The `ocsf-audit` index keeps receiving 6003 events (index id
unchanged, only the uid inside the document changes).

## D2 — Generic syslog is a Base Event, not Event Log Activity

Researching the spec corrected an earlier assumption: **Event Log Activity
(class 1008)** describes *operations on the logging service itself* — clear,
delete, rotate, disable (MITRE T1070.001 territory), **not** generic log
lines. Mapping arbitrary syslog onto 1008 would be wrong.

A generic syslog line carries no intrinsic OCSF activity. The spec-correct
target is **Base Event (class_uid 0, category_uid 0)**. The `syslog` parser
therefore:
- emits Base Event with the severity normalisation preserved,
- routes to **raw-logs** (the generic bucket) with `parse_status = "parsed"`,
- is documented as a fallback: app-specific parsers (`auth-log`, service
  parsers) are always preferred when the source is known.

## D3 — Quickwit retention

No index had a retention policy; everything was kept forever, `raw-logs`
included — a disk bomb under multi-tenant load.

| Index | Retention | Schedule |
|---|---|---|
| `raw-logs` | **30 days** | daily |
| `ocsf-*` (all 7) | **90 days** | daily |

Set in each `init/indexes/*.json` via the Quickwit `retention` block. Tunable
per index; revisit with Kupol when real volume is known.

## D4 — Vector version aligned on 0.55

CI and the image built on 0.54 while the homelab pod ran 0.55 — testing a
version other than the one in production. Everything moves to **0.55.0-debian**
(CI image, Dockerfile base pinned by digest, kolektor deployment). The 17
sources are re-validated under 0.55 before release.

## D5 — Orphan `ocsf` index purged

The Quickwit homelab carried a stray `ocsf` index from an early experiment,
matching no sink in the catalogue. Deleted. The catalogue's index set is
exactly: `raw-logs` + `ocsf-{network,http,dns,endpoint,identity,audit,k8s}`.

## D6 — OCSF reference enums (authoritative, from schema.ocsf.io)

Used by the schema validator (`ci/validate_ocsf.py`) and every parser.

**severity_id:** 0 Unknown · 1 Informational · 2 Low · 3 Medium · 4 High ·
5 Critical · 6 Fatal · 99 Other

**Process Activity (1007) activity_id:** 1 Launch · 2 Terminate · 3 Open ·
4 Inject · 5 Set User ID · 99 Other

**Authentication (3002) activity_id:** 1 Logon · 2 Logoff · 3 Authentication
Ticket · 4 Service Ticket Request · 5 Service Ticket Renew · 6 Preauth ·
7 Account Switch · 99 Other

**API Activity (6003) activity_id:** 1 Create · 2 Read · 3 Update · 4 Delete ·
99 Other

**Network Activity (4001) activity_id:** 1 Open · 2 Close · 3 Reset · 4 Fail ·
5 Refuse · 6 Traffic · 7 Listen · 99 Other

**HTTP Activity (4002) activity_id:** 1 Connect · 2 Delete · 3 Get · 4 Head ·
5 Options · 6 Post · 7 Put · 8 Trace · 9 Patch · 99 Other

**DNS Activity (4003) activity_id:** 1 Query · 2 Response · 6 Traffic · 99 Other

**Event Log Activity (1008) activity_id:** 1 Clear · 2 Delete · 3 Export ·
4 Archive · 5 Rotate · 6 Start · 7 Stop · 8 Restart · 9 Enable · 10 Disable

## D7 — Tooling becomes the guardrail

Three CI guards are added so the review's bug classes cannot recur silently:

- **`ci/lint_vrl.py`** — greps the antipattern set: inline `if` inside object
  literals, dead `to_x(null) ?? fallback`, bare `to_int!`/`to_string!` on
  log-derived data, missing mandatory OCSF field assignments.
- **`ci/validate_ocsf.py`** — runs each parser's nominal test through Vector
  and validates the output against the D6 enums and the mandatory-field set
  per class.
- **manifest `version`** — every `manifest.yaml` carries a semver, surfaced in
  `index.json`, so Kolektor-kontroler can display and diff what runs.

## D8 — Tier-1 source coverage

The catalogue gains the sources a credible SIEM cannot omit, prioritised by
dogfooding value then market share: **falco**, **crowdsec** (both already
running in the homelab), **microsoft-defender-endpoint**, **zeek**,
**postfix** (opens OCSF category 4009 — email), **okta**, **google-workspace**
(non-Microsoft identity). Tier 2/3 (PAN-OS, Cisco ASA, GuardDuty, PowerShell
4104, etc.) tracked separately.
