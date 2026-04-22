# Release on Merge Action Source

[![CI](https://github.com/dexwritescode/release-on-merge-action-source/actions/workflows/ci.yaml/badge.svg)](https://github.com/dexwritescode/release-on-merge-action-source/actions/workflows/ci.yaml)

This is the source code for [Release on Merge Action](https://github.com/marketplace/actions/release-on-merge-action). For usage documentation see the [action definition repo](https://github.com/dexwritescode/release-on-merge-action).

## What it does

A Rust binary packaged as a Docker-based GitHub Action that creates a GitHub release on every merge to main. It:

1. Reads configuration from GitHub Action inputs
2. Optionally resolves the increment strategy from the merged PR's labels
3. Fetches the latest release from the GitHub API
4. Increments the semver tag (stable or pre-release)
5. Creates a new GitHub release and writes `version` / `tag` outputs

## Inputs

| Input | Default | Description |
|---|---|---|
| `version-increment-strategy` | `patch` | `major` \| `minor` \| `patch` \| `norelease` |
| `initial-version` | `0.1.0` | Version used when no prior release exists |
| `tag-prefix` | `v` | Prefix prepended to the version number |
| `body` | `''` | Text prepended to auto-generated release notes |
| `generate-release-notes` | `true` | Auto-generate GitHub release notes |
| `dry-run` | `false` | Log without creating the release |
| `github-host` | `https://api.github.com` | Override for GitHub Enterprise |
| `prerelease` | `false` | Create a pre-release with counter suffix |
| `prerelease-identifier` | `rc` | Identifier in the pre-release suffix (e.g. `beta` → `v1.2.0-beta.1`) |
| `use-label-strategy` | `false` | Derive strategy from merged PR labels |
| `label-major` | `release:major` | Label that triggers a major bump |
| `label-minor` | `release:minor` | Label that triggers a minor bump |
| `label-patch` | `release:patch` | Label that triggers a patch bump |
| `label-skip` | `release:skip` | Label that skips release creation |

## Outputs

| Output | Example | Description |
|---|---|---|
| `version` | `1.2.3` or `1.2.3-rc.1` | Version number without prefix |
| `tag` | `v1.2.3` or `v1.2.3-rc.1` | Full tag including prefix |

## Development

```bash
cargo build --release   # build
cargo test              # run all tests (31 tests)
cargo clippy            # lint
```

The Docker image is built and published to `ghcr.io/dexwritescode/release-on-merge-action-source` via the CD workflow when a GitHub release is published.
