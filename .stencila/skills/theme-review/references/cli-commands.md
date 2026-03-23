# Theme CLI commands for review

Use the Stencila theme CLI as the live source of truth for builtin token inventories, theme inspection, and validation during review.

## Listing themes

```sh
# List all available themes (workspace, user, and builtin) with type and location
stencila themes
stencila themes list
```

## Showing a theme

```sh
# Show the default resolved theme CSS (follows resolution order)
stencila themes show

# Show a specific theme by name
stencila themes show tufte

# Show a theme with all resolved CSS variable values
stencila themes show --verbose
stencila themes show stencila --verbose
```

Use `--verbose` to see the final computed value of every variable after base-theme merging and `var()` resolution. This is especially useful during review for understanding what values a theme inherits and what overrides actually change.

## Token discovery

```sh
# List all builtin theme tokens
stencila themes tokens

# Filter by scope
stencila themes tokens --scope semantic
stencila themes tokens --scope node
stencila themes tokens --scope site
stencila themes tokens --scope plot
stencila themes tokens --scope print

# Filter by family within a scope
stencila themes tokens --scope node --family table
stencila themes tokens --scope site --family nav-menu
stencila themes tokens --scope plot --family axis

# Output machine-readable token inventories
stencila themes tokens --scope plot --as json
```

## Validation

```sh
# Validate a theme file
stencila themes validate theme.css

# Fail on unknown tokens
stencila themes validate theme.css --strict

# Output validation result as JSON
stencila themes validate theme.css --as json
```

## Review guidance

- Use `stencila themes list` to see what themes are available and where they live.
- Use `stencila themes show --verbose` to understand the resolved variable values before judging whether overrides are correct or redundant.
- Use CLI output to confirm what tokens exist now.
- Use the local references in this skill for review criteria, naming quirks, and cross-target caveats.
- When exact names matter, prefer the narrowest useful `--scope` and `--family` filters.
- Treat CLI verification as stronger evidence than memory.
