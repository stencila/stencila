# Contributing to Stencila

Stencila welcomes 💖 all types of contributions! If you've spotted a small typo in the documentation 📖, have a fix for bug 🐛, an idea for a new feature ✨, or anything else that will make Stencila better, we'd love to hear from you.

This document provides an overview of how to make different types of contributions to the project. It includes links to issues and discussions on GitHub and to other `CONTRIBUTING.md` files in sub-projects of Stencila.


## Committing messages

Commit messages should follow the [conventional commits](https://www.conventionalcommits.org/) specification. This is useful convention to follow because commit messages are used to determine the semantic version of releases and to generate the project's [CHANGELOG.md](https://github.com/stencila/thema/blob/next/CHANGELOG.md). We use sentence case for the commit's "scope" and "subject" to make both `git log` and the CHANGELOG more readable. Some examples,

- `chore(Rust crates): Exclude experimental crates`
- `ci(Python): Install numpy`
- `feat(Form): Add support, in various places, for Form nodes`
- `fix(Patches): Apply _many macros for patchable enum variants`
- `perf(Patching): Push ops directly to differ`
- `refactor(Prop tests): Use a consistent order of a-zA-Z in patterns`
- `test(HTML codec): Fix snapshot`

## Code organization

Most code lives in sub-project sub-directories of the root directory of the repository (e.g. `rust`, `docs`). Most of the sub-projects have their own `CONTRIBUTING.md` which should describe the code organization within that sub-project.

This section describes the organization of code in the root directory and non-sub-project sub-directories (e.g. `.ci`, `.github`). Most of the files in these locations is for management of the repository as a whole including commit linting, dependency management, continuous integration and semantic release numbering.

### `package.json`

This file is for configuring JavaScript based tooling used for repository-wide commit linting, dependency management, and semantic release numbering.

#### `commitlint` property

We use [`commitlint`](https://github.com/conventional-changelog/commitlint) and [`husky`](https://github.com/typicode/husky) to check that commit messages conform to the [conventional commits](https://www.conventionalcommits.org/) specification as described above.

