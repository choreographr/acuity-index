# AGENTS

- [Architecture](./ARCHITECTURE.md)
- [API](./API.md)
- [Security](./SECURITY.md)

## Test policy

- Unit tests must be deterministic and must not rely on wall-clock timing.
- Do not use `sleep`, `timeout`, polling delays, scheduler yields, or elapsed-time assertions in unit tests.
- Use explicit synchronization, test hooks, or direct helper/function tests instead.
- If behavior inherently depends on real time, OS file watching, network timing, or similar environment effects, move that coverage to integration tests rather than unit tests.
