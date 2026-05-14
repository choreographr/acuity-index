## Overview

`acuity-index` is a config-driven event indexer for Substrate chains. It connects to a node over WebSocket RPC, decodes runtime events with `subxt`, derives query keys from TOML config, stores index entries in `sled`, and exposes read/query access over its own WebSocket API.

Design goals:

- keep chain-specific behavior in TOML
- decode events generically with `scale_value`, not generated Rust types
- store queryable data in compact binary keys
- keep the document focused on invariants that help AI agents change the code safely

## Main Components

- `src/main.rs` — CLI startup, config loading, sled setup, genesis-hash checks, long-lived task orchestration, watched-spec reloads, and reconnect/supervisor logic.
- `src/indexer.rs` — indexing pipeline, span tracking, resume logic, live-head/backfill catchup, key derivation, and subscriber fanout.
- `src/websockets.rs` — public WebSocket API, request/response handling, connection lifecycle, and event proof responses.
- `src/shared.rs` — wire types, sled tree layout, shared runtime state, and common enums.
- `src/config.rs` — TOML schema and runtime key-mapping rules.
- `src/event_hydration.rs` — hydrates indexed refs into API responses and fetches finalized proofs when requested.
- `src/config_gen.rs` — starter config generation from live runtime metadata.
- `src/metrics.rs` — optional Prometheus metrics endpoint.
- `src/synthetic_devnet.rs` — helpers for the local synthetic runtime, config rendering, readiness checks, and API probes.
- `src/bin/seed_synthetic_runtime.rs` — deterministic seeder for synthetic tests and benchmarks.
- `src/bin/benchmark_synthetic_indexing.rs` — end-to-end benchmark harness for the synthetic stack.
- `runtime/` — in-repo Polkadot SDK runtime workspace used for local devnet testing.
- `tests/common/mod.rs` — orchestration helpers for the synthetic integration suite.
- `tests/synthetic_integration.rs` — ignored external integration test covering the full synthetic flow.

## Core Invariants

- A database directory is tied to a single chain genesis hash.
- Chain-specific indexing behavior should live in TOML, not hardcoded Rust.
- `Key::Custom` is the main path for declared queryable keys.
- Event payloads are decoded schema-less; generated Rust event types are not required.
- `acuity_indexStatus` is local and continues working while upstream RPC is unavailable.
- `acuity_getEventMetadata` and `acuity_getEvents` require a live RPC handle.
- The WebSocket server does not own indexing state; it reads from sled and shared subscriber state.

## Data Model

The sled layout is opened in `Trees::open` in `src/shared.rs`.

- `root` — root keys like `genesis_hash`
- `span` — indexed block spans for resume/reindex logic
- `variant` — variant index records keyed by `(pallet_index, variant_index, block_number, event_index)`
- `index` — custom and built-in query keys with a `(block_number, event_index)` suffix

Span values record which block ranges are indexed for a specific index-spec revision. `IndexSpec.spec_change_blocks` controls when spans become stale and need reindexing.

## Indexing Flow

`run_indexer` in `src/indexer.rs` does the main work.

1. Choose the starting head (`finalized` head or best head).
2. Load spans from sled.
3. Resume an existing span or index the current head.
4. Run backfill toward genesis while also tailing live heads.
5. Save the current span before returning on errors so the supervisor can resume safely.

Per block, the indexer:

- fetches the block hash
- creates a block-scoped `subxt` view
- reads decoded runtime events
- optionally writes a variant record
- decodes fields with `scale_value`
- derives configured keys
- writes event refs into sled

Important operational points:

- runtime decoding of persisted keys/values is defensive; malformed records are skipped with logging
- `Indexer::keys_for_event(...)` only uses explicit TOML mappings
- if historical state is unavailable, the indexer exits with `StatePruningMisconfigured`

## Concurrency and Catchup

The indexer uses one async loop with `tokio::select!` over:

- exit notifications
- new head notifications
- queued live-head futures
- queued backfill futures
- periodic stats logging

Two queues exist:

- backfill queue for descending historical blocks
- live-head queue for ascending new blocks

`queue_depth` controls both. Futures may complete out of order, so orphan maps hold early results until continuity is satisfied:

