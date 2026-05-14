#!/usr/bin/env bash
set -euo pipefail

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> cargo test"
cargo test

echo "==> just test-integration"
just test-integration
