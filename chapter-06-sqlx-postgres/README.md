# Chapter 6 - Persistence with SQLx and Postgres

Covers:

- Postgres connection pool
- Migrations
- Repository-style data access
- `fetch_optional` for 404 mapping
- Transactions for atomic updates

Start Postgres:

```bash
docker compose up -d postgres
```

Run:

```bash
export DATABASE_URL=postgres://tasktracker:tasktracker@localhost:5432/tasktracker
export RUN_MIGRATIONS=true
cargo run
```

Test:

```bash
USER_ID=$(uuidgen)
# Insert a user first, or create one manually in psql for this chapter.
curl http://127.0.0.1:3000/health
```

This chapter focuses on SQLx patterns. The auth chapter adds user registration/login.
