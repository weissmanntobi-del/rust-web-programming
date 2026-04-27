# Chapter 9 - Real-time APIs: WebSockets and SSE

Covers:

- WebSocket upgrade
- Server-Sent Events
- Broadcast channel fan-out
- Small event payloads

Run:

```bash
cargo run
```

Test SSE:

```bash
curl -N http://127.0.0.1:3000/events
```

In another terminal:

```bash
curl -X POST http://127.0.0.1:3000/demo/task-created
```

Test WebSocket with websocat:

```bash
websocat ws://127.0.0.1:3000/ws
```
