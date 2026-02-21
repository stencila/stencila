---
title: Syncing Guide
description: Push and pull workspace database state across collaborators using git and Stencila Cloud
---

This guide walks you through syncing your workspace database so collaborators can share workflow run history and other workspace state.

# Prerequisites

Before syncing, you need:

- A Stencila workspace initialized with `stencila init` (this creates `stencila.toml` with a `workspace.id`)
- Authentication with Stencila Cloud (`stencila signin`)
- A git repository (the manifest file is tracked by git)

# First-time push

After running workflows or building other workspace state, push the database to make it available to collaborators:

```bash
stencila db push -m "initial state"
```

On the first push, Stencila creates a compressed snapshot of the entire database and uploads it to Stencila Cloud. It then writes a manifest file at `.stencila/db.json` that describes this snapshot.

Commit the manifest to git:

```bash
git add .stencila/db.json
git commit -m "Add workspace db state"
git push
```

# Pulling database state

When you clone a project that has a database manifest, or after a `git pull` brings an updated manifest, run:

```bash
stencila db pull
```

This downloads the base snapshot and any changesets from Stencila Cloud, applies them to a temporary database, and then atomically replaces your local database. If anything goes wrong during the download or apply, your existing database is untouched.

If you don't have a local database yet (e.g. you just cloned the project), `pull` performs a full restore from the snapshot plus all changesets.

# Ongoing push/pull workflow

After the initial setup, the workflow is:

1. Make changes locally (run workflows, etc.)
2. Push the changes:

   ```bash
   stencila db push -m "add batch-2 results"
   ```

   Subsequent pushes upload only the incremental changes (a changeset), not the entire database. If nothing has changed, push detects this and skips.

3. Commit the updated manifest:

   ```bash
   git add .stencila/db.json
   git commit -m "Update db state"
   git push
   ```

4. Collaborators pull:

   ```bash
   git pull
   stencila db pull
   ```

# Checking sync status

To see where things stand:

```bash
stencila db status
```

This shows the database size, schema version, base snapshot, how many changesets have been applied versus how many exist in the manifest, and whether your local database is up to date with the manifest or has diverged.

# Viewing history

To see the changeset history recorded in the manifest:

```bash
stencila db log
```

This shows each changeset with its hash, timestamp, and message (if one was provided during push).

# Compacting history

Over time, changesets accumulate. When there are many, replay time during pull grows. You can compact by creating a new baseline snapshot:

```bash
stencila db snapshot -m "compact after batch processing"
git add .stencila/db.json
git commit -m "Compact db history"
git push
```

This resets the changeset list — subsequent pulls download just the new snapshot instead of replaying dozens of changesets.

Stencila also rotates to a new snapshot automatically when adding another changeset would bring the count to 50, or when the existing cumulative changeset size is already at least 50 MB.

# Team collaboration

## Single producer, many consumers

The simplest pattern: one person runs workflows and pushes, everyone else pulls. There is no risk of conflicts because only one person modifies the database.

## Multiple contributors

When more than one person pushes changes, coordinate through git:

1. Person A pushes and commits the manifest.
2. Person B makes local changes, then tries to push:

   ```bash
   stencila db push -m "my changes"
   git add .stencila/db.json
   git commit -m "Update db state"
   git push  # ← often fails as non-fast-forward (manifest changed upstream)
   ```

3. Person B resolves by pulling the latest:

   ```bash
   git pull                    # gets Person A's manifest
   stencila db pull            # applies Person A's changesets
   stencila db push -m "my changes"  # re-diffs against updated state
   git add .stencila/db.json
   git commit -m "Update db state"
   git push
   ```

   Because changesets are computed by diffing the current database against the manifest head (not accumulated in a persistent session), the second push correctly captures only the new differences.

## PR-based workflow

For more control, use feature branches:

1. Create a branch, run workflows, push, and commit the manifest.
2. Open a PR. Reviewers can see the manifest diff (changeset hashes and messages).
3. After merge, collaborators `git pull` and `stencila db pull` on the main branch.

## Branch switching

If you `git checkout` a branch with a different manifest, your local database may be out of sync. `stencila db status` detects this (it reports "diverged" when your local sync position isn't in the current manifest). To fix it:

```bash
stencila db reset
```

This downloads the snapshot and changesets from Stencila Cloud and rebuilds the database locally.
