# Chapter 4 - Validation and Error Handling

Covers:

- DTO vs domain types
- `TaskTitle` newtype
- Field-level validation errors
- Domain error to API error mapping

Run tests:

```bash
cargo test
```

Run server:

```bash
cargo run
```

Test invalid input:

```bash
curl -i -X POST http://127.0.0.1:3000/tasks -H "content-type: application/json" -d '{"title":"   "}'
```
