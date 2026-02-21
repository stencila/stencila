---
title: Troubleshooting
description: Common issues and solutions for workspace database sync
---

This page covers common problems you might encounter with workspace database sync and how to resolve them.

# Verifying database integrity

If you suspect your local database has drifted from the manifest state, verify it:

```bash
stencila db verify
```

This rebuilds the database from the manifest (downloading the snapshot and replaying all changesets) in a temporary directory and compares it against your local database table-by-table. It reports whether the data matches or lists the specific differences.

If verification fails, rebuild from the manifest:

```bash
stencila db reset
```

# Database has diverged

**Symptom:** `stencila db status` shows "diverged" or `stencila db pull` fails with "Local database has diverged from the manifest."

**Cause:** This happens when a `git checkout` moves the manifest backwards or sideways relative to your local database — your local sync position isn't in the current manifest's history.

**Solutions:**

- If you have local changes you want to keep, push first (on the original branch), then switch branches and reset:

  ```bash
  git checkout original-branch
  stencila db push -m "save my changes"
  git add .stencila/db.json && git commit -m "Save db state"
  git checkout other-branch
  stencila db reset
  ```

- If you don't need local changes, rebuild from the manifest:

  ```bash
  stencila db reset
  ```

- Or force a pull, which discards local state:

  ```bash
  stencila db pull --force
  ```

# Network errors during push

**Symptom:** Push fails with "cannot reach Stencila Cloud" or a timeout error.

**What happens:** If the blob upload fails, the manifest is not updated — your local database is unchanged. If the blob uploads succeed but the manifest write fails, the blobs sit on the cloud unreferenced (harmless).

**Solution:** Retry the push. Blobs are deduplicated by hash, so re-uploading is safe and efficient.

# Network errors during pull

**Symptom:** Pull fails partway through downloading blobs.

**What happens:** Pull applies changes to a temporary file. If anything fails, the working database is untouched.

**Solution:** Retry the pull. Already-downloaded blobs are in the local cache and won't be re-downloaded.

# Missing blob on pull

**Symptom:** Pull fails with "missing blob" identifying a specific hash.

**Cause:** The blob referenced by the manifest doesn't exist on Stencila Cloud. This can happen if someone ran `stencila db gc` too aggressively, or if blobs were manually deleted.

**Solution:** Contact the person who last pushed and ask them to re-push. If they still have the database locally, `stencila db snapshot` followed by a push will create a new baseline that doesn't depend on the missing blob.

# Compacting a long changeset history

**Symptom:** `stencila db log` shows dozens of changesets. Push and pull are slower because they must reconstruct the manifest head by replaying all changesets.

**Solution:** Create a new baseline snapshot:

```bash
stencila db snapshot -m "compact history"
git add .stencila/db.json
git commit -m "Compact db history"
git push
```

This resets the changeset list. Stencila also does this automatically when adding another changeset would bring the count to 50, or when the existing cumulative changeset size is already at least 50 MB.

# Cleaning up local disk space

Downloaded snapshots and changesets are cached at `.stencila/cache/db/`. After snapshot rotation, old blobs in the cache are no longer needed.

```bash
stencila db clean
```

This removes cached blobs that aren't referenced by the current manifest.

# Cleaning up remote blobs

After rebasing, force-pushing, or compacting, orphaned blobs may remain on Stencila Cloud. To remove them:

```bash
stencila db gc
```

This scans all git refs (branches, tags, remote-tracking branches) to find every manifest version, collects the set of referenced blob hashes, then deletes any remote blobs not in that set. It offers to run `git fetch --all` first to ensure remote-tracking refs are up to date.

Preview what would be removed without deleting:

```bash
stencila db gc --dry-run
```

For non-interactive use (e.g. in scripts), you can explicitly control whether `git fetch` runs:

```bash
# Fetch first, then GC
stencila db gc --fetch

# Skip fetch
stencila db gc --no-fetch
```

# Schema migration created a snapshot unexpectedly

**Symptom:** After updating Stencila (which includes a new database migration), your next push says "Schema changed, creating snapshot instead of changeset" and uploads a full snapshot.

**This is expected behavior.** When the schema version changes, Stencila automatically creates a snapshot to ensure all changesets within a snapshot epoch are recorded against the same schema version. The snapshot is larger than a changeset but this only happens once per migration. Subsequent pushes resume creating incremental changesets.

# Crash left a temporary file behind

**Symptom:** You see files like `db.sqlite3.tmp`, `db.head.tmp`, or `db.verify.tmp` in `.stencila/`.

**Cause:** A previous sync command was interrupted. `db.sqlite3.tmp` is used by pull/reset, `db.verify.tmp` by verify, and `db.head.tmp` by push when reconstructing manifest head for diffing.

**Solution:** These are cleaned up automatically the next time the corresponding command runs (`pull`/`reset` for `db.sqlite3.tmp`, `verify` for `db.verify.tmp`, `push` for `db.head.tmp`). You can also safely delete them manually.

# "No workspace.id configured"

**Symptom:** Push or pull fails with "No workspace.id configured."

**Solution:** Run `stencila init` to create a `stencila.toml` with a workspace ID, or add `workspace.id` to your existing `stencila.toml` manually.
