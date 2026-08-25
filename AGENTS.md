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

## Toolchain

- The default toolchain is NIGHTLY (see `rust-toolchain.toml`), so every adhoc `cargo` command auto-applies the fast per-profile `-Z` flags via the `[unstable] profile-rustflags` opt-in in `.cargo/config.toml`. Do not pass `+nightly` or set `RUSTFLAGS` — they apply automatically.
- A STABLE build is an explicit opt-out: `just build-stable` / `check-stable` / `test-stable` (via `scripts/build-stable.sh`), which temporarily strips the nightly-only keys and restores them. Do not hand-edit `Cargo.toml` or `.cargo/config.toml` to get stable; use the script.

## Test policy

- Unit tests must be deterministic and must not rely on wall-clock timing.
- Do not use `sleep`, `timeout`, polling delays, scheduler yields, or elapsed-time assertions in unit tests.
- Use explicit synchronization, test hooks, or direct helper/function tests instead.
- If behavior inherently depends on real time, OS file watching, network timing, or similar environment effects, move that coverage to integration tests rather than unit tests.
