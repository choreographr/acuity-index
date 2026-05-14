# Overview

Acuity Index is a configurable event indexer for Substrate-based blockchains.
It connects to a node over WebSocket RPC, decodes runtime events, stores
queryable index entries in a local `sled` database, and exposes the indexed data
through its own WebSocket API.

The project is intentionally config-driven:

- chain-specific indexing rules live in TOML instead of generated Rust types
- event payloads are decoded generically
- the on-disk index is built around explicit query keys
- operators can update accepted index specs without restarting the public service

## Who This Book Is For

This book serves three overlapping audiences:

- operators running `acuity-index` against a live chain
- application developers integrating with the WebSocket API
- contributors working on the Rust codebase, synthetic devnet, and benchmarks

## What To Read First

- Start with [Installation](./installation.md) and [Quickstart](./quickstart.md) to run the binary.
- Read [Configuration](./configuration.md) to define an index specification.
- Read [CLI Reference](./cli.md) and [Operations](./operations.md) for day-2 usage.
- Read [WebSocket API](./api.md) to integrate clients.
- Read [Architecture](./architecture.md) and [Contributing](./contributing.md) to work on the codebase.

## Project Context

The older website documentation under `index.acuity.network/content/docs` has
been folded into this book. In particular, the [Problem](./problem.md),
[Solution](./solution.md), [Features](./features.md), and refreshed getting
started sections now live here alongside the operator and contributor guides.
