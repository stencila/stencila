<!-- Generated from Taskfile. Do not edit. -->

# `git`: Tasks related to `git`

## Includes

Other `Taskfile`s included:

- [`apt`](apt.md)

## Tasks

### <a id='install'>`install`</a> : Install Git

Checks for `git` on the `PATH` and installs it if it is not.

#### Command

1. [`install-{{OS}}`](#install-{{OS}})

### <a id='install-linux'>`install-linux`</a> : Install Git on Debian-based Linux using Apt

#### Command

1. [`apt:install-packages`](apt.md#install-packages) `PACKAGES=git`

### <a id='install-darwin'>`install-darwin`</a> : Install Git on MacOS using Homebrew

#### Command

1. [`brew:add`](brew.md#add) `PACKAGES=git`
