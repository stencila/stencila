---
title: "`stencila workflows`"
description: Manage workflow definitions
---

Manage workflow definitions

# Usage

```sh
stencila workflows [COMMAND]
```

# Examples

```bash
# List all workflows
stencila workflows

# Show details about a specific workflow
stencila workflows show data-pipeline

# Validate a workflow by name, directory, or file path
stencila workflows validate data-pipeline

# Create a new workflow in the workspace
stencila workflows create my-workflow "A multi-stage data pipeline"

# Run a workflow
stencila workflows run code-review

# Run a workflow with a goal override
stencila workflows run code-review --goal "Implement login feature"

# List recent workflow runs
stencila workflows runs

# Resume the last failed or interrupted run
stencila workflows resume

# Resume a specific run by ID
stencila workflows resume 01926f3a-...

# Save an ephemeral workflow
stencila workflows save my-workflow

# Discard an ephemeral workflow
stencila workflows discard my-workflow
```

# Subcommands

| Command                   | Description                                             |
| ------------------------- | ------------------------------------------------------- |
| [`list`](list.md)         | List available workflows                                |
| [`show`](show.md)         | Show a workflow                                         |
| [`validate`](validate.md) | Validate a workflow                                     |
| [`create`](create.md)     | Create a new workflow                                   |
| [`run`](run.md)           | Run a workflow                                          |
| [`runs`](runs.md)         | List recent workflow runs                               |
| [`resume`](resume.md)     | Resume a failed, cancelled, or interrupted workflow run |
| [`save`](save.md)         | Save an ephemeral workflow                              |
| [`discard`](discard.md)   | Discard an ephemeral workflow                           |
