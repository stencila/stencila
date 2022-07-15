<!-- Generated from Taskfile. Do not edit. -->

# `npm`: Tasks related to `npm`

## Includes

Other `Taskfile`s included:

- [`gitignore`](gitignore.md)
- [`node`](node.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which NPM related tasks are needed

Adds the `npm:install` task if there is a `package.json` file.

#### Command

```sh
test -f package.json && echo npm:install >> .stencila/tasks/detected
```

### <a id='ensure'>`ensure`</a> : Ensure NPM is installed

#### Command

1. [`node:install`](node.md#install)

### <a id='gitignore'>`gitignore`</a> : Update `.gitignore` for use with NPM

Adds a line to the local `.gitignore` file (creating one if necessary) so that
the `node_modules` folder is ignored by Git.

#### Command

1. [`gitignore:add`](gitignore.md#add) `PATTERNS=node_modules`

### <a id='init'>`init`</a> : Initialize NPM in a directory

Checks for a `package.json` file, and if none exists, runs `npm init`.

#### Commands

1. [`ensure`](#ensure)

2. `npm init --yes`

### <a id='add'>`add`</a> : Add a Node.js package using NPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `npm install --save {{.PACKAGE}}@{{.VERSION | default "latest"}}`

### <a id='remove'>`remove`</a> : Remove a Node.js package using NPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. `npm uninstall --save {{.PACKAGE}}`

### <a id='install'>`install`</a> : Install all Node.js packages in `package.json` using NPM

#### Sources

- `package.json`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `npm install`

### <a id='update'>`update`</a> : Update all Node.js packages in `package.json` using NPM

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `npm update`

### <a id='clean'>`clean`</a> : Delete package installation artifacts created by NPM

Deletes the `node_modules` directory.

#### Command

```sh
rm -rf node_modules
```

### <a id='purge'>`purge`</a> : Remove files associated with NPM from the project

Deletes `package-lock.json` and `node_modules`.
Does NOT remove `nodejs` from `.tool-versions` or delete `package.json`.

#### Commands

1. `rm -f package-lock.json`

2. [`clean`](#clean)
