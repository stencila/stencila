# Stencila extensions to ASTRA

This document records experimental ASTRA fields recognized by Stencila before
they are part of the upstream ASTRA schema. These extensions are proposals, not
ASTRA v1 fields. A manifest using them may therefore be rejected by the
upstream `astra validate` command until the corresponding schema change is
accepted.

## `Output.target`

Status: experimental Stencila extension, intended for a future upstream ASTRA
pull request.

`target` is an optional URI or path where a locally produced output is
materialized:

```yaml
outputs:
  - id: penguins_dataset
    type: data
    target: penguins.csv
    inputs: [adelie, gentoo, chinstrap]
    recipe:
      command: python download.py
```

A relative path is resolved from the directory containing the output's
`astra.yaml`. It must remain inside the workspace. A URI denotes a non-workspace
destination. Re-exported outputs using `from` cannot override `target`; the
target is inherited from the original output.

### Why it is useful

ASTRA output IDs identify logical research artifacts, while workflow systems
and source code usually identify concrete files or URIs. Without an explicit
mapping, a consumer cannot safely know that an output such as
`penguins_dataset` is the file `penguins.csv`, even when a recipe happens to
write that file.

`target` makes that association declarative. Stencila can consequently:

- connect the logical ASTRA output to its concrete materialization;
- converge ASTRA, workflow, and static-analysis evidence on the same graph
  relationship;
- distinguish an output's identity and provenance from the command used to
  produce it; and
- retain a useful locator for an expected output even before it exists.

The field belongs to `Output`, not `Recipe`, because ASTRA is asset-centric:
the output owns its provenance and materialization, while the recipe remains
purely how the output is produced.

### Why the name is `target`

The name deliberately complements `Input.source`:

- `Input.source` is the URI or path from which an input is obtained.
- `Output.target` is the URI or path where an output is materialized.

`path` would imply a local filesystem location and would fit URIs poorly.
`destination` would emphasize transfer or execution rather than the artifact.
Within the context of an `Output`, `target` concisely covers both local paths
and remote URIs without moving materialization concerns into the recipe.

### Stencila graph projection

Stencila retains the logical ASTRA output node. When `target` resolves to a
concrete graph resource, it also projects:

- `WrittenTo` from the logical output to a local target, or `SentTo` to a remote
  target;
- `Generated` from the ASTRA workflow unit to a local target, or `SentTo` to a
  remote target; and
- for a conservatively recognized direct script recipe, `Generated` or
  `SentTo` from that script to the target.

Those relationships carry `Declared` graph evidence from `astra.yaml`.
Recipe-to-target evidence points to `recipe.command` and records the target;
logical materialization evidence points directly to `target`. This allows the
same concrete generation edge to merge independent evidence from ASTRA,
workflow declarations such as a Snakefile, and source-code static analysis.
