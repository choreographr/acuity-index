# Features

This page combines the historical website feature list with the current project
behavior.

## User-Facing Features

- Config-driven indexing with support for scalar, composite, and multi-value keys
- Explicit pallet and event mappings in TOML
- Index specification checkpoints via `spec_change_blocks` for controlled reindexing
- Hot reload for accepted index-spec changes
- Concurrent backfill and live head catch-up
- WebSocket queries and subscriptions
- Event variant indexing via `index_variant`
- Optional finalized event proofs for `GetEvents`
- Prometheus / OpenMetrics metrics export
- Resume-safe persistence of indexed spans
- Synthetic devnet, integration tests, and end-to-end benchmarking

## Operational Features

- chain identity protection through persisted `genesis_hash`
- recoverable RPC reconnect loop with exponential backoff
- public WebSocket server stays up across accepted spec reloads and transient upstream failures
- configurable connection and subscription limits
- bounded subscription control paths and backpressure handling

## Performance Notes

The earlier public documentation highlighted throughput around `63.1k` events per
second on one benchmark machine. Treat that as historical context, not a
universal guarantee. For current measurement in this repository, use the
[Synthetic Devnet](./synthetic-devnet.md) and [Benchmarking](./benchmarking.md)
workflows.
