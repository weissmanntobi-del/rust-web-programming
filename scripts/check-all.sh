#!/usr/bin/env bash
set -euo pipefail

for dir in chapter-*; do
  if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
    echo "Checking $dir"
    (cd "$dir" && cargo check)
  fi
done

echo "All chapters passed cargo check."
