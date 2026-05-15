# Quickstart

## 1. Generate A Starter Spec

```bash
acuity-index generate-index-spec ./mychain.toml --url wss://mynode:443
```

This inspects live runtime metadata and writes a starter TOML file.

## 2. Review And Edit The Spec

At minimum, verify:

- `name`
- `genesis_hash`
- `default_url`
- declared `[keys]`
- `[[pallets]]` and `[[pallets.events]]` entries you want indexed

Remove keys you do not need and add composite keys where appropriate.

## 3. Run The Indexer

```bash
acuity-index run ./mychain.toml
```

Common overrides:

```bash
acuity-index run ./mychain.toml --url wss://mynode:443 --queue-depth 4 --port 8172
```

## 4. Query The Service

See the [WebSocket API](./api.md) for the protocol and methods.

By default, connect to:

```text
ws://localhost:8172
```

Minimal request:

```json
{"id":1,"type":"Status"}
```

## 5. Iterate Safely

When `run <INDEX_SPEC>` points to a file, Acuity Index watches that file for
accepted changes. Valid changes restart only the RPC/indexer loop. The public
WebSocket server and optional metrics listener stay up.

Rejected changes do not kill the running process.
