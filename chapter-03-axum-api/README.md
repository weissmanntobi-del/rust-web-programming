# Chapter 3 - Building APIs with Axum

Covers:

- Feature-style route organization
- Shared `AppState`
- Thin handlers
- Structured errors with `IntoResponse`
- In-memory TaskTracker API

Run:

```bash
cargo run
```

Test:

```bash
curl http://127.0.0.1:3000/health
curl -X POST http://127.0.0.1:3000/tasks -H "content-type: application/json" -d '{"title":"Write Rust API"}'
curl http://127.0.0.1:3000/tasks
```
