# Benchmarking

The repository includes an end-to-end benchmark harness for the synthetic
runtime.

## Run The Benchmark

```bash
just benchmark-indexing
```

This recipe:

1. builds the release binaries
2. starts a disposable synthetic node with timed blocks
3. bulk-seeds deterministic transactions
4. runs `benchmark_synthetic_indexing`
5. waits until seeded queries are observable through `GetEvents`
6. emits throughput and size metrics

## Important Benchmark Property

Benchmark success is defined by observable API behavior, not just internal logs.
The harness waits until seeded queries become visible through the public
WebSocket interface.

## Parameterized Runs

```bash
just benchmark-indexing <rpc_port> <queue_depth> <batch_start> <batches> <burst_count> <timeout_secs>
```

Example:

```bash
just benchmark-indexing 9944 8 1000 1000 128 600
```
