# Architecture

Acuity Index is a config-driven event indexer for Substrate chains. It decodes
runtime events with `subxt`, derives query keys from TOML config, stores index
entries in `sled`, and serves query access over WebSocket.

## Main Components

- `src/main.rs`: CLI, config loading, database initialization, watcher setup, metrics listener, and reconnect supervisor loop
- `src/indexer.rs`: indexing pipeline, resume logic, live-head tailing, key derivation, notification fanout
- `src/config.rs`: index-spec and options-config parsing plus validation
- `src/ws_api/`: public API implementation and connection lifecycle
- `src/protocol.rs`: wire types, key encodings, tree layout, and WebSocket runtime limits
- `src/runtime_state.rs`: shared live process state across long-lived tasks
- `src/event_hydration.rs`: decoded-event hydration plus finalized `System.Events` proof fetching
- `src/config_gen.rs`: live metadata to starter spec generation
- `src/metrics.rs`: metrics registry and HTTP export
- `src/synthetic_devnet.rs`: synthetic local chain helpers and shared test types

## Startup Sequence

The normal startup path is:

1. parse CLI args
2. load the required index spec
3. validate config and resolve runtime options
4. open `sled`
5. verify or initialize `genesis_hash`
6. start long-lived tasks such as WebSocket serving and spec watching
7. enter the RPC reconnect and indexing supervisor loop

## Data Model

The `sled` database is organized into trees opened by `Trees::open`:

- `root`: database-level values such as `genesis_hash`
- `span`: indexed block spans for resume and reindex logic
- `variant`: event references keyed by pallet and variant indices
- `index`: custom and built-in query keys with `(block_number, event_index)` suffixes
- `events`: decoded event JSON keyed by `(block_number, event_index)`

## Indexing Flow

High-level behavior:

- determine the starting head block
- load previously indexed spans
- resume an existing tail span or index the current head immediately
- run backward backfill and live-head tracking concurrently

If finalized mode is enabled, the same finalized-only setting governs whether
API callers can receive proof material for `GetEvents`.

## Concurrency Model

The indexer uses async queues for both descending backfill and ascending live
head processing. `queue_depth` applies to both so the process can catch up on
fast-moving chains instead of advancing one block at a time.

Because futures can complete out of order, the implementation keeps orphan maps
until contiguous spans can be extended safely.

## Invariants

- a database path belongs to exactly one chain identity
- accepted spec reloads should not take down the public service
- malformed persisted data should be handled defensively
- the synthetic test path should stay close to production architecture
- public API correctness is validated end to end, not only through unit tests
