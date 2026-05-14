# Security And Deployment

This chapter summarizes the current deployment posture of the Internet-facing
surfaces in this repository, the hardening already implemented, and the main
remaining requirements before the indexer is exposed directly to dapp clients.

It is a companion to [Architecture](./architecture.md) and [WebSocket API](./api.md),
and should reflect the current code layout under `src/main.rs`,
`src/indexer.rs`, `src/event_hydration.rs`, `src/runtime_state.rs`,
`src/protocol.rs`, and `src/metrics.rs`.

## Scope

Reviewed components:

- `src/main.rs`
- `src/indexer.rs`
- `src/event_hydration.rs`
- `src/runtime_state.rs`
- `src/protocol.rs`
- `src/metrics.rs`
- dependency surface via `cargo audit`

Primary attack surface:

- public WebSocket listener on `0.0.0.0:<port>`
- JSON-RPC request parsing for:
  - `acuity_indexStatus`
  - `acuity_getEventMetadata`
  - `acuity_getEvents`
  - `acuity_subscribeStatus`
  - `acuity_unsubscribeStatus`
  - `acuity_subscribeEvents`
  - `acuity_unsubscribeEvents`
- subscription dispatch path between connection handlers and the indexer/runtime subscription registry
- optional OpenMetrics HTTP listener on `0.0.0.0:<metrics_port>`
- upstream node RPC trust boundary used for event hydration and proof retrieval

## Current Hardening

The following protections are implemented in the current server.

### Resource-exhaustion controls

- Bounded subscription control queue
  - The connection-to-dispatcher control path uses a bounded Tokio channel.
  - Saturation is treated as a disconnect/error path rather than unbounded buffering.

- Bounded per-subscriber notification queues
  - Each connection gets a bounded notification channel for subscription pushes.
  - Slow subscribers are removed instead of being buffered indefinitely.
  - The server sends a best-effort `terminated` notification with reason `backpressure` before removal.

- WebSocket frame and message size limits
  - max WebSocket message size: `256 KiB`
  - max WebSocket frame size: `64 KiB`
  - Oversized payloads are rejected during protocol handling.

- Bounded custom key input sizes
  - custom key name limit: `128` bytes
  - custom string value limit: `1024` bytes
  - composite custom values are limited to:
    - `64` elements
    - `8` nesting levels
    - `16384` encoded bytes
  - Invalid or oversized key payloads are rejected before subscription registration or index scans.

- Global connection cap
  - Concurrent WebSocket connections are capped at `1024` by default.
  - When the cap is exhausted, new upgrade attempts are rejected with HTTP `503 Service Unavailable`.

- Subscription caps
  - A single connection may hold at most `128` subscriptions by default.
  - There is also a global total subscription cap of `65536` by default across all connections.
  - Duplicate subscription attempts do not create extra server-side registrations.

- Query result cap
  - `acuity_getEvents` applies a configurable per-request limit clamp.
  - Default maximum events per query: `1000`.
  - Client-provided `limit` values are clamped into `1..=max_events_limit`.

- Idle timeout
  - Idle connections are closed after `300` seconds by default.
  - Setting `idle_timeout_secs = 0` disables this timeout.

- Heartbeat pings
  - The server sends WebSocket ping frames periodically.
  - The interval is bounded by the idle-timeout configuration (`idle_timeout_secs / 2`, capped at 120 seconds).
  - Incoming ping/pong traffic counts as connection activity.

### Crash-resistance and recovery improvements

- Recoverable upstream RPC failures no longer require a full process crash.
  - The supervisor loop reconnects with exponential backoff.
  - The current span is saved before the indexer task returns.
  - Existing WebSocket clients remain connected while local-only requests continue to work.

- RPC-backed requests degrade to temporary unavailability.
  - `acuity_getEventMetadata` and `acuity_getEvents` return JSON-RPC `-32001` with `data.reason = "temporarily_unavailable"` while the node RPC is down.
  - `acuity_indexStatus` remains available from local sled state.

- Malformed persisted records are handled defensively.
  - Malformed span values/keys are skipped with logging during reads.
  - Malformed persisted event index records are skipped with logging rather than crashing the process.

- Poisoned subscription/runtime locks are recovered.
  - Shared runtime mutexes use a recovery path that logs and continues rather than panicking immediately.

- Startup and reconnect behavior are explicit.
  - Genesis-hash mismatch remains a fail-fast startup/runtime error to prevent cross-chain data mixing.
  - State-pruning misconfiguration remains a fatal operator error.

### Exposure segmentation

- Metrics are served on a separate HTTP listener and port.
  - This keeps the observability surface distinct from the public WebSocket API.
  - It does not make the metrics endpoint safe for public exposure by itself.

## Residual Risks

The most important remaining deployment requirements are below.

### 1. Meter every query and require payment

Before direct public exposure, the indexer needs per-query metering and payment enforcement so the index provider can bill usage.

Impact:

