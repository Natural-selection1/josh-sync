# BlueOS Josh sync utilities
This repository contains a binary utility for performing [Josh](https://github.com/josh-project/josh)
synchronizations (pull and push) between a subtree repository and the BlueOS monorepo configured in that subtree.

## Installation
Install a specific commit of the binary so that it matches the reusable workflow revision:

```bash
$ cargo install --locked \
  --git https://github.com/<josh-sync-owner>/josh-sync \
  --rev <josh-sync-commit>
```

## Creating config file

First, create a configuration file for a given subtree repo using `vivoblueos-josh-sync init`. The config will be created under the path `josh-sync.toml`. It is tracked in the subtree repository, so the monorepo source is reviewed together with the mapping.

For a Natural-selection1 trial, a kernel mapping looks like this:

```toml
org = "Natural-selection1"
repo = "kernel"
upstream-repo = "Natural-selection1/blueos-mono"
upstream-branch = "main"
path = "kernel"
filter-version = 2
```

For production, change the reviewed `upstream-repo` to `vivoblueos/blueos` (and `org` to `vivoblueos` if the subtree also moves). Do not pass a different upstream from CI.

If you need to specify a more complex Josh `filter`, use `filter` field in the configuration file instead of the `path` field.

The `init` command will also create an empty `blueos-version` file (if it doesn't already exist) that stores the last configured monorepo SHA that was synced in the subtree.

### Repository mapping examples

Repositories that map directly to a top-level path use the repository name as `path`:

```toml
repo = "kernel"
path = "kernel"
```

Repositories nested under `apps/` use a nested path. In this example, `apps_shell` maps to `apps/shell` in the configured monorepo:

```toml
repo = "apps_shell"
path = "apps/shell"
```

`upstream-branch` selects the monorepo branch. A subtree with a default branch other than `main` does not need a special `josh-sync.toml` setting; configure its PR base only in the CI workflow that consumes this tool:

```yaml
pr-base-branch: blueos-dev
```

The [`josh-sync.example.toml`](josh-sync.example.toml) file contains all the things that can be configured.

## Performing pull

A pull operation resolves the configured `upstream-branch`, fetches its subtree projection, and merges it into the subtree repository. After performing a pull, a pull request is sent against the *subtree repository*.

1) Checkout the latest default branch of the subtree
2) Create a new branch that will be used for the subtree PR, e.g. `pull`
3) Run `vivoblueos-josh-sync pull`
4) Send a PR to the subtree repository

- Note that `vivoblueos-josh-sync` can do this for you if you have the [gh](https://cli.github.com/) CLI tool installed.

You can also configure a set of postprocessing operations to be performed after a successful pull using the `post-pull` configuration.

## Performing push

A push operation takes changes performed in the subtree repository and merges them into the subtree subdirectory of the configured BlueOS monorepo. After performing a push, a PR is sent against that monorepo's configured `upstream-branch`.

1) Checkout the latest default branch of the subtree
2) Run `vivoblueos-josh-sync push <branch> <your-github-username>`

- The branch with the push contents will be created in the `<your-github-username>/<configured-monorepo-name>` fork, in the `<branch>` branch.

3) Send a PR to the configured BlueOS monorepo.

## Automating pulls on CI

This repository contains a reusable workflow for performing the `pull` operation from CI. The workflow does the following:

1) Installs a pinned `vivoblueos-josh-sync` revision (which manages Josh)
2) Performs a `pull` operation
3) Either creates a new PR (if it did not exist) with the resulting pull branch or force-pushes to an existing PR on the subtree repository

Here is an example of how you can use the workflow in a subtree repository:

```yaml
name: blueos-pull

on:
  workflow_dispatch:
  schedule:
    # Run at 04:00 UTC every Monday and Thursday
    - cron: '0 4 * * 1,4'

env:
  # Optional to print detailed command logs
  JOSH_SYNC_VERBOSE: true

jobs:
  pull:
    # During the trial, use Natural-selection1/josh-sync and the same immutable SHA below.
    # Production uses vivoblueos/josh-sync at a reviewed release SHA.
    uses: Natural-selection1/josh-sync/.github/workflows/blueos-pull.yml@<josh-sync-commit>
    with:
      github-app-id: ${{ vars.APP_CLIENT_ID }}
      # Must end with [bot]
      pr-author: "github-actions[bot]"
      josh-sync-repository: Natural-selection1/josh-sync
      josh-sync-revision: <josh-sync-commit>
      pr-base-branch: main     # optional
      branch-name: blueos-pull # optional
    secrets:
      github-app-secret: ${{ secrets.APP_PRIVATE_KEY }}
```

You will need to have a GitHub app configured on the repository with permissions to create pull requests in order to use the workflow.

See [test.md](test.md) for the Natural-selection1 trial sequence and the production migration boundary.

## Git peculiarities

NOTE: If you use Git/SSH protocol to push to your fork of the configured monorepo,
ensure that you have this entry in your Git config,
else the 2 steps that follow would prompt for a username and password:

```
[url "git@github.com:"]
insteadOf = "https://github.com/"
```

### Minimal git config

For simplicity (ease of implementation purposes), the josh-sync script simply calls out to system git. This means that the git invocation may be influenced by global (or local) git configuration.

You may observe "Nothing to pull" even if you *know* blueos-pull has something to pull if your global git config sets `fetch.prunetags = true` (and possibly other configurations may cause unexpected outcomes).

To minimize the likelihood of this happening, you may wish to keep a separate *minimal* git config that *only* has `[user]` entries from global git config, then repoint system git to use the minimal git config instead. E.g.

```
GIT_CONFIG_GLOBAL=/path/to/minimal/gitconfig GIT_CONFIG_SYSTEM='' vivoblueos-josh-sync ...
```
