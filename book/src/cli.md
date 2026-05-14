# CLI Reference

## Main Command

```bash
acuity-index <COMMAND>
```

## Commands

| Command | Description |
|---|---|
| `run &lt;INDEX_SPEC&gt; [OPTIONS]` | Run the indexer for an index specification |
| `purge-index &lt;INDEX_SPEC&gt; [OPTIONS]` | Delete the index database for an index spec |
| `generate-index-spec &lt;INDEX_SPEC&gt; --url &lt;URL&gt; [--force|-f]` | Inspect live metadata and write a starter index specification |

## Run Options

| Option | Default | Description |
|---|---|---|
| `--options-config &lt;PATH&gt;` | none | Path to runtime options TOML |
| `-d, --db-path &lt;PATH&gt;` | `~/.local/share/acuity-index/<spec-name>/db` | Database directory |
| `--db-mode &lt;MODE&gt;` | `low-space` | `low-space` or `high-throughput` |
| `--db-cache-capacity &lt;SIZE&gt;` | `1024.00 MiB` | Maximum `sled` page cache |
| `-u, --url &lt;URL&gt;` | index spec default | Substrate node WebSocket URL |
| `--queue-depth &lt;N&gt;` | `1` | Concurrent block requests for backfill and head catch-up |
| `-f, --finalized` | `false` | Index finalized blocks only |
| `-p, --port &lt;PORT&gt;` | `8172` | Public WebSocket API port |
| `--metrics-port &lt;PORT&gt;` | disabled | Optional OpenMetrics HTTP port |
| `--max-connections &lt;N&gt;` | `1024` | Maximum concurrent WebSocket connections |
| `--max-total-subscriptions &lt;N&gt;` | `65536` | Maximum subscriptions across all connections |
| `--max-subscriptions-per-connection &lt;N&gt;` | `128` | Maximum subscriptions on one connection |
| `--subscription-buffer-size &lt;N&gt;` | `256` | Per-connection notification buffer size |
| `--subscription-control-buffer-size &lt;N&gt;` | `1024` | Subscription control channel buffer size |
| `--idle-timeout-secs &lt;N&gt;` | `300` | Idle connection timeout |
| `--max-events-limit &lt;N&gt;` | `1000` | Maximum events returned per query |
| `-v / -q` | none | Increase or decrease log verbosity |

`run` requires a positional `&lt;INDEX_SPEC&gt;` before any options.

Running with `--finalized` also enables finalized proof responses for
`GetEvents` requests that set `includeProofs: true`.
