# Solution

Acuity Index runs alongside a Substrate node and builds queryable indexes over
runtime events. Instead of forcing every dapp to rescan chain history, the
indexer stores event references keyed by explicit query keys such as account IDs,
object IDs, or composite identifiers.

## Core Idea

At its simplest, Acuity Index maintains an index of `(block_number, event_index)`
for each configured key. Clients can then:

- query matching events efficiently
- subscribe to live updates for a key
- optionally request decoded event payloads
- optionally request finalized proof material for returned blocks

## Why This Helps

### High Performance

Acuity Index uses `sled` to maintain an embedded local index. That is better
suited to repeated key-based queries than ad hoc log scanning.

### Config-Driven Operation

Per-chain indexing rules live in TOML, not generated Rust code. Operators can
adapt behavior by editing index specs instead of recompiling the binary.

### Well-Defined Query Surface

Every index node exposes the same public WebSocket API for a given binary
version, making it easier for clients to switch providers.

### Verification-Friendly Responses

When the indexer runs in finalized mode, `GetEvents` can include finalized block
headers and `System.Events` storage proofs so clients can verify returned event
material against the chain state root.

### Extensible Design

The event index is the current focus, but the architecture leaves room for
richer indexing layers in the future.