- `orphans` for backfill
- `head_orphans` for live head

This lets the indexer catch up quickly without assuming completion order.

## Resume and Supervisor Behavior

The supervisor loop in `src/main.rs` handles transient RPC failures and accepted spec reloads.

- Recoverable errors trigger reconnection with exponential backoff.
- Fatal errors exit the process.
- Accepted spec updates restart only the indexer, not the whole process.
- The current span is persisted before returning on error, so replay is idempotent.

Spec watcher behavior:

- watches the parent directory of the active `<INDEX_SPEC>`
- accepts in-place writes and rename-based saves
- validates the full updated spec before publishing it
- rejects updates that change `name` or `genesis_hash`

## Config Model

`IndexSpec` in `src/config.rs` defines the TOML schema.

Top-level fields include:

- `name`
- `genesis_hash`
- `default_url`
- `index_variant`
- `spec_change_blocks`
- `keys`
- `pallets`

Runtime options come from `--options-config` and CLI flags. Precedence is:

**CLI flags > `--options-config` > built-in defaults**

`finalized` uses OR logic, and `index_variant` only comes from `IndexSpec`.

## Event Key Extraction and Encoding

Field extraction helpers in `src/indexer.rs` work on generic `scale_value::Value` trees.

Supported key shapes include scalar and composite keys declared in TOML. Unknown key names fail validation early.

Decoded events are encoded to JSON with a stable outer shape. Notable choices:

- `u128`/`i128` become strings
- some byte-like composites become `0x...` hex strings
- hydrated events include `blockNumber`, `eventIndex`, `timestamp`, and `event`

`acuity_getEvents` reads refs from sled first, then hydrates event payloads from the node. For each returned block it also reads `Timestamp::Now`; if unavailable, timestamp falls back to `0`.

## WebSocket API

`src/websockets.rs` implements the public API.

Request paths:

- `acuity_indexStatus` — local sled-backed status
- `acuity_getEventMetadata` — RPC-backed metadata lookup
- `acuity_getEvents` — local refs + RPC hydration, with optional proofs

Subscriptions are handled by a bounded dispatcher task. The server enforces:

- connection caps
- per-connection and total subscription caps
- idle timeouts
- message/frame size limits
- key/value and composite size limits for custom queries

When limits are exceeded, requests are rejected with explicit errors rather than blocking unboundedly.

## Generated Configs

`src/config_gen.rs` builds starter TOML from live metadata.

It requires metadata v14 or higher and uses `subxt` metadata conversion. The generator is heuristic: it infers likely account-like or scalar fields, but it does not try to understand semantic aliases.

## Synthetic Devnet, Tests, and Benchmarking

The repo includes a self-contained synthetic stack for deterministic validation.

Flow:

1. `runtime/` builds a small local runtime with a custom `Synthetic` pallet.
2. `polkadot-omni-node` runs that runtime locally from a generated chain spec.
3. `src/synthetic_devnet.rs` renders a matching synthetic index config.
4. `src/bin/seed_synthetic_runtime.rs` submits deterministic transactions.
5. `acuity-index` indexes the chain normally.
6. Tests and benchmarks validate behavior through the public WebSocket API.

`tests/synthetic_integration.rs` is ignored because it depends on heavyweight local tooling, but it is the end-to-end validation path for the synthetic stack.

## Key Files for Common Changes

- startup, reconnection, reloads: `src/main.rs`
- indexing, spans, key derivation: `src/indexer.rs`
- config schema or runtime options: `src/config.rs`, `src/shared.rs`
- WebSocket request/response behavior: `src/websockets.rs`, `src/shared.rs`
- config generation heuristics: `src/config_gen.rs`
- synthetic validation flow: `src/synthetic_devnet.rs`, `tests/common/mod.rs`, `tests/synthetic_integration.rs`

## Mental Model

1. TOML says which fields matter.
2. The indexer turns matching event fields into binary index entries.
3. Spans record what has already been indexed.
4. The supervisor recreates RPC clients and the indexer task after transient failures.
5. The WebSocket server is a thin query and subscription layer over sled plus shared state.

If you are changing behavior, first decide which layer owns it:

- config/schema
- field extraction
- per-event indexing
- span/resume
- reconnection
- API/query
