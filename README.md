# acuity-index

`acuity-index` is a configurable event indexer for Substrate-based blockchains.
It is primarily intended for dapps to query directly as an event indexer,
although it can serve other consumers as well. It connects to a node over
WebSocket RPC, decodes on-chain events without chain-specific generated types,
stores indexed references in an embedded [`sled`](https://github.com/spacejam/sled)
database, and exposes the indexed data through its own WebSocket API.

This repository is primarily a Rust CLI application.

## Documentation

Primary documentation lives in the in-repo mdBook:

- overview: [`book/src/index.md`](./book/src/index.md)
- installation: [`book/src/installation.md`](./book/src/installation.md)
- quick start: [`book/src/quickstart.md`](./book/src/quickstart.md)
- CLI reference: [`book/src/cli.md`](./book/src/cli.md)
- configuration: [`book/src/configuration.md`](./book/src/configuration.md)
- WebSocket API: [`book/src/api.md`](./book/src/api.md)
- security: [`book/src/security.md`](./book/src/security.md)
- contributing: [`book/src/contributing.md`](./book/src/contributing.md)
- full table of contents: [`book/src/SUMMARY.md`](./book/src/SUMMARY.md)

Additional project documents:

- architecture notes: [`ARCHITECTURE.md`](./ARCHITECTURE.md)
- changelog: [`CHANGELOG.md`](./CHANGELOG.md)

Build or serve the book locally with:

```bash
just book-build
just book-serve
```

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

## Installation

### Build from source

```bash
cargo build --release
```

### Install the binary locally

```bash
cargo install --path .
```

For more setup details, see [`book/src/installation.md`](./book/src/installation.md).

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

For a fuller walkthrough, see [`book/src/quickstart.md`](./book/src/quickstart.md).

## Usage

```bash
acuity-index <COMMAND>
```

### Commands

| Command | Description |
|---|---|
| `run <INDEX_SPEC> [OPTIONS]` | Run the indexer for an index specification |
| `purge-index <INDEX_SPEC> [OPTIONS]` | Delete the index database for an index spec |
| `generate-index-spec <INDEX_SPEC> --url <URL> [--force|-f]` | Inspect live metadata and write a starter index specification |

For full CLI details, use:

```bash
acuity-index --help
acuity-index run --help
```

For complete command and runtime reference, see [`book/src/cli.md`](./book/src/cli.md),
[`book/src/configuration.md`](./book/src/configuration.md), and
[`book/src/api.md`](./book/src/api.md).

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

For the full index specification format and semantics, see
[`book/src/configuration.md`](./book/src/configuration.md).

## Development

Common development commands:

```bash
cargo build
cargo test
```

Using `just`:

```bash
just build
just test
just release-checks
```

Documentation:

```bash
# see the mdBook under book/src/
```

See [`book/src/contributing.md`](./book/src/contributing.md) for contributor-oriented notes.

## Synthetic devnet

The repository includes a small in-repo synthetic runtime under [`runtime/`](./runtime/)
for local integration testing and benchmarking.

Useful commands:

```bash
just synthetic-node
just seed-smoke
just test-integration
just benchmark-indexing
```

`polkadot-omni-node` is required for the synthetic runtime workflows.

## License

Licensed under Apache-2.0.