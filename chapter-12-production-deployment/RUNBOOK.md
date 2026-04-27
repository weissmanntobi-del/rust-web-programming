# TaskTracker Runbook

## Deploy

1. Build and push the image.
2. Apply secrets.
3. Deploy Kubernetes manifests.
4. Check rollout status.

```bash
kubectl rollout status deploy/tasktracker
```

## Rollback

```bash
kubectl rollout undo deploy/tasktracker
```

## Debug

```bash
kubectl logs deploy/tasktracker
kubectl describe pod -l app=tasktracker
kubectl get events --sort-by=.lastTimestamp
```

## First checks during incident

- Is `/livez` passing?
- Is `/readyz` passing?
- Are DB connections exhausted?
- Did the last deployment change env vars or secrets?
- Are request timeouts increasing?
