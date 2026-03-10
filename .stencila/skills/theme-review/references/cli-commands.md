# Theme CLI commands for review

Use the Stencila theme CLI as the live source of truth for builtin token inventories and theme validation.

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
```

## Review guidance

- Use CLI output to confirm what tokens exist now.
- Use the local references in this skill for review criteria, naming quirks, and cross-target caveats.
- When exact names matter, prefer the narrowest useful `--scope` and `--family` filters.
- Treat CLI verification as stronger evidence than memory.
