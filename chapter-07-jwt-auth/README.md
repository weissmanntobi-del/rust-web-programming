# Chapter 7 - Authentication with JWT

Covers:

- Password hashing with Argon2
- JWT claims, signing, expiry
- Protected route with `UserContext` extractor
- Generic login errors

Run:

```bash
export JWT_SECRET="dev-secret-change-me-dev-secret-change-me"
cargo run
```

Test:

```bash
curl -X POST http://127.0.0.1:3000/auth/register \
  -H "content-type: application/json" \
  -d '{"email":"demo@example.com","password":"password123"}'

TOKEN=$(curl -s -X POST http://127.0.0.1:3000/auth/login \
  -H "content-type: application/json" \
  -d '{"email":"demo@example.com","password":"password123"}' | jq -r .access_token)

curl http://127.0.0.1:3000/me -H "authorization: Bearer $TOKEN"
```
