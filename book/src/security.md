# Security And Deployment

Acuity Index is designed to expose a public WebSocket service, so deployment
controls matter as much as application behavior.

## Current Hardening

The service currently includes:

- WebSocket frame and message size limits
- bounded subscription control queues
- bounded custom key sizes in request handling
- a global connection cap
- a total subscription cap
- a per-connection subscription cap
- idle connection timeout
- graceful overload rejection
- reconnect behavior for transient upstream RPC failures

Default limits include:

- `max_message_size = 256 KiB`
- `max_frame_size = 64 KiB`
- `CustomKey.name <= 128` bytes
- `CustomValue::String <= 1024` bytes
- global connection cap of `1024`
- total subscription cap of `65536`
- per-connection subscription cap of `128`
- idle timeout of `300` seconds
- query `max_events_limit` of `1000`

Duplicate subscriptions do not consume extra quota.

## Main Residual Risks

### No Authentication Or Authorization

The public service does not authenticate clients. Any reachable client can query
indexed data and subscribe to updates.

### No In-Process TLS

The server speaks plain WebSocket (`ws`), not `wss`. Use external TLS
termination for Internet-facing deployments.

### Expensive Public Endpoints

`Variants`, `GetEvents`, and live subscriptions remain public. Limits reduce
blast radius, but they are not a substitute for real rate limiting.

### Operational Metadata Exposure

`SizeOnDisk` is public. When metrics are enabled, `/metrics` exposes operational
state that should usually remain internal.

### Dependency Risk

Like any Rust service, the project inherits maintenance and advisory risk from
its dependency graph.

## Deployment Guidance

For Internet-facing deployment, put the service behind infrastructure that
provides at least:

- TLS termination
- request logging
- connection or IP rate limiting
- firewalling or edge filtering
- overload monitoring and health checks
- internal-only exposure for the metrics listener when enabled

## Metrics

If `--metrics-port` is configured, the process serves `/metrics` in OpenMetrics
text format on a separate HTTP listener. Treat that endpoint as an internal
observability surface, not part of the public API.

## Storage And Chain Identity

The most important operational safety guard is the stored `genesis_hash` check.
It prevents one database path from silently mixing data from multiple chains.
