# WebSocket API

The public API is a JSON-over-WebSocket protocol exposed on `ws://localhost:8172`
by default.

There are two message classes:

- request/response messages, which always carry an `id`
- notifications, which never carry an `id`

For Internet-facing deployment guidance, read [Security And Deployment](./security.md).

## Requests

Every request includes:

- `id`: client-selected unsigned integer
- `type`: request discriminator

Example:

```json
{"id":1,"type":"Status"}
```

## Main Request Types

- `Status`: returns indexed block spans
- `Variants`: returns pallet and event variant metadata
- `GetEvents`: queries indexed events for a key, optionally paginated with `before`
  and optionally enriched with finalized proofs via `includeProofs`
- `SubscribeStatus`: subscribes to status changes
- `SubscribeEvents`: subscribes to updates for one key
- `UnsubscribeStatus`: removes a status subscription
- `UnsubscribeEvents`: removes an event subscription
- `SizeOnDisk`: returns current database size

## `Status`

Request:

```json
{"id":1,"type":"Status"}
```

Example response:

```json
{
  "id": 1,
  "type": "status",
  "data": [
    {"start": 1, "end": 1000}
  ]
}
```

## `Variants`

Request:

```json
{"id":2,"type":"Variants"}
```

Example response:

```json
{
  "id": 2,
  "type": "variants",
  "data": [
    {
      "index": 42,
      "name": "Referenda",
      "events": [
        {"index": 0, "name": "Submitted"}
      ]
    }
  ]
}
```

## `GetEvents`

Example request:

```json
{
  "id": 3,
  "type": "GetEvents",
  "key": {
    "type": "Custom",
    "value": {"name": "ref_index", "kind": "u32", "value": 42}
  },
  "limit": 100,
  "before": null,
  "includeProofs": false
}
```

Request fields:

- `key`: query key
- `limit`: optional `u16`, default `100`
- `before`: optional event cursor
- `includeProofs`: optional boolean, default `false`

Response payload includes:

- `key`: the queried key
- `events`: matching event refs, newest first
- `decodedEvents`: decoded payloads when available
- `proofsByBlock`: omitted unless proofs were requested; `null` if requested but unavailable
- `proofsStatus`: omitted unless proofs were requested

When proofs are available, each proof object includes:

- `blockNumber`
- `blockHash`
- `header`
- `storageKey`
- `storageValue`
- `storageProof`

If proofs are requested while the indexer is not running in finalized mode,
`proofsByBlock` is `null` and `proofsStatus.reason` is
`finalized_proofs_unavailable`.

## Composite Custom Keys

Composite keys use an ordered array of typed values:

```json
{
  "id": 3,
  "type": "GetEvents",
  "key": {
    "type": "Custom",
    "value": {
      "name": "item_revision",
      "kind": "composite",
      "value": [
        {"kind": "bytes32", "value": "0xabc123..."},
        {"kind": "u32", "value": 7}
      ]
    }
  }
}
```

## `SizeOnDisk`

Request:

```json
{"id":4,"type":"SizeOnDisk"}
```

Example response:

```json
{
  "id": 4,
  "type": "sizeOnDisk",
  "data": 123456
}
```

## Notifications

Representative notification types:

- `status`
- `eventNotification`
- `subscriptionTerminated`

`eventNotification` includes the subscribed key, matching event reference, and a
decoded event object.

`subscriptionTerminated` is sent best-effort before the server drops a
subscriber that cannot keep up. The current reason is `backpressure`.

## Errors

Invalid requests and handler failures are returned as responses with
`type: "error"` and the original request `id` when available.

Current error codes include:

- `invalid_request`
- `internal_error`
- `subscription_limit`
- `temporarily_unavailable`

During node outages, local requests such as `Status` and `SizeOnDisk` continue
to work. `Variants` and `GetEvents` require live RPC access and return
`temporarily_unavailable` until the node connection is restored.

## Query Limits And Validation

Current protocol-level validation includes:

- custom key names limited to `128` bytes
- custom string values limited to `1024` bytes
- composite keys with at most `64` elements
- composite nesting depth limited to `8`
- encoded custom key payload size limited to `16384` bytes
- maximum events returned per query limited by server configuration, default `1000`

## Pagination Semantics

`GetEvents` returns matches in descending order by `(blockNumber, eventIndex)`.

- default `limit` is `100`
- `before` is exclusive
