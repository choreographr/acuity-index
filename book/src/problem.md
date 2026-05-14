# Problem

Dapps need to read and write blockchain state, consume events, and often work
with off-chain content such as IPFS. Current state can be read on-chain or
off-chain, but historic state and events are usually consumed off-chain.

| Data | Read | Write | Modify |
|---|---|---|---|
| Current state | on/off-chain | on-chain | on-chain |
| Historic state | off-chain | — | — |
| Events | off-chain | on-chain | — |
| IPFS | off-chain | off-chain | — |

Writing data to events is often much cheaper than writing it to state, but that
creates a retrieval problem for dapps: the chain emits the data, while users and
applications still need a fast, trustworthy way to search and subscribe to it.

## Why Plain RPC Is Not Enough

Typical dapps depend on centralized RPC providers. That creates several issues:

### Incorrect Or Missing Data

A dapp generally trusts the RPC provider to return complete and correct results.
A malicious or faulty provider can omit data or return misleading answers.

### Event Query Limits

Substrate RPC does not provide rich event search. Other ecosystems often impose
limits such as:

- maximum blocks scanned per query
- earliest scannable block
- maximum execution time
- indexing disabled on free tiers

### Slow Queries

Scanning logs repeatedly is slower and more resource-intensive than querying a
purpose-built index.

### Unavailability And Policy Risk

Centralized providers can be unavailable due to outages, geoblocking, payment
requirements, or policy decisions.

### Tracking And Privacy Loss

Providers may correlate IP addresses, identities, and query history.

### Pressure Toward Centralized Dapp Backends

Because raw RPC is a poor fit for search, teams are incentivized to build their
own centralized indexing backend, which weakens the decentralization story.

### More Expensive Chain Design

If developers cannot reliably search events, they may push more data into chain
state just to make retrieval easier, increasing fees and block-space pressure.

### Limited Extensibility

Chains and dapps may need richer indexing beyond events, including full-text,
classification, or domain-specific search over linked off-chain data.

Acuity Index exists to solve the event indexing part of this problem with a
configurable, chain-aware, high-performance utility.
