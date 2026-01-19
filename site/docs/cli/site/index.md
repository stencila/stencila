---
title: "`stencila site`"
description: Manage the workspace site
---

Manage the workspace site

# Usage

```sh
stencila site [COMMAND]
```

# Examples

```bash
# View details of the workspace site
stencila site
stencila site show

# List configured routes
stencila site list

# Add a route
stencila site add / index.md
stencila site add /about/ README.md
stencila site add /old/ --redirect /new/ --status 301

# Remove a route
stencila site remove /about/

# Push site content to cloud
stencila site push

# Show current access restrictions
stencila site access

# Make site public (remove all restrictions)
stencila site access --public

# Enable team access restriction
stencila site access team

# Set a password for the site
stencila site access password

# Clear the password
stencila site access password --clear
```

# Subcommands

| Command                       | Description                                         |
| ----------------------------- | --------------------------------------------------- |
| [`show`](show.md)             | Show details of the workspace site                  |
| [`list`](list.md)             | List all routes (configured and file-implied)       |
| [`add`](add.md)               | Add a route                                         |
| [`remove`](remove.md)         | Remove a route                                      |
| [`render`](render.md)         | Render site content to a directory                  |
| [`preview`](preview.md)       | Preview the workspace site locally with live reload |
| [`push`](push.md)             | Push site content to Stencila Cloud                 |
| [`access`](access/index.md)   | Manage access restrictions for the workspace site   |
| [`reviews`](reviews/index.md) | Manage site reviews configuration                   |
| [`domain`](domain/index.md)   | Manage custom domain for the workspace site         |
| [`branch`](branch/index.md)   | Manage branches for the workspace site              |
