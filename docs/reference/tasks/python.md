<!-- Generated from Taskfile. Do not edit. -->

# `python`: Tasks related to `python`

## Includes

Other `Taskfile`s included:

- [`apt`](apt.md)
- [`asdf`](asdf.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which Python related tasks are needed

Adds the `python:install` task if there are any Python files in the
project (including in subdirectories).

#### Command

```sh
test $(find . -type f -iname '*.py' -print -quit) && echo python:install >> .stencila/tasks/detected
```

### <a id='install'>`install`</a> : Install Python for the project

Installs Python at the version specified (or the latest version if none specified).
The version installed will be added to the project's `.tool-versions` file.

#### Command

1. [`install-{{OS}}`](#install-{{OS}}) `VERSION={{.VERSION}}`

### <a id='install-linux'>`install-linux`</a> : Install Python on Debian-based Linux

#### Commands

1. [`apt:install-packages`](apt.md#install-packages) `PACKAGES=make build-essential libssl-dev zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev wget curl llvm libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev `

2. [`asdf:add`](asdf.md#add) `PACKAGE=python` `VERSION={{.VERSION}}`

### <a id='install-darwin'>`install-darwin`</a> : Install Python on MacOS

#### Commands

1. [`brew:add`](brew.md#add) `PACKAGES=openssl readline sqlite3 xz zlib tcl-tk`

2. [`asdf:add`](asdf.md#add) `PACKAGE=python` `VERSION={{.VERSION}}`

### <a id='upgrade'>`upgrade`</a> : Upgrade Python to the latest version

Installs the latest version of Python and adds it to the project's `.tool-versions` file.

#### Command

1. [`install`](#install) `VERSION=latest`
