# BlueOS Josh sync utilities
This repository contains a binary utility for performing [Josh](https://github.com/josh-project/josh)
synchronizations (pull and push) of Josh subtrees in the [vivoblueos/blueos] repository.

## Installation
You can install the binary `vivoblueos-josh-sync` tool using the following command:

```bash
$ cargo install --locked --git https://github.com/vivoblueos/josh-sync
```

## Creating config file

First, create a configuration file for a given subtree repo using `vivoblueos-josh-sync init`. The config will be created under the path `josh-sync.toml`. Modify the file to fill in the name of the subtree repository (e.g. `kernel`) and its relative path in the main `vivoblueos/blueos` repository (e.g. `kernel`).

If you need to specify a more complex Josh `filter`, use `filter` field in the configuration file instead of the `path` field.

The `init` command will also create an empty `blueos-version` file (if it doesn't already exist) that stores the last `vivoblueos/blueos` SHA that was synced in the subtree.

### Repository mapping examples

Repositories that map directly to a top-level path use the repository name as `path`:

```toml
repo = "kernel"
path = "kernel"
```

Repositories nested under `apps/` use a nested path. In this example, `apps_shell` maps to `apps/shell` in `vivoblueos/blueos`:

```toml
repo = "apps_shell"
path = "apps/shell"
```

Repositories with a default branch other than `main` do not need a special `josh-sync.toml` setting. Configure that branch only in the CI workflow that consumes this tool:

```yaml
pr-base-branch: blueos-dev
```

The [`josh-sync.example.toml`](josh-sync.example.toml) file contains all the things that can be configured.

## Performing pull

A pull operation fetches changes to the subtree subdirectory that were performed in `vivoblueos/blueos` and merges them into the subtree repository. After performing a pull, a pull request is sent against the *subtree repository*. We *pull from `vivoblueos/blueos`*.

1) Checkout the latest default branch of the subtree
2) Create a new branch that will be used for the subtree PR, e.g. `pull`
3) Run `vivoblueos-josh-sync pull`
4) Send a PR to the subtree repository

- Note that `vivoblueos-josh-sync` can do this for you if you have the [gh](https://cli.github.com/) CLI tool installed.

You can also configure a set of postprocessing operations to be performed after a successful pull using the `post-pull` configuration.

## Performing push

A push operation takes changes performed in the subtree repository and merges them into the subtree subdirectory of the `vivoblueos/blueos` repository. After performing a push, a push request is sent against `vivoblueos/blueos`. We *push to `vivoblueos/blueos`*.

1) Checkout the latest default branch of the subtree
2) Run `vivoblueos-josh-sync push <branch> <your-github-username>`

- The branch with the push contents will be created in `https://github.com/<your-github-username>/blueos` fork, in the `<branch>` branch.

3) Send a PR to [vivoblueos/blueos]

## Automating pulls on CI

This repository contains a reusable workflow for performing the `pull` operation from CI. The workflow does the following:

1) Installs `vivoblueos-josh-sync` and `josh`
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
    uses: vivoblueos/josh-sync/.github/workflows/blueos-pull.yml@main
    with:
      github-app-id: ${{ vars.APP_CLIENT_ID }}
      # Must end with [bot]
      pr-author: "github-actions[bot]"
      pr-base-branch: main     # optional
      branch-name: blueos-pull # optional
    secrets:
      github-app-secret: ${{ secrets.APP_PRIVATE_KEY }}
```

You will need to have a GitHub app configured on the repository with permissions to create pull requests in order to use the workflow.

## Git peculiarities

NOTE: If you use Git/SSH protocol to push to your fork of [vivoblueos/blueos],
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

[vivoblueos/blueos]: (https://github.com/vivoblueos/blueos)
