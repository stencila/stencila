<!-- Generated from Taskfile. Do not edit. -->

# `asdf`: Tasks related to `asdf`

## Includes

Other `Taskfile`s included:

- [`curl`](curl.md)
- [`git`](git.md)

## Template variables

- `ASDF`: `$HOME/.asdf/bin/asdf`

## Tasks

### <a id='detect'>`detect`</a> : Detect which `asdf` tasks are needed for the project

Adds the `asdf:install` task if there is an `.tool-versions` file in the root of the project.

#### Command

```sh
test -f .tool-versions && echo asdf:install >> .stencila/tasks/detected
```

### <a id='ensure'>`ensure`</a> : Ensure `asdf` is installed

Checks that `asdf` is installed (and on `PATH`) and installs it using `git` if it is not.

#### Commands

1. [`curl:install`](curl.md#install)

2. [`git:install`](git.md#install)

3. `git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v{{.VERSION}}`

### <a id='ensure-plugin'>`ensure-plugin`</a> : Ensure an `asdf` plugin is installed

#### Commands

1. [`ensure`](#ensure)

2. `{{.ASDF}} plugin add {{.NAME}} {{.URL}}`

### <a id='ensure-package'>`ensure-package`</a> : Ensure an `asdf` package version is installed

#### Commands

1. [`ensure-plugin`](#ensure-plugin) `NAME={{.PACKAGE}}`

2. `{{.ASDF}} install {{.PACKAGE}} {{.VERSION | default "latest"}}`

### <a id='add'>`add`</a> : Add a tool using `asdf`

Ensures that the corresponding `asdf` plugin is installed and that
the tool version is installed.

#### Commands

1. [`ensure-package`](#ensure-package) `PACKAGE={{.PACKAGE}}` `VERSION={{.VERSION}}`

2. `{{.ASDF}} local {{.PACKAGE}} {{.VERSION | default "latest"}}`

### <a id='remove'>`remove`</a> : Remove a tool using `asdf`

Removes the tool from `.tool-versions` but does not uninstall it
from the `asdf` directory.

#### Command

```sh
sed -i '/{{.PACKAGE}} /d' .tool-versions
```

### <a id='install'>`install`</a> : Install all tools listed in `.tool-versions` using `asdf`

For each tool listed in `.tool-versions`, ensures that the corresponding `asdf`
plugin is installed, and that the tool version itself is installed.

#### Commands

1. [`ensure`](#ensure)

2. `if [ -f .tool-versions ] ...`

3. `{{.ASDF}} install`
