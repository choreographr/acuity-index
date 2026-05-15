# Changelog

This changelog covers the history of the combined `acuity-index` repository,
starting from the initial import of the earlier `acuity-index-substrate` and
`acuity-index-polkadot` work.

It intentionally lists only major changes.

## v0.9.1

### Changed

- Finished the post-`v0.8.0` JSON-RPC API follow-up by simplifying live
  subscription tracking around subscription IDs.
- Updated event responses so hydrated events now expose a top-level
  `timestamp`, and the JSON-RPC event payload merges decoded event data into the
  main event object.
- Added a server WebSocket heartbeat to improve connection liveness handling.
- Tightened test reliability by removing timing-based unit test waits and
  making the unit test suite deterministic.

### Documentation

- Restored the in-repo mdBook documentation and expanded it so the book is once
  again the primary project documentation.
- Moved the API and security reference material into book chapters and updated
  the status subscription documentation to match the current payloads.

## v0.8.0 - 2026-05-01

Released at commit `40a39df`.

### Added

- A config-driven indexing model centered on explicit index-spec TOML files,
  including declared scalar and composite query keys, explicit pallet/event
  mappings, and generated starter specs.
- A stable CLI shape with explicit `run`, `purge-index`, and
  `generate-index-spec` commands, plus `--force` support for overwriting
  generated specs.
- Index-spec hot reload and hot-reloadable runtime options so accepted config
  changes can restart indexing without dropping the public service.
- Support for `spec_change_blocks`, allowing controlled reindexing across
  historical index-spec revisions.
- Composite and multi-field key indexing, generalized runtime key handling, and
  explicit declaration of all query keys in the spec.
- Optional OpenMetrics `/metrics` export for Prometheus scraping.
- Finalized event proof support for `acuity_getEvents`, backed by synthetic
  GRANDPA-enabled test coverage.
- An in-repo synthetic Polkadot runtime with end-to-end integration tests,
  seeding tools, and benchmarking workflows.
- In-repo mdBook documentation covering operator, contributor, and architecture
  workflows.

### Changed

- Removed built-in chain specs, built-in keys, and implicit SDK shortcuts so all
  indexing behavior is now explicit in the index specification.
- Separated the index specification from deployment-specific runtime options.
- Changed the WebSocket API and server behavior substantially, culminating in
  the JSON-RPC 2.0 migration implemented in the release commit.
- Improved indexing throughput and scalability with concurrent backfill/head
  catch-up, blocking-pool event decode work, better synthetic benchmark
  pipelines, and `event_index` widened from `u16` to `u32`.

### Fixed

- Hardened production error handling by removing panic paths on untrusted or
  operational input and returning structured errors instead.
- Preserved WebSocket sessions and subscriptions across upstream RPC
  reconnects, with exponential-backoff recovery and span-state preservation.
- Made span database updates atomic and improved resume safety during reloads,
  reconnects, and shutdown.
- Hardened database path handling and chain-name derived paths against path
  traversal.
- Improved behavior against syncing and warp-synced nodes, including head
  catch-up, backfill correctness, queue handling, and archive-state
  enforcement.
