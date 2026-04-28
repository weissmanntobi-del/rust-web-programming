## Rust Complete Material: https://tobiweissmann.gumroad.com/l/gnuvxu
# Rust Web Programming - Chapter-wise Code Pack

This repository contains chapter-wise Rust code examples for the **Rust Web Programming MiniBook**.

The examples follow the book's TaskTracker journey:

1. Rust web foundations
2. HTTP essentials and routing
3. Building APIs with Axum
4. Validation and error handling
5. Middleware and security basics
6. Persistence with SQLx and Postgres
7. Authentication with JWT
8. Redis caching and background work
9. Real-time APIs with WebSockets and SSE
10. Frontend integration
11. Testing and quality
12. Production deployment checklist

## Prerequisites

- Rust stable
- Cargo
- Docker and Docker Compose for the Postgres and Redis chapters

## Run one chapter

```bash
cd chapter-03-axum-api
cargo run
```

Then test:

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/tasks
```

## Check all chapters

```bash
./scripts/check-all.sh
```

## Notes

- Chapters 1-5 and 9-12 are designed to run without external services.
- Chapter 6 needs Postgres.
- Chapter 8 can run with or without Redis. If Redis is unavailable, the example falls back gracefully.
- These examples are intentionally small and teaching-focused. For a paid production starter kit, combine the patterns into one cohesive app with stricter config, observability, migrations, CI/CD, and deployment docs.
