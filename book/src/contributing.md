# Contributing

## Development Entry Points

The repository uses `just` as the main developer command surface:

```bash
just build
just test
just runtime-build
just synthetic-node
just seed-smoke
just test-integration
just benchmark-indexing
```

## Code Areas

High-level module boundaries:

- `src/main.rs`: CLI, startup, supervisor loop, long-lived tasks
- `src/indexer.rs`: indexing pipeline and span tracking
- `src/config.rs`: TOML schema and runtime mapping resolution
- `src/ws_api/`: public WebSocket API
- `src/protocol.rs`: shared protocol types and database key layout
- `src/runtime_state.rs`: live shared runtime state
- `src/metrics.rs`: Prometheus/OpenMetrics integration
- `src/config_gen.rs`: starter index-spec generation from metadata
- `src/synthetic_devnet.rs`: synthetic local chain support
- `runtime/`: in-repo synthetic runtime used for integration testing and benchmarking

## Suggested Workflow

1. run unit tests with `just test`
2. use the synthetic devnet when your change affects observable runtime behavior
3. run ignored integration tests when changing indexing, API, or reconnect logic
4. benchmark with the synthetic harness when changing throughput-sensitive code
5. update the book when user-facing or operator-facing behavior changes

## Testing Philosophy

Unit tests must be deterministic and must not rely on wall-clock timing.
Behavior that depends on real node timing or networking belongs in integration
tests instead.
