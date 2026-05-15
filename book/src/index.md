# Overview

Acuity Index is a configurable event indexer for Substrate-based blockchains.
It is primarily intended for dapps to query directly as an event indexer,
although other consumers can use it too. It connects to a node over WebSocket
RPC, decodes runtime events, stores queryable index entries in a local `sled`
database, and exposes the indexed data through its own WebSockets API.

The project is intentionally config-driven:

- chain-specific indexing rules live in TOML instead of generated Rust types
- event payloads are decoded generically
- the on-disk index is built around explicit query keys
- returned events can include GRANDPA proofs so light clients can verify correctness
- operators can update accepted index specs without restarting the public service

## Funding

Acuity Index was originally called Hybrid and was funded by two ([1](https://github.com/w3f/Grants-Program/blob/master/applications/hybrid.md), [2](https://github.com/w3f/Grants-Program/blob/master/applications/hybrid2.md)) Web3 Foundation grants and a Kusama Treasury [referendum](https://kusama.subsquare.io/referenda/534). A second funding referendum [failed](https://kusama.subsquare.io/referenda/567). Treasury funding for a Kusama Forum based on Acuity Index has been [secured](https://kusama.subsquare.io/referenda/603).

## Who This Book Is For

This book serves three overlapping audiences:

- operators running `acuity-index` against a live chain
- application developers integrating with the WebSockets API
- contributors working on the Rust codebase and benchmarks

