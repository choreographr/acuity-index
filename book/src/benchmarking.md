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
