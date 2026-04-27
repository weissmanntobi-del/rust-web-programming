# Chapter 12 - Production Deployment Checklist

Covers:

- Environment-based config
- Fail-fast validation
- `/health`, `/livez`, `/readyz`
- Graceful shutdown
- Dockerfile
- GitHub Actions
- Kubernetes manifest starter

Run locally:

```bash
export DATABASE_URL=postgres://example
export JWT_SECRET=change-me-with-32-plus-random-bytes
cargo run
```

Build image:

```bash
docker build -t tasktracker:local .
docker run --rm -p 3000:3000 \
  -e HTTP_ADDR=0.0.0.0:3000 \
  -e DATABASE_URL=postgres://example \
  -e JWT_SECRET=change-me-with-32-plus-random-bytes \
  tasktracker:local
```
