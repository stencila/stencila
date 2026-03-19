---
name: software-dependency-maintenance
description: Update, audit, and maintain package dependencies across monorepo workspaces. Use when packages need upgrading, outdated dependencies need reviewing, lockfiles or caches need cleaning, or dependency conflicts need resolving. Discovers the project's package managers and workspace structure, audits outdated dependencies, researches changelogs and breaking changes, performs targeted or bulk upgrades, cleans lockfiles and caches when needed, and verifies that builds, lints, and tests continue to pass after every change. Works with NPM workspaces, Cargo workspaces, and other monorepo tooling.
keywords:
  - dependency update
  - dependency upgrade
  - package update
  - package upgrade
  - npm update
  - npm install
  - npm outdated
  - cargo update
  - outdated packages
  - monorepo
  - npm workspaces
  - cargo workspaces
  - package-lock.json
  - node_modules
  - lockfile
  - semver
  - breaking changes
  - changelog
  - release notes
  - dependency audit
  - dependency conflict
  - version resolution
  - clean install
  - cache clean
  - maintenance
  - not new features
  - not refactoring
  - not test creation
allowed-tools: read_file write_file edit_file apply_patch glob grep shell web_fetch ask_user
---

## Overview

Update, audit, and maintain package dependencies across monorepo workspaces. This skill covers the full dependency maintenance lifecycle: auditing what is outdated, researching whether upgrades are safe, performing the upgrades, cleaning lockfiles and caches when needed, and verifying that everything still builds and passes tests.

The core principles are:

- **Safety first.** Every dependency change must leave builds passing and tests green. If an upgrade breaks something, revert it immediately and report the issue.
- **Discover first, prescribe only as fallback.** Adapt to whatever package managers, workspace layouts, and build systems the project already uses.
- **One change at a time.** Upgrade dependencies individually or in small related groups so that failures are easy to diagnose and revert.
- **Research before upgrading.** Check changelogs and release notes for breaking changes before bumping a major version. Do not blindly upgrade.

## Required Inputs

This skill requires the following information to operate:

| Input                  | Required | Description                                                                 |
|------------------------|----------|-----------------------------------------------------------------------------|
| Task description       | Yes      | What the user wants done (e.g., "update vite to latest", "review all outdated packages", "clean install") |
| Target packages        | No       | Specific packages, workspaces, or crates to focus on (if omitted, audit the entire project) |
| Version constraints    | No       | Specific version targets or constraints (e.g., "vite ^6", "don't go past react 18") |
| Build/test commands    | No       | Commands to verify the project after changes (if omitted, discover from the build system) |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

After completing its work, this skill reports:

| Output                 | Description                                                                  |
|------------------------|------------------------------------------------------------------------------|
| Changes made           | List of dependencies updated, added, or removed with old → new versions      |
| Files modified         | List of manifest and lockfiles changed                                       |
| Verification result    | Build and test commands run and their outcomes                                |
| Skipped upgrades       | Dependencies that were not upgraded, with reasons (breaking changes, incompatibilities, user constraints) |
| Recommendations        | Any follow-up actions the user should consider (e.g., major version upgrades that need code changes) |

## Steps

### 1. Gather inputs and context

Ensure the required inputs are available before proceeding:

- If the task description is vague, use `ask_user` to clarify what the user wants (audit only, update specific packages, update everything, clean install, etc.)
- Parse target packages into a list of workspaces, crates, or directories to focus on
- Note any version constraints the user has specified

### 2. Discover project structure and package managers

Systematically discover how the project manages dependencies. Do not assume any particular toolchain — let the project tell you.

#### 2a. Identify package managers and workspace layout

- Use `glob` to search for build and configuration files at the project root and in subdirectories:
  - **NPM/Node**: `package.json`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `.npmrc`, `.nvmrc`
  - **Rust/Cargo**: `Cargo.toml`, `Cargo.lock`
  - **Python**: `pyproject.toml`, `requirements.txt`, `poetry.lock`, `uv.lock`, `setup.py`, `setup.cfg`
  - **Other**: `go.mod`, `go.sum`, `Gemfile`, `Gemfile.lock`, `mix.exs`, `pom.xml`, `build.gradle*`
- Read the root manifest files to understand workspace configuration:
  - NPM workspaces: check `workspaces` field in root `package.json`
  - Cargo workspaces: check `[workspace]` section in root `Cargo.toml`
  - PNPM workspaces: check `pnpm-workspace.yaml`
  - Yarn workspaces: check `workspaces` field in root `package.json` or `.yarnrc.yml`

#### 2b. Discover build and test commands

- Look for `Makefile`, `justfile`, `taskfile.yml`, or scripts in `package.json` to find the project's standard build, lint, and test commands
- Read relevant portions to identify:
  - How to build the project (or specific packages)
  - How to run tests (or specific test suites)
  - How to run linters or type checks
  - Any pre-install or post-install hooks
