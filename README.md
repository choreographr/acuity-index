# acuity-index

`acuity-index` is a configurable event indexer for Substrate-based blockchains.
It is primarily intended for dapps to query directly as an event indexer,
although it can serve other consumers as well. It connects to a node over
WebSocket RPC, decodes on-chain events without chain-specific generated types,
stores indexed references in an embedded [`sled`](https://github.com/spacejam/sled)
database, and exposes the indexed data through its own WebSocket API.

This repository is primarily a Rust CLI application.

Documentation is available at <https://acuity-network.github.io/acuity-index/>.

Additional project documents:

- architecture notes: [`ARCHITECTURE.md`](./ARCHITECTURE.md)
- changelog: [`CHANGELOG.md`](./CHANGELOG.md)


## Features

- Config-driven indexing with TOML index specifications
- Schema-less event decoding for Substrate runtimes
- Resumable indexing with persisted block-span tracking
- WebSocket API for dapp queries and subscriptions
- Optional finalized-mode proofs for indexed events, including GRANDPA proofs for light-client verification
- Hot reload of the active index specification file
- Concurrent block fetching for backfill and head catch-up

## Requirements

- Rust stable (see [`rust-toolchain.toml`](./rust-toolchain.toml))
- A running Substrate node with WebSocket RPC enabled
- Historical state available via `--state-pruning archive-canonical`

## Quick start

Generate a starter index specification from a live node:

```bash
acuity-index generate-index-spec ./mychain.toml --url ws://127.0.0.1:9944
```

Run the indexer with that spec:

```bash
acuity-index run ./mychain.toml --url ws://127.0.0.1:9944
```

By default, the WebSocket API listens on port `8172`.

To remove the local index for a spec:

```bash
acuity-index purge-index ./mychain.toml
```

For a fuller walkthrough, see the online documentation.

## Index specification example

Each chain is described by an index specification TOML file passed as
`<INDEX_SPEC>`.

```toml
name = "mychain"
genesis_hash = "abc123..."
default_url = "wss://my-node:443"
index_variant = false
spec_change_blocks = [0]

[keys]
account_id = "bytes32"
item_id = "bytes32"
revision_id = "u32"
item_revision = { fields = ["bytes32", "u32"] }

[[pallets]]
name = "MyPallet"

[[pallets.events]]
name = "SomeEvent"

[[pallets.events.params]]
field = "who"
key = "account_id"

[[pallets.events.params]]
field = "item_id"
key = "item_id"

[[pallets.events.params]]
fields = ["item_id", "revision_id"]
key = "item_revision"
```

For the full index specification format and semantics, see the online documentation.

## License

Licensed under Apache-2.0.