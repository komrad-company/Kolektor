# Linux auth.log — SSH, sudo, PAM

## Description
Collecte les evenements d'authentification Linux depuis `/var/log/auth.log`.
Normalise en OCSF classe 3002 (Authentication).

## Events couverts
- SSH login accepted (password, publickey)
- SSH login failed (password, publickey, invalid user)
- sudo command execution
- PAM authentication events

## Format attendu
```
Apr 14 10:23:45 hostname sshd[12345]: Accepted publickey for user from 1.2.3.4 port 54321 ssh2
Apr 14 10:24:01 hostname sshd[12346]: Failed password for root from 1.2.3.4 port 44820 ssh2
Apr 14 11:00:00 hostname sudo[5678]:   user : TTY=pts/0 ; PWD=/home/user ; USER=root ; COMMAND=/usr/bin/apt update
```

## Configuration cote source
Le fichier `/var/log/auth.log` est ecrit par PAM/sshd/sudo nativement.
Aucune configuration supplementaire n'est necessaire cote source.

Pour centraliser, Vector lit le fichier en mode `file` source.

## Variables
| Variable      | Default            | Description           |
|--------------|--------------------|-----------------------|
| AUTH_LOG_PATH | /var/log/auth.log | Chemin du fichier log |

## Mapping OCSF
| Event auth.log      | activity_name         | status  |
|---------------------|-----------------------|---------|
| Accepted password   | Logon                 | Success |
| Accepted publickey  | Logon                 | Success |
| Failed password     | Logon                 | Failure |
| Invalid user        | Logon                 | Failure |
| sudo command        | Privilege Escalation  | Success |

## Liens
- [OpenSSH logging](https://man.openbsd.org/sshd.8)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
