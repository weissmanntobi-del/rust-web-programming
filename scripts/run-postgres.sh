#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../chapter-06-sqlx-postgres"
docker compose up -d postgres
export DATABASE_URL=postgres://tasktracker:tasktracker@localhost:5432/tasktracker
cargo run
