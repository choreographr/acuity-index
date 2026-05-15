# Maintainer Release Process

`acuity-index` uses two separate release tools:

- `cargo-release` handles the crate release itself
- `cargo-dist` handles the GitHub Release artifacts
- GitHub Actions only reacts after a release tag has been pushed

In practice, that means the version bump and crates.io publish happen through
standard Rust release tooling, while GitHub is used only to build and upload
binary artifacts for the tagged release.

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

After `cargo-release` pushes the release tag, GitHub Actions runs the
`cargo-dist` workflow that was generated from the dist configuration.

`cargo-dist` is not the thing that decides whether the release is allowed to
happen. The integration tests and other release checks run locally before the
tag exists.

The GitHub workflow only:

1. reads the pushed version tag
2. plans the dist build for that tag
3. builds the configured release artifacts
4. extracts the matching tagged section from `CHANGELOG.md`
5. creates or updates the GitHub Release with those notes
6. uploads the archives and checksum files

The current dist configuration builds release artifacts for:

- `aarch64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

## Dist Configuration

`cargo-dist` is configured in `dist-workspace.toml`. That file tells dist what
artifacts to build and how the release workflow should behave.

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

## Publicise The Release

Once the release is published, share it in the places most likely to reach
acuity-index users and contributors:

- the Polkadot forum or another main Polkadot/Substrate developer discussion channel
- the Polkadot Discord, Matrix, or Telegram developer channels
- a short X/Twitter post linking to the GitHub release

For larger releases, consider also posting to Hacker News (Show HN) and targeted
Reddit communities. If the release is especially substantial, a short blog or
book note can help with long-tail discovery and search visibility.

## Regenerating Dist CI

If you change `dist-workspace.toml` or other dist-related release settings,
regenerate the GitHub Actions workflow with:

```bash
dist generate --mode ci
```

This updates the generated workflow under `.github/workflows/` so it matches
the current dist configuration.

The generated workflow is intentionally tag-driven only. Do not move the
integration suite into GitHub Actions; those checks are meant to stay in the
local release gate.
