# Configuration

Acuity Index uses two configuration layers:

- an index specification TOML that describes chain identity and indexing rules
- an optional runtime options TOML passed via `--options-config`

Runtime precedence is:

```text
CLI flags > --options-config file > built-in defaults
```

## Index Specification

Each chain is described by an index specification passed as `<INDEX_SPEC>`.

```toml
name = "mychain"
genesis_hash = "abc123..."
default_url = "wss://my-node:443"
index_variant = false
spec_change_blocks = [0]

[keys]
account_id = "bytes32"
item_id = "bytes32"
revision_id = "u32"
item_revision = { fields = ["bytes32", "u32"] }

[[pallets]]
name = "MyPallet"

[[pallets.events]]
name = "SomeEvent"

[[pallets.events.params]]
field = "who"
key = "account_id"

[[pallets.events.params]]
fields = ["item_id", "revision_id"]
key = "item_revision"
```

## Important Fields

- `name`: logical name for the spec and default local storage path
- `genesis_hash`: chain identity guard; a database is tied to one chain only
- `default_url`: fallback node URL when CLI and runtime options do not override it
- `index_variant`: whether to store pallet and variant lookups in the variant tree
- `spec_change_blocks`: revision boundaries for historical reindex behavior

`spec_change_blocks` must:

- not be empty
- start with `0`
- be strictly increasing

## Declared Keys

Scalar key kinds:

- `bytes32`
- `u32`
- `u64`
- `u128`
- `string`
- `bool`

Composite keys are declared structurally:

```toml
[keys]
item_revision = { fields = ["bytes32", "u32"] }
```

Event parameter mappings use:

- `field = "..."` for scalar keys
- `fields = ["...", "..."]` for composite keys
- `multi = true` for one-to-many scalar extraction when the event field contains multiple values

Composite key values are ordered and binary encoded, so field order matters.

## Runtime Options Config

Deployment-specific settings can live in a separate TOML file:

```toml
url = "wss://mynode:443"
db_path = "/var/lib/acuity-index/mychain"
db_mode = "low_space"
db_cache_capacity = "1024 MiB"
queue_depth = 4
finalized = false
port = 8172
metrics_port = 9000
max_connections = 1024
max_total_subscriptions = 65536
max_subscriptions_per_connection = 128
subscription_buffer_size = 256
subscription_control_buffer_size = 1024
idle_timeout_secs = 300
max_events_limit = 1000
```

Supported fields:

| Field | Type | Default |
|---|---|---|
| `url` | string | index spec `default_url` |
| `db_path` | string | `~/.local/share/acuity-index/<chain>/db` |
| `db_mode` | string | `low_space` |
| `db_cache_capacity` | string | `1024.00 MiB` |
| `queue_depth` | integer | `1` |
| `finalized` | boolean | `false` |
| `port` | integer | `8172` |
| `metrics_port` | integer | disabled |
| `max_connections` | integer | `1024` |
| `max_total_subscriptions` | integer | `65536` |
| `max_subscriptions_per_connection` | integer | `128` |
| `subscription_buffer_size` | integer | `256` |
| `subscription_control_buffer_size` | integer | `1024` |
| `idle_timeout_secs` | integer | `300` |
| `max_events_limit` | integer | `1000` |

`index_variant` belongs in the index spec, not the runtime options file.

## Hot Reload Notes

- index-spec changes can restart the indexer when accepted
- changes to `name` or `genesis_hash` are rejected
- options-config changes can also be reloaded
- some WebSocket settings are applied live, while others require indexer restart according to runtime validation logic
