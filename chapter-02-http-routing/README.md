# Chapter 2 - HTTP Essentials and Routing

Covers:

- HTTP status codes
- Path params: `/tasks/{id}`
- Query params: `/tasks?limit=20`
- Request ID headers
- Optional success envelope

Run:

```bash
cargo run
```

Test:

```bash
curl http://127.0.0.1:3000/tasks
curl "http://127.0.0.1:3000/tasks?limit=500"
curl http://127.0.0.1:3000/tasks/task_123 -H "x-request-id: demo-123"
```
