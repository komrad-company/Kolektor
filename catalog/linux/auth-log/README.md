# Linux auth.log — SSH, sudo, PAM

## Description
Collects Linux authentication events from `/var/log/auth.log`.
Normalises to OCSF class 3002 (Authentication).

## Covered events
- SSH login accepted (password, publickey)
- SSH login failed (password, publickey, invalid user)
- sudo command execution
- PAM authentication events

## Expected format
```
Apr 14 10:23:45 hostname sshd[12345]: Accepted publickey for user from 1.2.3.4 port 54321 ssh2
Apr 14 10:24:01 hostname sshd[12346]: Failed password for root from 1.2.3.4 port 44820 ssh2
Apr 14 11:00:00 hostname sudo[5678]:   user : TTY=pts/0 ; PWD=/home/user ; USER=root ; COMMAND=/usr/bin/apt update
```

## Source-side configuration
The `/var/log/auth.log` file is written natively by PAM/sshd/sudo.
No additional source-side configuration is required.

To centralise, Vector reads the file via the `file` source.

## Variables
| Variable      | Default            | Description    |
|--------------|--------------------|----------------|
| AUTH_LOG_PATH | /var/log/auth.log | Log file path  |

## OCSF mapping
| Event auth.log      | activity_name         | status  |
|---------------------|-----------------------|---------|
| Accepted password   | Logon                 | Success |
| Accepted publickey  | Logon                 | Success |
| Failed password     | Logon                 | Failure |
| Invalid user        | Logon                 | Failure |
| sudo command        | Privilege Escalation  | Success |

## Links
- [OpenSSH logging](https://man.openbsd.org/sshd.8)
- [Vector file source](https://vector.dev/docs/reference/configuration/sources/file/)
