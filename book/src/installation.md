# Installation

## Requirements

- Rust `stable` as pinned by `rust-toolchain.toml`
- a Substrate node with WebSocket RPC enabled
- archival historical state on that node via `--state-pruning archive-canonical`
- `polkadot-omni-node` if you want to use the in-repo synthetic devnet workflows
- `just` for the documented developer command surface

## Install The Binary

From this repository:

```bash
cargo install --path .
```

For local development you can also run it directly without installing:

```bash
cargo run -- run ./mychain.toml
```

## Build From Source

Common entry points:

```bash
just build
just test
just build-release
```

## Install Documentation Tooling

If you want to build this book locally, install `mdbook`:

```bash
cargo install mdbook
```

Then use:

```bash
just book-build
just book-serve
```
