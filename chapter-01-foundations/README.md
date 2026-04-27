# Chapter 1 - Rust Web Foundations

Covers:

- Tokio async entry point
- Minimal Axum server
- Health endpoint
- Hello endpoint
- Version endpoint using environment variables

Run:

```bash
cargo run
```

Test:

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/hello
curl http://127.0.0.1:3000/version
```
