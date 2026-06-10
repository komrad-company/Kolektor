# OCSF conventions — decrees of the collective

> Ratified 2026-06-10. These rulings are not suggestions. Every parser,
> existing or future, complies. The linter (`ci/lint_vrl.py`) enforces what
> can be enforced statically; the reviewer enforces the rest.

## Class taxonomy — strict OCSF alignment

Classes follow the official OCSF schema (schema.ocsf.io), verified against
the upstream definitions — never from memory.

| Signal | class_uid | category_uid | Index |
|---|---|---|---|
| Process / syscall execution | **1007** Process Activity | 1 | ocsf-endpoint |
| File events (sysmon 11) | 1001 File System Activity | 1 | ocsf-endpoint |
| Authentication (logon/logoff) | 3002 Authentication | 3 | ocsf-identity |
| Account/directory management | 3001 Account Change | 3 | ocsf-identity |
| Network flows / firewall | 4001 Network Activity | 4 | ocsf-network |
| HTTP access | 4002 HTTP Activity | 4 | ocsf-http |
| DNS | 4003 DNS Activity | 4 | ocsf-dns |
| Security findings / EDR & IDS alerts | 2001 Security Finding | 2 | ocsf-endpoint |
| API /