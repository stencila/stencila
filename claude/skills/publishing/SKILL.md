---
name: publishing
description: Publishing Stencila documents - rendering to HTML/PDF/DOCX with themes, building and pushing workspace sites, publishing to Ghost or Zenodo, and signing outputs with content credentials. Use when the user wants to publish, deploy a site, apply a theme, produce final outputs, or push to Ghost, Zenodo or Stencila Cloud.
user-invocable: false
---

# Publishing Stencila documents

## Rendering final outputs

`render` executes a document and encodes it to one or more outputs:

```sh
NO_COLOR=1 stencila render article.smd article.html article.pdf --yes
```

Options that matter for publishing:

- `--to <FORMAT>` — set the output format explicitly (otherwise inferred
  from each output's extension).
- `--theme <THEME>` — theme for HTML/PDF outputs; `stencila themes list`
  shows what is available.
- `--standalone` / `--not-standalone` — encode as a complete document
  (with header and footer) or a fragment.
- `--reproducible` — encode outputs (where supported, e.g. DOCX, PDF) with
  links back to the source so the document can be reproduced.
- `--embed-media` / `--extract-media [<FOLDER>]` — inline media as data
  URIs, or write it out to files.
- Document parameters pass after `--`:
  `stencila render template.smd out.html --yes -- --year=2024`

## Workspace sites

A workspace with a `[site]` section in `stencila.toml` can be built and
served as a site:

```sh
NO_COLOR=1 stencila site show --yes      # details of the configured site
NO_COLOR=1 stencila site list --yes      # all routes, configured and file-implied
NO_COLOR=1 stencila site render dist --yes   # render the site to a directory
NO_COLOR=1 stencila site preview --yes   # local preview with live reload
NO_COLOR=1 stencila site push --yes      # push to Stencila Cloud
```

`site add` / `site remove` manage explicit routes; `site domain`, `site
access`, and `site branch` manage the deployed site on Stencila Cloud.

## Publishing to services

```sh
NO_COLOR=1 stencila publish ghost article.smd --yes    # Ghost blog post/page
NO_COLOR=1 stencila publish zenodo article.smd --yes   # Zenodo deposit
NO_COLOR=1 stencila publish stencila article.smd --yes # Stencila Cloud
```

- Ghost: `--post` (default) or `--page`, `--draft` or `--publish`,
  `--title`, `--slug`, `--tag`, `--schedule <SCHEDULE>`. Requires
  `--domain` and `--key` (or the corresponding secrets, see
  `stencila secrets`).
- Zenodo: starts as a draft deposit; `--sandbox` targets the Zenodo sandbox,
  `--reserve-doi` reserves a DOI, `--doi` supplies an existing one, plus
  metadata flags (`--title`, `--description`, `--license`, `--keywords`,
  `--publication-date`). Use `--dry-run` to see what would be deposited.

## Content credentials (C2PA signing)

Rendered outputs and generated assets can carry signed content credentials.
This is configured in `stencila.toml` rather than per-invocation:

```toml
# Simplest: enable with defaults (public profile, automatic signer)
content-credentials = true

# Or detailed configuration
[content-credentials]
enabled = true
profile = "public"   # public | private | full
signer = "auto"
```

The profile controls how much local detail (paths, environment) is embedded
in the credential; use `public` for anything leaving the machine.

## Before publishing

Render locally first and confirm there are no execution errors — publishing
a document whose chunks error produces stale or missing outputs. See the
`execution` skill.
