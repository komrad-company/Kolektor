# Kubernetes Audit Log

## Description
Collects `kube-apiserver` audit events (JSON, `audit.k8s.io/v1` API).
Normalises to OCSF class 6003 (API Activity).

## Expected format
One JSON line per event, typical `ResponseComplete` stage:
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

## Source-side configuration

### kube-apiserver flags
```
--audit-policy-file=/etc/kubernetes/audit-policy.yaml
--audit-log-path=/var/log/kubernetes/audit.log
--audit-log-maxage=30
--audit-log-maxbackup=10
--audit-log-maxsize=200
```

### Minimal audit policy (`/etc/kubernetes/audit-policy.yaml`)
```yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
  # Do not log health/metrics requests
  - level: None
    nonResourceURLs: ["/healthz*", "/readyz*", "/livez*", "/metrics"]
  # Auth / RBAC events in detail
  - level: RequestResponse
    resources:
      - group: rbac.authorization.k8s.io
        resources: ["roles", "rolebindings", "clusterroles", "clusterrolebindings"]
  # Pods / secrets / configmaps at Metadata level
  - level: Metadata
    resources:
      - group: ""
        resources: ["pods", "secrets", "configmaps", "serviceaccounts"]
  # Default: Metadata
  - level: Metadata
```

### File collection
- **K3s**: audit written to `/var/lib/rancher/k3s/server/logs/audit.log`
- **kubeadm**: `/var/log/kubernetes/audit.log` (via hostPath)
- The Kolektor parser reads via the `file` source: mount the path on the Kolektor pod or ship it agent-side via Fluent Bit / Vector.

## Variables
| Variable         | Default                             | Description                     |
|------------------|-------------------------------------|---------------------------------|
| `K8S_AUDIT_LOG`  | `/var/log/kubernetes/audit.log`     | Audit log file path             |
| `TENANT_ID`      | -                                   | Injected at runtime             |
| `DATASOURCE_ID`  | -                                   | Injected at runtime             |
| `QUICKWIT_ENDPOINT` | -                                | Injected at runtime             |

## OCSF mapping
| K8s audit field                  | OCSF field                           |
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
- other → 99 (Other)

## Links
- [K8s Auditing docs](https://kubernetes.io/docs/tasks/debug/debug-cluster/audit/)
- [OCSF 6003 API Activity](https://schema.ocsf.io/classes/api_activity)
