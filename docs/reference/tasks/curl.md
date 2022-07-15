<!-- Generated from Taskfile. Do not edit. -->

# `curl`: Tasks related to `curl`

## Includes

Other `Taskfile`s included:

- [`apt`](apt.md)
- [`brew`](brew.md)

## Tasks

### <a id='install'>`install`</a> : Install Curl

Checks for `curl` on the `PATH` and installs it if it is not.

#### Command

1. [`install-{{OS}}`](#install-{{OS}})

### <a id='install-linux'>`install-linux`</a> : Install Curl on Debian-based Linux using Apt

#### Command

1. [`apt:install-packages`](apt.md#install-packages) `PACKAGES=curl`

### <a id='install-darwin'>`install-darwin`</a> : Install Curl on MacOS using Homebrew

#### Command

1. [`brew:add`](brew.md#add) `PACKAGES=curl`