- Every query should be counted and accounted for.
- Expensive request paths should be covered by the metering model.
- Payment checks should gate access before event hydration and proof retrieval are served.
- Direct dapp access should remain tied to the provider's billing policy.

Assessment:

- This is the primary remaining product requirement before treating the service as Internet-facing.

### 2. No in-process TLS

The service speaks plain WebSocket (`ws`) and plain HTTP for metrics.

Impact:

- Confidentiality and integrity depend on external deployment infrastructure.
- Direct exposure without TLS termination allows traffic observation and tampering in transit.
- The metrics endpoint has the same issue when exposed directly.

Assessment:

- Safe deployment requires TLS termination and normal edge protections outside the process.

### 3. Public expensive endpoints remain available

`acuity_getEventMetadata`, `acuity_getEvents`, event hydration, proof retrieval, and live subscriptions are still publicly callable.

Impact:

- Attackers can still consume CPU, RPC bandwidth, storage I/O, and upstream node capacity within the configured limits.
- Hydrated event reads are not purely local; they depend on live node access.
- Finalized proof inclusion adds extra upstream work.
- Current caps reduce per-connection blast radius but do not provide fairness across many clients or many source IPs.

Assessment:

- The server is substantially more resilient than an unbounded implementation.
- It is still vulnerable to sustained abuse, especially distributed abuse, because there is no built-in rate limiting or admission control by identity/network.

### 4. Metrics endpoint can leak operational metadata

If enabled, the separate OpenMetrics endpoint exposes operational state such as:

- RPC connectivity
- reconnect counts
- current indexed span
- latest seen head
- WebSocket connection count
- status and event subscription counts
- database size
- block fetch/process/commit timing histograms

Impact:

- Anything that can reach the metrics port can observe internal health and capacity signals.
- These signals may help attackers tune abuse or infer operational state.

Assessment:

- Lower severity than unauthenticated data access, but still an important exposure.
- The metrics port should be treated as an internal observability interface, not part of the public API.

### 5. Dependency maintenance risk remains material

The project still depends on some crates with maintenance or ecosystem advisories.

Impact:

- Even without a confirmed application-level exploit, these dependencies increase long-term supply-chain and maintenance risk.
- Some affected crates are in production dependency paths; others land through dev/test or optional light-client-related paths.

Assessment:

- This is primarily a patch-management and dependency-upgrade concern.
- The `sled` dependency remains a notable long-term risk because multiple advisories continue to land through it.

### 6. Some fail-fast paths remain intentional

The production runtime has moved many remotely reachable failure paths away from `unwrap()`-style crashes, but some startup-only or invariant-enforcement failures still intentionally stop the process.

Impact:

- Misconfiguration such as wrong genesis hash or pruned historical state still results in process exit.
- This is mainly an operability/reliability concern rather than a direct remote-exploitation path.

Assessment:

- Reasonable for invariant protection.
- Operators should still treat startup config validation and deployment checks as part of the security boundary.

## Cargo Audit

`cargo audit -q` reported the following advisories during review:

1. `RUSTSEC-2024-0388`
   - dependency chain includes `derivative 2.2.0`
   - status: unmaintained

2. `RUSTSEC-2025-0057`
   - dependency chain: `sled -> fxhash 0.2.1`
   - status: unmaintained

3. `RUSTSEC-2024-0384`
   - dependency chain: `sled -> parking_lot 0.11.2 -> instant 0.1.13`
   - status: unmaintained

4. `RUSTSEC-2025-0161`
   - dependency chain includes `libsecp256k1 0.7.2`
   - status: unmaintained

5. `RUSTSEC-2024-0436`
   - dependency chain includes `paste 1.0.15`
   - status: unmaintained

Assessment:

- The `sled` advisories continue to indicate maintenance risk in the storage stack.
- Additional advisories now also land through Substrate / cryptography dependency chains.
- These findings do not by themselves prove a directly exploitable application bug in the WebSocket service, but they should be tracked and revisited during dependency updates.

## Recommended Next Steps

Highest-value remaining work:

1. Add per-query metering for all public request paths.
2. Require cryptocurrency payment to the index provider before serving billable queries.
3. Define which request types are billable and enforce that policy before hydration or proof retrieval.
4. Continue to require TLS termination in documented deployment paths.
5. Keep the metrics endpoint internal-only by default in deployment examples.
6. Track or remediate `cargo audit` findings, especially long-term `sled` replacement/containment work and transitive cryptography dependency maintenance risk.
7. Continue expanding end-to-end tests around overload and backpressure behavior.

## Deployment Guidance

For Internet exposure, deploy behind infrastructure that supports the billing and metering model:

- TLS termination
- request logging for query accounting
- per-query metering
- cryptocurrency payment enforcement at the access edge or application layer
- health checks
- internal-only exposure for the metrics port when enabled

Also assume the upstream Substrate node is part of the service envelope:

- keep RPC access restricted where possible
- run archival pruning settings required by the application
- monitor RPC availability separately from the public WebSocket service

Without metering and payment enforcement, the service should not be treated as ready for direct public dapp access.
