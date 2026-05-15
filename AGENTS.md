# AGENTS

## Primary docs

- [Architecture](./ARCHITECTURE.md)
- [Book docs](./book/src/SUMMARY.md)
- [WebSocket API](./book/src/api.md)
- [Configuration](./book/src/configuration.md)
- [Security](./book/src/security.md)

## Guidance for changes

- Treat `ARCHITECTURE.md` as the source of truth for high-level invariants, data flow, and ownership boundaries.
- Prefer updating the relevant book page or code comments instead of re-stating architecture here.
- Keep this file focused on navigation and repo-specific test policy.

## Test policy

- Unit tests must be deterministic and must not rely on wall-clock timing.
- Do not use `sleep`, `timeout`, polling delays, scheduler yields, or elapsed-time assertions in unit tests.
- Use explicit synchronization, test hooks, or direct helper/function tests instead.
- If behavior inherently depends on real time, OS file watching, network timing, or similar environment effects, move that coverage to integration tests rather than unit tests.
