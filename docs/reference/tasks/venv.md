<!-- Generated from Taskfile. Do not edit. -->

# `venv`: Tasks related to `venv`

## Includes

Other `Taskfile`s included:

- [`gitignore`](gitignore.md)
- [`python`](python.md)

## Tasks

### <a id='gitignore'>`gitignore`</a> : Update `.gitignore` for use with a Python virtual environment

Adds a line to the local `.gitignore` file (creating one if necessary) so that
the virtual environment is ignored.

#### Command

1. [`gitignore:add`](gitignore.md#add) `PATTERNS=.venv`

### <a id='init'>`init`</a> : Initialize a Python virtual environment is present

Checks that there is a `.venv/bin/python` present and executable in the current directory, and if there is not, creates a new virtual environment in `.venv`.

#### Commands

1. [`python:install`](python.md#install)

2. [`gitignore`](#gitignore)

3. `python3 -m venv .venv`

### <a id='clean'>`clean`</a> : Delete the virtual environment

Removes the `.venv` directory.

#### Command

```sh
rm -rf .venv
```
