<!-- Generated from Taskfile. Do not edit. -->

# `r`: Tasks related to `r`

## Includes

Other `Taskfile`s included:

- [`apt`](apt.md)
- [`asdf`](asdf.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which R related tasks are needed

Adds the `r:install` task if there are any R files in the project (including in subdirectories).

#### Command

```sh
test $(find . -type f -iname '*.r' -print -quit) && echo r:install >> .stencila/tasks/detected
```

### <a id='install'>`install`</a> : Install R for the project

#### Command

1. [`install-{{OS}}`](#install-{{OS}}) `VERSION={{.VERSION}}`

### <a id='install-linux'>`install-linux`</a> : Install R on Debian-based Linux

#### Commands

1. [`apt:install-packages`](apt.md#install-packages) `PACKAGES=build-essential gfortran libbz2-1.0 libbz2-dev libcurl3-dev liblzma-dev liblzma5 libpcre2-dev libreadline-dev xorg-dev `

2. [`asdf:add`](asdf.md#add) `PACKAGE=R` `VERSION={{.VERSION}}`

### <a id='install-darwin'>`install-darwin`</a> : Install R on MacOS

#### Commands

1. [`brew:add`](brew.md#add) `PACKAGES=gcc xz libxt cairo`

2. [`asdf:add`](asdf.md#add) `PACKAGE=R` `VERSION={{.VERSION}}`
