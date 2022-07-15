<!-- Generated from Taskfile. Do not edit. -->

# `pnpm`: Tasks related to `pnpm`

## Includes

Other `Taskfile`s included:

- [`asdf`](asdf.md)
- [`gitignore`](gitignore.md)
- [`node`](node.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which PNPM related tasks are needed

Adds the `pnpm:install` task if there is a `package.json` file.

#### Command

```sh
test -f package.json && echo npm:install >> .stencila/tasks/detected
```

### <a id='ensure'>`ensure`</a> : Ensure PNPM is installed

#### Commands

1. [`node:install`](node.md#install)

2. [`asdf:add`](asdf.md#add) `PACKAGE=pnpm`

### <a id='gitignore'>`gitignore`</a> : Update `.gitignore` for use with PNPM

Adds a line to the local `.gitignore` file (creating one if necessary) so that
the `node_modules` folder is ignored by Git.

#### Command

1. [`gitignore:add`](gitignore.md#add) `PATTERNS=node_modules`

### <a id='init'>`init`</a> : Initialize PNPM in a directory

Checks for a `package.json` file, and if none exists, runs `pnpm init`.

#### Commands

1. [`ensure`](#ensure)

2. [`gitignore`](#gitignore)

3. `pnpm init`

### <a id='add'>`add`</a> : Add a Node.js package using PNPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `pnpm add {{.PACKAGE}}@{{.VERSION | default "latest"}}`

### <a id='remove'>`remove`</a> : Remove a Node.js package using PNPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. `pnpm remove {{.PACKAGE}}`

### <a id='install'>`install`</a> : Install all Node.js packages in `package.json` using PNPM

#### Sources

- `package.json`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `pnpm install`

### <a id='update'>`update`</a> : Update all Node.js packages in `package.json` using PNPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `pnpm update`

### <a id='clean'>`clean`</a> : Delete package installation artifacts created by PNPM

Removes the `node_modules` directory.

#### Command

```sh
rm -rf node_modules
```

### <a id='purge'>`purge`</a> : Remove PNPM and associated files from the project

Removes `pnpm` from `.tool-versions` and deletes `pnpm-lock.yaml` and `node_modules`.
Does NOT remove `nodejs` from `.tool-versions` or delete `package.json`.

#### Commands

1. `rm -f pnpm-lock.yaml`

2. [`clean`](#clean)

3. [`asdf:remove`](asdf.md#remove) `PACKAGE=pnpm`
