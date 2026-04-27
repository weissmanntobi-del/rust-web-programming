# Chapter 8 - Redis Caching and Background Work

Covers:

- Cache-aside pattern
- Versioned cache keys
- Soft-failure behavior when Redis is unavailable
- Background audit job with Tokio task

Start Redis:

```bash
docker compose up -d redis
```

Run:

```bash
export REDIS_URL=redis://127.0.0.1:6379
cargo run
```

Test:

```bash
curl http://127.0.0.1:3000/tasks
curl -X POST http://127.0.0.1:3000/tasks -H "content-type: application/json" -d '{"title":"cache aside"}'
curl http://127.0.0.1:3000/tasks
```
