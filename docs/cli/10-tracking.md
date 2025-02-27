---
title: Tracking
description: Using the Stencila CLI to track documents
config:
  publish:
    ghost:
      slug: cli-tracking
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

```sh
stencila init
stencila new
stencila status
stencila track
stencila untrack
stencila move
stencila remove
```

# Overview 

Stencila's tracking features are used to be able to track documents, and changes to them in a coherent way. [Stencila's Schema](docs/schema) enables rich provenance data to be stored, but in order to not clutter the file itself with these records, they're stored in a `.stencila` folder. Similar to how version control systems can track files, Stencila tools, and the Stencail VSCode extension will save provenance records for files that are being tracked.

# Commands 

## `stencila init`

Begins tracking a directory and its sub directory with metadata about files. This tracking information is stored in a hidden `.stencila` directory. 

## `stencila new`

Creates a new file which is tracked by Stencila.

## `stencila status`

Shows all of the tracked files in the current scope (nearest `.stencila` tracking folder).

## `stencila track`

Explicity add an existing file for tracking.


## `stencila untrack`

Explicity remove tracking from a file, but leave the file in place.

## `stencila move`

Update the location and tracking record for an already tracked file. Equivalent to using `mv`, but updates tracking record.

## `stencila remove`

Remove both a file and its tracking record.