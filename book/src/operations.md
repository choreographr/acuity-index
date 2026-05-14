# Operations

## Running The Service

The main command is:

```bash
acuity-index run <INDEX_SPEC> [OPTIONS]
```

Common runtime flags:

- `--options-config <PATH>`
- `--db-path <PATH>`
- `--db-mode <MODE>`
- `--db-cache-capacity <SIZE>`
- `--url <URL>`
- `--queue-depth <N>`
- `--finalized`
- `--port <PORT>`
- `--metrics-port <PORT>`
- WebSocket capacity and timeout flags

## Startup Behavior

At startup the process:

1. loads and validates the index spec
2. resolves runtime options
3. opens `sled`
4. verifies or initializes the stored `genesis_hash`
5. starts the public WebSocket server and optional metrics listener
6. enters a reconnecting RPC/indexer supervisor loop

One database directory belongs to one chain genesis hash. If the stored hash and
connected chain do not match, startup fails instead of mixing data.

## Reconnect And Resume

The indexer persists span state so it can resume after restart. On transient RPC
failure it keeps the process alive, saves the active span, reconnects with
exponential backoff, and resumes without a clean full restart.

During that window:

- sled-backed local reads can continue to work
- existing clients stay connected
- RPC-backed requests such as `Variants` and `GetEvents` return temporary unavailability

## Hot Reload

When the active index spec file changes, accepted edits restart only the
RPC/indexer loop. The WebSocket and metrics servers remain up.

Changes to `name` or `genesis_hash` are rejected.

If an options config is used, accepted changes there can also update runtime
behavior. Some values are applied live, while others are restart-gated.

## Historical State Requirement

`acuity-index` needs archival historical state so it can query old blocks with
`api.at_block(...)`. If the upstream node prunes state, indexing fails with an
explicit error explaining that the node must use `--state-pruning archive-canonical`.

## Purging An Index

To delete the local index database for a spec:

```bash
acuity-index purge-index ./mychain.toml
```
