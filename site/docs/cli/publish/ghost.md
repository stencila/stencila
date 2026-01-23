---
title: "`stencila publish ghost`"
description: Publish to Ghost
---

Publish to Ghost

# Usage

```sh
stencila publish ghost [OPTIONS] <PATHS>...
```

# Arguments

| Name      | Description                    |
| --------- | ------------------------------ |
| `<PATHS>` | Paths to the files to publish. |

# Options

| Name                   | Description                                                                                               |
| ---------------------- | --------------------------------------------------------------------------------------------------------- |
| `--domain`             | The Ghost domain.                                                                                         |
| `--key`                | The Ghost Admin API key.                                                                                  |
| `--post`               | Create a post. Possible values: `true`, `false`. Default value: `true`.                                   |
| `--page`               | Create a page. Possible values: `true`, `false`.                                                          |
| `--push`               | Create or update Ghost post or page from a file. Possible values: `true`, `false`. Default value: `true`. |
| `--pull`               | Update file from an existing Ghost post or page. Possible values: `true`, `false`.                        |
| `--id`                 | Ghost id of the page or post.                                                                             |
| `--title`              | Title for page or post.                                                                                   |
| `--draft`              | Mark page or post as draft. Possible values: `true`, `false`. Default value: `false`.                     |
| `--publish`            | Publish page or post. Possible values: `true`, `false`.                                                   |
| `--schedule`           | Schedule page or post.                                                                                    |
| `--slug`               | Set slug(URL slug the page or post will be available at).                                                 |
| `--tag`                | Tags for page or post.                                                                                    |
| `--excerpt`            | Excerpt for page or post.                                                                                 |
| `--featured`           | Feature post or page. Possible values: `true`, `false`.                                                   |
| `--inject-code-header` | Inject HTML header.                                                                                       |
| `--inject-code-footer` | Inject HTML footer.                                                                                       |
| `--dry-run`            | Dry run test. Possible values: `true`, `false`. Default value: `false`.                                   |
