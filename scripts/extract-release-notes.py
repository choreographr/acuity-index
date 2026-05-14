#!/usr/bin/env python3
import argparse
import pathlib
import re
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract a tagged release section from CHANGELOG.md."
    )
    parser.add_argument(
        "tag",
        nargs="?",
        help="release tag to extract, for example v0.8.0; defaults to v<package version>",
    )
    parser.add_argument(
        "--changelog",
        default="CHANGELOG.md",
        help="path to changelog file (default: CHANGELOG.md)",
    )
    return parser.parse_args()


def default_tag() -> str:
    cargo_toml = pathlib.Path("Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', cargo_toml, re.MULTILINE)
    if not match:
        raise SystemExit("error: could not determine package version from Cargo.toml")
    return f"v{match.group(1)}"


def extract_section(changelog: str, tag: str) -> str:
    pattern = re.compile(rf"^##\s+{re.escape(tag)}(?:\s|$)", re.MULTILINE)
    match = pattern.search(changelog)
    if not match:
        raise SystemExit(f"error: could not find section for {tag} in CHANGELOG.md")

    rest = changelog[match.start():]
    next_match = re.search(r"^##\s+", rest[1:], re.MULTILINE)
    return (rest if next_match is None else rest[: next_match.start() + 1]).rstrip() + "\n"


def main() -> int:
    args = parse_args()
    tag = args.tag or default_tag()
    changelog_path = pathlib.Path(args.changelog)
    changelog = changelog_path.read_text(encoding="utf-8")
    sys.stdout.write(extract_section(changelog, tag))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
