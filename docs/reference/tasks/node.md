<!-- Generated from Taskfile. Do not edit. -->

# `node`: Tasks related to `node`

## Includes

Other `Taskfile`s included:

- [`asdf`](asdf.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which Node.js related tasks are needed

Adds the `node:install` task if there are any JavaScript files in the
project (including in subdirectories).

#### Command

```sh
test $(find . -type f -iname '*.js' -print -quit) && echo node:install >> .stencila/tasks/detected
```

### <a id='install'>`install`</a> : Install Node.js for the project

Installs Node.js at the version specified (or latest version none specified).
The version installed will be added to `.tool-versions`.

#### Command

1. [`asdf:add`](asdf.md#add) `PACKAGE=nodejs` `VERSION={{.VERSION}}`
