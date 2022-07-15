<!-- Generated from Taskfile. Do not edit. -->

# `poetry`: Tasks related to `poetry`

## Includes

Other `Taskfile`s included:

- [`asdf`](asdf.md)
- [`python`](python.md)
- [`venv`](venv.md)

## Environment variables

- `POETRY_VIRTUALENVS_IN_PROJECT`: `true`

## Tasks

### <a id='detect'>`detect`</a> : Detect which Poetry related tasks are needed

Adds the `poetry:install` task if there is a `pyproject.toml` file in the root
of the project with a `tools.poetry` section.

#### Command

```sh
test -f pyproject.toml && grep -q '^\[tool.poetry\]$' pyproject.toml && echo poetry:install >> .stencila/tasks/detected

```

### <a id='ensure'>`ensure`</a> : Ensure Poetry is installed

Checks whether Poetry is installed, and if not, installs it.

#### Commands

1. [`python:install`](python.md#install) `VERSION=`

2. [`asdf:add`](asdf.md#add) `PACKAGE=poetry` `VERSION=`

### <a id='init'>`init`</a> : Initialize Poetry in a directory

Checks for a `pyproject.toml` file with a `tool.poetry` section, and if none exists, or it not setup
for Poetry, runs `poetry init`.
Ensures that Python is installed in `.tool-version` in the directory.

#### Commands

1. [`ensure`](#ensure)

2. `poetry init --no-interaction`

### <a id='add'>`add`</a> : Add a Python package using Poetry

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`venv:gitignore`](venv.md#gitignore)

4. `poetry add {{.PACKAGE}}@{{.VERSION | default "latest"}}`

### <a id='remove'>`remove`</a> : Remove a Python package using Poetry

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. `poetry remove {{.PACKAGE}}`

### <a id='install'>`install`</a> : Install all Python packages in `pyproject.toml` using Poetry

#### Sources

- `pyproject.toml`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`venv:gitignore`](venv.md#gitignore)

4. `poetry install`

### <a id='update'>`update`</a> : Update all Python packages in `pyproject.toml` using Poetry

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`venv:gitignore`](venv.md#gitignore)

4. `poetry update`

### <a id='clean'>`clean`</a> : Delete package installation artifacts created by Poetry

Removes the `.venv` directory.

#### Command

1. [`venv:clean`](venv.md#clean)

### <a id='purge'>`purge`</a> : Remove Poetry and associated files from the project

Removes `poetry` from `.tool-versions` and deletes `poetry.lock` and `.venv`.
Does NOT delete `pyproject.toml` or remove Poetry sections from it.

#### Commands

1. `rm -f poetry.lock`

2. [`clean`](#clean)

3. [`asdf:remove`](asdf.md#remove) `PACKAGE=poetry`
