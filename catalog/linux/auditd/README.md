# Linux auditd

## Description
Collecte les logs du daemon d'audit Linux (`auditd`). Normalise en OCSF :
- SYSCALL/EXECVE/PROCTITLE → classe 1003 (Process Activity)
- USER_AUTH/USER_LOGIN/CRED_ACQ → classe 3002 (Authentication)

## Format attendu
```
type=SYSCALL msg=audit(EPOCH.MS:SERIAL): key=value key=value ...
type=USER_AUTH msg=audit(EPOCH.MS:SERIAL): pid=NNN uid=NNN ...
```

## Configuration cote source

### auditd rules (/etc/audit/rules.d/)
```
# Surveillance des executions privilegiees
-a always,exit -F path=/usr/bin/passwd -F perm=x -F auid>=1000 -F auid!=4294967295 -k privileged
-a always,exit -F path=/usr/bin/sudo -F perm=x -F auid>=1000 -F auid!=4294967295 -k privileged

# Surveillance des fichiers sensibles
-w /etc/shadow -p wa -k sensitive_files
-w /etc/passwd -p wa -k sensitive_files
```

### Envoi vers Vector
Configurer audisp-remote ou copier `/var/log/audit/audit.log` via rsync/filebeat.

## Variables
| Variable       | Default                    | Description           |
|---------------|----------------------------|-----------------------|
| AUDIT_LOG_PATH | /var/log/audit/audit.log  | Chemin du fichier log |

## Liens
- [auditd documentation](https://man7.org/linux/man-pages/man8/auditd.8.html)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
