# Stencila Skills

An implementation of the [Agent Skills Specification](https://agentskills.io/specification).

## Usage

Skills are directories containing a `SKILL.md` file placed in `<workspace>/.stencila/skills/`:

```
.stencila/skills/
├── data-analysis/
│   └── SKILL.md
└── code-review/
    └── SKILL.md
```

### CLI

```sh
# List all skills in the current workspace
stencila skills

# Show details about a specific skill
stencila skills show data-analysis

# Validate a skill directory
stencila skills validate .stencila/skills/data-analysis
```

### Programmatic

```rust
// List skills in a workspace
let skills = stencila_skills::list(workspace_path).await;

// Get a specific skill by name
let skill = stencila_skills::get(workspace_path, "data-analysis").await?;

// Serialize to XML for context injection
let xml = stencila_skills::to_xml(&skill);

// Compact metadata XML for progressive disclosure
let metadata_xml = stencila_skills::metadata_to_xml(&skills);
```

## Extensions

The following extensions to the spec are implemented for better interoperability with Stencila:

- **`allowed-tools` as array**: The spec's space-delimited `allowed-tools` string is parsed into a `Vec<String>` for easier programmatic use.

- **`license` maps to `licenses`**: The spec's singular `license` field maps to the inherited `licenses` array from `CreativeWork` (the `license` alias is handled by serde).

- **`metadata` translation**: Stencila stores `CreativeWork` properties like `authors`, `version`, and `licenses` as flat, top-level fields on the `Skill` struct. The spec nests these under a `metadata:` object in frontmatter. Translation happens in both directions: on decode, entries under `metadata:` are hoisted to the top level so they populate CreativeWork fields; on encode, non-spec top-level fields are nested back under `metadata:` so the output conforms to the spec.

- **`node_type` hint in `DecodeOptions`**: The Stencila Markdown decoder accepts a `node_type` hint to parse `SKILL.md` files as `Skill` nodes without requiring `type: Skill` in the frontmatter.

## Deviations

These are intentional deviations from the spec:

- **Workspace-scoped**: Skills live in `.stencila/skills/` within a workspace rather than a global directory. This keeps skills project-local.

- **ASCII-only names**: The spec says "unicode lowercase alphanumeric characters" but the parenthetical character class `(a-z and -)` and all examples use ASCII only. We enforce ASCII `[a-z0-9-]` and measure length in bytes (equivalent to character count for ASCII). This may be relaxed if the upstream spec clarifies Unicode intent.

## Limitations

The following are known limitations of this implementation:

- Optional directories (`scripts/`, `references/`, `assets/`) are not explicitly modeled; they are accessible via relative paths from the skill's home directory.

## Development

### Updating the spec

A vendored copy of the spec is kept in `specs/` for reference. Use the protocol below when upstream changes.

1. Preview upstream changes without mutating the repo:

```sh
make spec-diff
```

2. Vendor the latest spec:

```sh
make update-spec
```

3. Generate the repo diff for review and PR context:

```sh
git --no-pager diff -- specs/agent-skills.md
```

4. Convert spec diffs into implementation work:

- Add or update failing tests in the matching `tests/spec_*.rs` file(s) first.
- Implement the minimum code changes in `src/` until tests pass.
- Keep deferred subsections explicit in `## Limitations` if any gaps remain.

### Testing

```sh
cargo fmt -p stencila-skills
cargo clippy --fix --allow-dirty --all-targets -p stencila-skills
cargo test -p stencila-skills
```