- If `Makefile` targets exist, prefer them over raw commands — they often include necessary setup steps

#### 2c. Understand dependency structure

- For NPM workspaces: identify which packages depend on which, and whether dependencies are hoisted to the root or workspace-local
- For Cargo workspaces: check for `[workspace.dependencies]` in the root `Cargo.toml` (shared dependency versions) and per-crate `Cargo.toml` files
- Note any dependency version pinning, overrides, or resolutions configured in the project

### 3. Audit outdated dependencies

Run the appropriate commands to discover what is outdated:

#### 3a. NPM ecosystem

- Run `npm outdated` (or `yarn outdated`, `pnpm outdated`) to list packages with available updates
- Parse the output to categorize updates:
  - **Patch updates** (e.g., 1.2.3 → 1.2.4): Generally safe, bug fixes only
  - **Minor updates** (e.g., 1.2.3 → 1.3.0): New features, should be backward-compatible
  - **Major updates** (e.g., 1.2.3 → 2.0.0): Potentially breaking, require research

#### 3b. Cargo ecosystem

- Run `cargo outdated` if available, or compare versions in `Cargo.toml` against crates.io
- Check for `cargo audit` output if security advisories are relevant

#### 3c. Other ecosystems

- Python: `pip list --outdated` or check `uv` / `poetry` equivalents
- Go: `go list -m -u all`
- Ruby: `bundle outdated`

#### 3d. Present the audit

If the user asked for a review or audit (not an immediate upgrade), present the findings:

- Organize by severity (major → minor → patch)
- For each outdated dependency, show: current version, latest version, update type (patch/minor/major)
- Highlight any with known security advisories
- Ask the user which dependencies to upgrade before proceeding

### 4. Research upgrade paths

Before performing any major version upgrade, research the changes:

#### 4a. Check changelogs and release notes

- Use `web_fetch` to retrieve changelogs from:
  - GitHub releases page: `https://github.com/<owner>/<repo>/releases`
  - NPM package page: `https://www.npmjs.com/package/<name>`
  - The package's CHANGELOG.md or CHANGES.md (if present in the project's node_modules or fetched from GitHub)
- Focus on:
  - Breaking changes listed in major version releases
  - Migration guides or upgrade instructions
  - Deprecated APIs that need replacement
  - Peer dependency changes

#### 4b. Check for peer dependency impacts

- For NPM packages: check if the upgrade changes peer dependency requirements
- Verify that other packages in the workspace are compatible with the new peer dependency versions
- Note any cascading upgrades that will be needed

#### 4c. Summarize findings

For each researched upgrade, note:

- Whether breaking changes affect the project's usage of the package
- Any code changes required alongside the version bump
- Any peer dependency adjustments needed
- Risk assessment: low (drop-in replacement), medium (minor code adjustments needed), high (significant API changes)

### 5. Perform the upgrades

Apply upgrades one at a time or in small related groups:

#### 5a. Update manifest files

- For NPM: edit `package.json` files directly to update version ranges, then run the install command. Or use `npm install <package>@<version>` for targeted updates.
- For Cargo: edit `Cargo.toml` files (or `[workspace.dependencies]` for workspace-level versions), then run `cargo update -p <crate>`
- For Python: edit `pyproject.toml`, `requirements.txt`, or use the package manager's upgrade command

#### 5b. Regenerate lockfiles

- After updating manifests, run the install command to regenerate the lockfile:
  - NPM: `npm install`
  - Yarn: `yarn install`
  - PNPM: `pnpm install`
  - Cargo: `cargo update` (or `cargo generate-lockfile`)
  - Poetry: `poetry lock`

#### 5c. Apply any required code changes

- If the upgrade requires code changes (API migrations, import path changes, configuration updates):
  - Use `grep` to find all usages of the changed API
  - Use `edit_file` to update each usage
  - Follow the migration guide from Step 4a

### 6. Clean install (when needed)

If the user requests a clean install, or if dependency resolution is failing:

#### 6a. Remove caches and generated files

- **NPM**: Delete `node_modules/` directories (root and all workspaces) and `package-lock.json`
  - `find . -name 'node_modules' -type d -prune -exec rm -rf {} +`
  - `rm -f package-lock.json`
  - Optionally clear the npm cache: `npm cache clean --force`
- **Cargo**: `cargo clean` to remove the `target/` directory
  - Only if needed — Cargo's incremental compilation means this is rarely necessary
- **Python**: remove `.venv/`, `__pycache__/`, `.eggs/`, `*.egg-info/`

#### 6b. Reinstall

- Run the appropriate install command from the project root:
  - NPM: `npm install`
  - Yarn: `yarn install`
  - PNPM: `pnpm install`
  - Cargo: `cargo build` (fetches and compiles dependencies)
  - Poetry: `poetry install`

#### 6c. Verify the install succeeded

- Check that the install command completed without errors
- Verify that lockfiles were regenerated
- Check for any peer dependency warnings or resolution conflicts

### 7. Verify builds and tests pass

