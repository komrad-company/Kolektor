# Kubernetes Audit Log

## Description
Collecte les audit events du `kube-apiserver` (JSON, API `audit.k8s.io/v1`).
Normalise en OCSF classe 6003 (API Activity).

## Format attendu
Une ligne JSON par event, stage `ResponseComplete` typique :
```json
{
  "kind": "Event",
  "apiVersion": "audit.k8s.io/v1",
  "level": "Metadata",
  "auditID": "abc-123-def",
  "stage": "ResponseComplete",
  "requestURI": "/api/v1/namespaces/default/pods/mypod",
  "verb": "get",
  "user": { "username": "kube-admin", "groups": ["system:masters"] },
  "sourceIPs": ["10.0.0.5"],
  "userAgent": "kubectl/v1.28.0",
  "objectRef": {
    "resource": "pods",
    "namespace": "default",
    "name": "mypod",
    "apiVersion": "v1"
  },
  "responseStatus": { "code": 200 },
  "requestReceivedTimestamp": "2024-04-14T10:23:45.123456Z",
  "stageTimestamp": "2024-04-14T10:23:45.234567Z",
  "annotations": {
    "authorization.k8s.io/decision": "allow"
  }
}
```

## Configuration cote source

### kube-apiserver flags
```
--audit-policy-file=/etc/kubernetes/audit-policy.yaml
--audit-log-path=/var/log/kubernetes/audit.log
--audit-log-maxage=30
--audit-log-maxbackup=10
--audit-log-maxsize=200
```

### Audit policy minimale (`/etc/kubernetes/audit-policy.yaml`)
```yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
  # Ne pas logger les requetes health/metrics
  - level: None
    nonResourceURLs: ["/healthz*", "/readyz*", "/livez*", "/metrics"]
  # Events auth / RBAC en detail
  - level: RequestResponse
    resources:
      - group: rbac.authorization.k8s.io
        resources: ["roles", "rolebindings", "clusterroles", "clusterrolebindings"]
  # Pods / secrets / configmaps en Metadata
  - level: Metadata
    resources:
      - group: ""
        resources: ["pods", "secrets", "configmaps", "serviceaccounts"]
  # Defaut : Metadata
  - level: Metadata
```

### Collecte du fichier
- **K3s** : audit ecrit dans `/var/lib/rancher/k3s/server/logs/audit.log`
- **kubeadm** : `/var/log/kubernetes/audit.log` (via hostPath)
- Le parser Kolektor lit via `file` source : monter le chemin sur le pod Kolektor ou expedier via Fluent Bit / Vector agent side.

## Variables
| Variable         | Default                             | Description                     |
|------------------|-------------------------------------|---------------------------------|
| `K8S_AUDIT_LOG`  | `/var/log/kubernetes/audit.log`     | Chemin du fichier audit log     |
| `TENANT_ID`      | -                                   | Injecte runtime                 |
| `DATASOURCE_ID`  | -                                   | Injecte runtime                 |
| `QUICKWIT_ENDPOINT` | -                                | Injecte runtime                 |

## Mapping OCSF
| Champ K8s audit                  | Champ OCSF                           |
|----------------------------------|--------------------------------------|
| `verb`                           | `api.operation`, `activity_id`       |
| `requestURI`                     | `api.request.uri`                    |
| `auditID`                        | `api.request.uid`, `metadata.uid`    |
| `user.username`                  | `actor.user.name`                    |
| `user.uid`                       | `actor.user.uid`                     |
| `sourceIPs[0]`                   | `src_endpoint.ip`                    |
| `userAgent`                      | `http_request.user_agent`            |
| `objectRef.resource`             | `resources[0].type`                  |
| `objectRef.namespace`            | `resources[0].namespace`             |
| `objectRef.name`                 | `resources[0].name`                  |
| `responseStatus.code`            | `api.response.code`, `severity_id`   |
| `annotations.authorization.k8s.io/decision` | `unmapped.authorization_decision` |
| `stageTimestamp`                 | `time`                               |

### activity_id
- `create`/`post` → 1 (Create)
- `get`/`list`/`watch` → 2 (Read)
- `update`/`patch`/`put` → 3 (Update)
- `delete`/`deletecollection` → 4 (Delete)
- autre → 99 (Other)

## Liens
- [K8s Auditing docs](https://kubernetes.io/docs/tasks/debug/debug-cluster/audit/)
- [OCSF 6003 API Activity](https://schema.ocsf.io/classes/api_activity)
