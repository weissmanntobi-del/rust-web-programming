#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../chapter-08-redis-background"
docker compose up -d redis
export REDIS_URL=redis://127.0.0.1:6379
cargo run