After every upgrade or group of related upgrades, verify the project still works:

#### 7a. Build verification

- Run the project's build command (discovered in Step 2b)
- For monorepos, build all affected workspaces — an upgrade in a shared dependency may break downstream packages
- If the build fails:
  - Read the error output carefully
  - Determine if the failure is caused by the upgrade (API change, type incompatibility, missing export)
  - If fixable: apply the code change and rebuild
  - If not fixable without significant effort: revert the upgrade and note it in the skipped upgrades output

#### 7b. Lint and type-check verification

- Run linters and type checkers if configured:
  - `make lint` or the project's lint command
  - `npx tsc --noEmit` for TypeScript projects
  - `cargo clippy` for Rust projects
- Fix any new warnings or errors introduced by the upgrade

#### 7c. Test verification

- Run the project's test suite:
  - `make test` or the project's test command
  - For large test suites, run the tests most likely to be affected first, then the full suite
- If tests fail:
  - Identify which test failures are caused by the upgrade
  - If the failures are due to expected behavior changes (e.g., updated snapshot, changed error message): update the tests
  - If the failures indicate a real regression: revert the upgrade and note it in the skipped upgrades output
  - Do not silently disable or skip failing tests

### 8. Present a summary

Output a clear summary including:

- **Changes made** — for each dependency updated:
  - Package name
  - Old version → new version
  - Update type (patch / minor / major)
  - Any code changes that were required alongside the version bump
- **Files modified** — list of all manifest files, lockfiles, and source files changed
- **Verification result** — build, lint, and test commands run and their outcomes
- **Skipped upgrades** — dependencies that were not upgraded, with reasons:
  - Breaking changes that would require significant code refactoring
  - Incompatibilities with other dependencies in the workspace
  - User-specified constraints that prevent the upgrade
- **Recommendations** — follow-up actions for the user to consider:
  - Major version upgrades that were skipped but should be planned
  - Deprecated packages that should be replaced with alternatives
  - Security advisories that need attention
  - Dependency consolidation opportunities (multiple packages providing overlapping functionality)

## Edge Cases

- **Conflicting dependency requirements across workspaces**: When different workspaces in a monorepo require incompatible versions of the same dependency, report the conflict clearly. Suggest resolutions: upgrade the lagging workspace, use npm `overrides` / Cargo `[patch]`, or accept different versions if the package manager supports it.
- **Peer dependency mismatches after upgrade**: When upgrading a package causes peer dependency warnings or errors, identify which other packages need upgrading to satisfy the new peer requirements. Present this as a cascading upgrade plan and ask the user before proceeding.
- **Lockfile merge conflicts**: If the lockfile has unresolved merge conflict markers, delete the lockfile entirely and regenerate it from the manifest files. Do not attempt to manually resolve lockfile conflicts.
- **Private or scoped packages**: Packages published to private registries (`@company/package`) may not be available on public npm. Do not attempt to fetch changelogs from npmjs.com for private packages — check the project's `.npmrc` for registry configuration and note any private packages in the summary.
- **Monorepo with mixed package managers**: Some projects use different package managers for different parts (e.g., npm for the web frontend, Cargo for the backend, pip for Python scripts). Handle each ecosystem independently with its own audit-upgrade-verify cycle.
- **Pre-release or canary versions**: Do not upgrade to pre-release versions (e.g., `2.0.0-beta.1`, `0.0.0-canary.123`) unless the user explicitly requests it. Treat the latest stable release as the upgrade target.
- **Packages with no changelog**: If a package has no discoverable changelog or release notes, check the git commit history on GitHub for the version range. If no information is available, note the risk and ask the user before proceeding with a major upgrade.
- **Build tools vs runtime dependencies**: Treat build tool upgrades (bundlers, compilers, linters) with extra caution — they can change output in subtle ways. Run a full build and test cycle after upgrading build tools, and compare build output if feasible.
- **Lock-step versioned packages**: Some packages must be upgraded together (e.g., `@babel/core` and `@babel/preset-env`, or `react` and `react-dom`). Identify these groups and upgrade them atomically. Do not leave them at mismatched versions.
- **User wants to audit only, not upgrade**: If the task is to review or audit dependencies without making changes, stop after Step 3 (audit) and Step 4 (research). Present findings and recommendations without modifying any files.
- **npm workspaces hoisting issues**: When a dependency is hoisted to the root but a workspace needs a different version, use the workspace-specific `package.json` to pin the version. Be aware that hoisting behavior differs between npm, yarn, and pnpm.
- **Cargo workspace dependency unification**: In Cargo workspaces using `[workspace.dependencies]`, a version bump in the workspace root affects all crates that reference it. Verify all affected crates still compile after the change.
- **Interrupted or partial upgrades**: If a previous upgrade attempt left the project in a broken state (partially updated manifests, missing lockfile, inconsistent versions), diagnose the current state first. A clean install (Step 6) is often the fastest path to recovery.
