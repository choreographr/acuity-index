# CLI Reference

## Main Command

```bash
acuity-index [OPTIONS] <COMMAND>
```

## Global Options

These options are accepted by `acuity-index` for every command:

| Option | Description |
|---|---|
| `-v, --verbose...` | Increase logging verbosity |
| `-q, --quiet...` | Decrease logging verbosity |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Commands

| Command | Description |
|---|---|
| `run` | Run the indexer for the specified chain |
| `purge-index` | Delete the index database for the specified chain |
| `generate-index-spec` | Generate a starter index spec TOML from live node metadata |

## `run`

```bash
acuity-index run [OPTIONS] <INDEX_SPEC>
```

Run the indexer for the specified chain.

### Arguments

| Argument | Description |
|---|---|
| `<INDEX_SPEC>` | Path to a hot-reloading index specification TOML file |

### Options

| Option | Default | Description |
|---|---|---|
| `--options-config <OPTIONS_CONFIG>` | none | Path to a hot-reloading options TOML file |
| `-d, --db-path <DB_PATH>` | none | Database path |
| `--db-mode <DB_MODE>` | `low-space` | Database mode: `low-space` or `high-throughput` |
| `--db-cache-capacity <DB_CACHE_CAPACITY>` | `1024.00 MiB` | Maximum size for the system page cache |
| `-u, --url <URL>` | none | URL of Substrate node to connect to |
| `--queue-depth <QUEUE_DEPTH>` | `1` | Maximum number of concurrent block requests |
| `-f, --finalized` | false | Only index finalized blocks |
| `-p, --port <PORT>` | `8172` | WebSocket port |
| `--metrics-port <METRICS_PORT>` | none | OpenMetrics HTTP port |
| `--max-connections <MAX_CONNECTIONS>` | `1024` | Maximum concurrent WebSocket connections |
| `--max-total-subscriptions <MAX_TOTAL_SUBSCRIPTIONS>` | `65536` | Maximum total subscriptions across all connections |
| `--max-subscriptions-per-connection <MAX_SUBSCRIPTIONS_PER_CONNECTION>` | `128` | Maximum subscriptions per connection |
| `--subscription-buffer-size <SUBSCRIPTION_BUFFER_SIZE>` | `256` | Per-connection subscription notification buffer size |
| `--subscription-control-buffer-size <SUBSCRIPTION_CONTROL_BUFFER_SIZE>` | `1024` | Subscription control channel buffer size |
| `--idle-timeout-secs <IDLE_TIMEOUT_SECS>` | `300` | Idle connection timeout in seconds |
| `--max-events-limit <MAX_EVENTS_LIMIT>` | `1000` | Maximum number of events returned per query |

## `purge-index`

```bash
acuity-index purge-index [OPTIONS] <INDEX_SPEC>
```

Delete the index database for the specified chain.

### Arguments

| Argument | Description |
|---|---|
| `<INDEX_SPEC>` | Path to an index specification TOML file |

### Options

| Option | Description |
|---|---|
| `-d, --db-path <DB_PATH>` | Database path |

## `generate-index-spec`

```bash
acuity-index generate-index-spec [OPTIONS] --url <URL> <INDEX_SPEC>
```

Generate a starter index spec TOML from live node metadata.

### Arguments

| Argument | Description |
|---|---|
| `<INDEX_SPEC>` | Path to write the generated index spec TOML file |

### Options

| Option | Description |
|---|---|
| `-u, --url <URL>` | URL of Substrate node to connect to |
| `-f, --force` | Overwrite the output file if it already exists |
