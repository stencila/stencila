---
title: Lint SVG Tool
description: Tool for statically analyzing SVG overlays in Stencila agent workflows.
---

The `lint_svg` tool statically analyzes SVG overlays containing Stencila (`s:*`) custom elements for layout, reference, and attribute errors. It catches issues before rendering, removing visual guesswork from the overlay authoring process.

Use `lint_svg` after authoring or modifying an SVG overlay to check for problems like overlapping labels, dangling anchor references, components outside the viewBox, and invalid attributes.

# Parameters

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `svg_content` | string | yes | The SVG overlay source string containing `s:*` custom elements to lint |

# Lint rules

The linter checks three categories of rules:

## Spatial rules

| Rule | Level | Description |
| ---- | ----- | ----------- |
| Text/text collision | Warning | Two component labels have overlapping bounding boxes |
| Text/line collision | Warning | A component label overlaps a line, arc, or connector from another component |
| Out-of-bounds | Warning | A component label or line extends outside the `viewBox` |
| Anchor crowding | Warning | Three or more components are positioned within 5px of each other |

Bounding boxes for labels are estimated using character-count heuristics (the same multipliers the compiler uses for layout). Structural elements like lines and arcs do not check against each other -- only label-vs-label and label-vs-line collisions are flagged.

Components excluded from collision checks: `spotlight`, `halo` (these are overlays by design).

## Semantic rules

| Rule | Level | Description |
| ---- | ----- | ----------- |
| Dangling reference | Error | A component references an anchor (via `from`, `to`, or `at`) that does not exist |
| Unused anchor | Warning | An `<s:anchor>` is defined but never referenced by any component |
| Missing namespace | Warning | The `<svg>` element is missing the `xmlns:s="https://stencila.io/svg"` declaration |

## Attribute rules

| Rule | Level | Description |
| ---- | ----- | ----------- |
| Unknown attribute | Warning | A component has an attribute not recognized for that component type |
| Invalid enum value | Warning | An attribute has a value outside its valid set (e.g. `curve="wobbly"` on `<s:arrow>`) |

# Suppressing warnings

To suppress collision warnings for a specific component, place an XML comment immediately before it:

```xml
<!-- lint-ignore collision -->
<s:badge x="100" y="50" label="Intentionally overlapping"/>
```

This suppresses both text/text and text/line collision warnings for that element.

# Output

Returns a JSON object with:

- **`status`** -- `"ok"` when no issues are found, `"issues_found"` otherwise
- **`count`** -- number of diagnostic messages (present when issues found)
- **`messages`** -- array of diagnostic objects, each with `level` (`"Error"` or `"Warning"`) and `message` (human-readable description)

Example output:

```json
{
  "status": "issues_found",
  "count": 2,
  "messages": [
    {
      "level": "Error",
      "message": "<s:callout> references anchor '#peak' which does not exist"
    },
    {
      "level": "Warning",
      "message": "Label collision: <s:badge> and <s:badge> labels overlap"
    }
  ]
}
```

# Typical workflow

1. Author or modify the SVG overlay
2. Run `lint_svg` to check for issues
3. Fix any errors and warnings
4. Use `snap` for final visual verification
