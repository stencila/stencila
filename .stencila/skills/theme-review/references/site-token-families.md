# Site token family review reference

Use this file for site-theme terminology, component families, exact-name pitfalls, and application guidance that commonly affect review quality.

For a comprehensive current list of builtin site tokens, use the CLI:

```sh
stencila themes tokens --scope site
```

Filter to a family when you need a narrower inventory:

```sh
stencila themes tokens --scope site --family nav-menu
stencila themes tokens --scope site --family breadcrumbs
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope site --as json
```

## Terminology and paths

Keep terminology aligned with the localized docs:

- `layout`
- `nav-menu`
- `nav-tree`
- `nav-groups`
- `toc-tree`
- `prev-next`
- `logo`
- `title` for the component documented as Site Title, with token names prefixed `--site-title-*`
- `site-search`
- `copyright`
- `social-links`
- `edit-on`
- `edit-source`
- `copy-markdown`
- `site-review`
- `breadcrumbs`

Important path note: the localized source listing currently places `breadcrumbs.smd` under node-token material, but the component is still a site breadcrumb/navigation surface. Keep the component name `breadcrumbs` and do not relabel it as a different surface.

## Exact-name guidance that the family list alone does not show

- The component family is `title`, but exact site title token names are prefixed `--site-title-*`.
- The component family is `site-search`, but exact search token names are prefixed `--search-*`.
- Logo image URLs are often set via inline custom properties such as `--logo`, `--logo-dark`, and `--logo-mobile`, not only through ordinary theme tokens.

## Small verified exact-name set

These exact names are useful reference anchors when reviewing without a live CLI query, but prefer the CLI for complete current inventories:

### Layout

- `--layout-header-height`
- `--header-background`
- `--header-border-color`

### Navigation menu

- `--nav-menu-color`
- `--nav-menu-color-hover`
- `--nav-menu-color-active`
- `--nav-menu-item-padding-x`

### Site title

- `--site-title-font-size`
- `--site-title-font-weight`
- `--site-title-color`

### Site search

- `--search-modal-width`
- `--search-highlight-color`

### Logo

- `--logo-height`
- `--logo-max-width`

### Breadcrumbs

- `--breadcrumbs-link-color`
- `--breadcrumbs-link-color-hover`
- `--breadcrumbs-current-color`

## How to use the CLI and this reference together

1. Use `stencila themes tokens --scope site` or `--scope site --family <family>` to get the current token inventory.
2. Verify exact site token names from the CLI output against what the artifact uses.
3. Use this reference for terminology, family boundaries, naming quirks, and the prefix mismatches that commonly cause review errors.
4. If the needed family is not clear, flag the uncertainty instead of approving a guessed token name.

## Review cautions

- Review site themes as site chrome plus document shell, not just article-body styling.
- Check responsive behavior, focus states, and navigation wrapping when header or navigation tokens change.
- Do not approve guessed exact names when a concrete patch depends on them.
- If the site theme also needs PDF support, review PDF behavior separately because site token coverage does not imply PDF parity.
- Flag dark-mode gaps in header, navigation, and search surface tokens when the theme targets web outputs.
- Flag reliance on broad site-chrome selectors when site tokens would be clearer and more stable.

## Validation workflow

When site theming matters, recommend checks such as:

1. render a site preview or HTML output
2. compare header, sidebars, navigation, breadcrumbs, search, and footer surfaces
3. test narrow and wide viewport behavior
4. verify focus states and contrast for keyboard navigation
5. if the site theme also needs PDF support, validate the document/PDF outputs separately because site chrome does not imply PDF parity
6. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail
