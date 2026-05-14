# Releases

`acuity-index` uses a split release flow:

- `cargo-release` is the release authority
- `cargo-dist` publishes GitHub release artifacts
- GitHub Actions only reacts to pushed release tags

This keeps versioning and crates.io publishing in standard Rust tooling while
keeping GitHub integration minimal.

## Prerequisites

The release machine must have:

- Rust stable
- `cargo-release`
- `cargo-dist`
- `just`
- `polkadot-omni-node`

The integration suite is part of the release gate, so the machine running the
release must be able to build the in-repo synthetic runtime and run the local
node-backed tests.

## Release Gate

Before `cargo-release` is allowed to tag or publish, it runs:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
just test-integration
```

These checks are wired through `scripts/release-checks.sh` and are also exposed
as:

```bash
just release-checks
```

If any step fails, the release stops before the version is bumped, before a tag
is created, and before anything is published.

## Updating The Changelog

Before running the release command, update the canonical repository changelog in
`CHANGELOG.md` and commit it. The GitHub release workflow publishes the matching
version section from that file as the release notes.

You can preview exactly what GitHub will publish with:

```bash
just release-notes
just release-notes v0.8.0
```

That helper reads `CHANGELOG.md` and extracts the section whose heading starts
with `## v<version>`.

The book no longer owns the release history; if you want a book-visible entry
point, keep `book/src/changelog.md` as a short pointer back to `CHANGELOG.md`.

## Creating A Release

Run one of:

```bash
just release patch
just release minor
just release major
```

Equivalent direct commands are:

```bash
cargo release patch --execute
cargo release minor --execute
cargo release major --execute
```

`cargo-release` is configured to:

1. require the `main` branch
2. run the release gate
3. bump the crate version in `Cargo.toml`
4. create a release commit with the message `chore(release): <version>`
5. create a git tag named `v<version>`
6. publish the crate to crates.io
7. push the release commit and tag

Dry runs are still available via plain `cargo release <level>` without
`--execute`.

## GitHub Artifacts

Once the release tag is pushed, GitHub Actions runs `cargo-dist`.

GitHub does **not** run the integration tests. Those are intentionally part of
the local release gate instead of the GitHub workflow.

The tag workflow only:

1. plans the dist build
2. builds the configured release artifacts
3. extracts the matching tagged section from `CHANGELOG.md`
4. creates or updates the GitHub Release with those release notes
5. uploads archives and checksum files

The current dist configuration builds release artifacts for:

- `aarch64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

## Dist Configuration

`cargo-dist` is configured in `dist-workspace.toml`.

The repo uses a dedicated `dist` profile in `Cargo.toml`:

```toml
[profile.dist]
inherits = "release"
lto = "thin"
```

That keeps distribution builds optimized while avoiding the heavier `fat` LTO
setting used by the normal `release` profile.

## Recommended Maintainer Flow

For a normal patch release:

```bash
git switch main
git pull --ff-only
just release-checks
just release patch
```

After the tag is pushed:

- crates.io gets the new crate release from `cargo-release`
- GitHub Releases gets binary artifacts from `cargo-dist`

## Regenerating Dist CI

If the dist configuration changes, regenerate the workflow with:

```bash
dist generate --mode ci
```

This project keeps the generated workflow tag-driven only. Do not move the
integration suite into GitHub Actions; the release gate is intentionally local.
