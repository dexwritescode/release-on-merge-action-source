# Release on Merge Action

[![CI](https://github.com/dexwritescode/release-on-merge-action/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/dexwritescode/release-on-merge-action/actions/workflows/ci.yaml)

Github action to create Github release on merge

## Usage example

``` yaml
on: 
  push:
    branches:
      - main

jobs:
  release-on-merge:
    runs-on: ubuntu-latest
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - uses: dexwritescode/release-on-merge-action@main
        with:
          version-increment-strategy: patch
```

List of inputs:

```yaml
  version-increment-strategy:  # Valid values: major|minor|patch|norelease
    description: 'The version number to increment. Options major|minor|patch|norelease'
    required: true
    default: 'patch'
  initial-version:
    description: 'The very first release version to create. The default Github tag will be v0.1.0'
    required: false
    default: '0.1.0'
  tag-prefix:
    description: 'Git tag prefix. Example the v in v1.2.3'
    required: false
    default: 'v'
  body:
    description: 'Body text to prepend the auto generated body'
    required: false
    default: ''
  generate-release-notes:
    description: 'Whether to generate release notes. Default true.'
    required: false
    default: true
  dry-run:
    description: 'Do not create a release, just log the oputput.'
    required: false
    default: false
```

For stability, we recommend pinning the version of the action. See [Releases](https://github.com/rymndhng/release-on-push-action/releases)


## Development

Uses [babashka](https://github.com/borkdude/babashka)

To run tests:

1. Install [babashka](https://github.com/borkdude/babashka#installation).
2. Install [direnv](https://direnv.net/docs/installation.html)
3. Create a `.envrc` file at the project root from this template:

``` sh
export GITHUB_TOKEN=<YOUR_TOKEN>
export GITHUB_REPOSITORY=rymndhng/release-on-push-action
export GITHUB_SHA=167c690247d0933acde636d72352bcd67e33724b
export GITHUB_API_URL=https://api.github.com
export INPUT_BUMP_VERSION_SCHEME=minor
export INPUT_MAX_COMMITS=5
export INPUT_TAG_PREFIX=v
export INPUT_RELEASE_NAME="<RELEASE_TAG>"
export INPUT_DRY_RUN=1
```

2. Run Tests

``` sh
make test
make dryrun
```