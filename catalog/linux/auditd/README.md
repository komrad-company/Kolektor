# Linux auditd

## Description
Collects logs from the Linux audit daemon (`auditd`). Normalises to OCSF:
- SYSCALL/EXECVE/PROCTITLE → class 1003 (Process Activity)
- USER_AUTH/USER_LOGIN/CRED_ACQ → class 3002 (Authentication)

## Expected format
```
type=SYSCALL msg=audit(EPOCH.MS:SERIAL): key=value key=value ...
type=USER_AUTH msg=audit(EPOCH.MS:SERIAL): pid=NNN uid=NNN ...
```

## Source-side configuration

### auditd rules (/etc/audit/rules.d/)
```
# Monitoring of privileged executions
-a always,exit -F path=/usr/bin/passwd -F perm=x -F auid>=1000 -F auid!=4294967295 -k privileged
-a always,exit -F path=/usr/bin/sudo -F perm=x -F auid>=1000 -F auid!=4294967295 -k privileged

# Monitoring of sensitive files
-w /etc/shadow -p wa -k sensitive_files
-w /etc/passwd -p wa -k sensitive_files
```

### Shipping to Vector
Configure audisp-remote or copy `/var/log/audit/audit.log` via rsync/filebeat.

## Variables
| Variable       | Default                    | Description    |
|---------------|----------------------------|----------------|
| AUDIT_LOG_PATH | /var/log/audit/audit.log  | Log file path  |

## Links
- [auditd documentation](https://man7.org/linux/man-pages/man8/auditd.8.html)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
