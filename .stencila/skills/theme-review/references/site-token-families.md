# Site token family review reference

Use this file for site-theme terminology, component families, and exact-name pitfalls that commonly affect review quality.

## Site component families

Use `stencila themes tokens --scope site` to inspect exact names for families such as:

- `layout`
- `nav-menu`
- `nav-tree`
- `nav-groups`
- `toc-tree`
- `prev-next`
- `logo`
- `title`
- `site-search`
- `copyright`
- `social-links`
- `edit-on`
- `edit-source`
- `copy-markdown`
- `site-review`
- `breadcrumbs`

## Important naming quirks

- The component family is `title`, but exact site title tokens are prefixed `--site-title-*`.
- The component family is `site-search`, but exact search token names are prefixed `--search-*`.
- Logo image URLs are often set via inline custom properties such as `--logo`, `--logo-dark`, and `--logo-mobile`, not only through ordinary theme tokens.
- Breadcrumbs are a site navigation surface even if some source material lists them under node-token paths.

## Small exact-name anchor set

These names are useful examples, but the CLI remains the source of truth for complete inventories:

- `--layout-header-height`
- `--header-background`
- `--header-border-color`
- `--nav-menu-color`
- `--nav-menu-color-hover`
- `--nav-menu-color-active`
- `--nav-menu-item-padding-x`
- `--site-title-font-size`
- `--site-title-font-weight`
- `--site-title-color`
- `--search-modal-width`
- `--search-highlight-color`
- `--breadcrumbs-link-color`

## Review cautions

- Review site themes as site chrome plus document shell, not just article-body styling.
- Check responsive behavior, focus states, and navigation wrapping when header or navigation tokens change.
- Do not approve guessed exact names when a concrete patch depends on them.
- If the site theme also needs PDF support, review PDF behavior separately because site token coverage does not imply PDF parity.
