<!-- Generated from Taskfile. Do not edit. -->

# `pip`: Tasks related to `pip`

## Includes

Other `Taskfile`s included:

- [`python`](python.md)
- [`venv`](venv.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which Pip related tasks are needed

Adds the `pip:install` task if there is a `requirements.txt` file in the root
of the project.

#### Command

```sh
test -f requirements.txt && echo pip:install >> .stencila/tasks/detected
```

### <a id='ensure'>`ensure`</a> : Ensure Pip is installed

Pip is a popular Python package manager. This task ensures that Pip
(and by extension, Python) is installed. It checks that the `pip` module is
installed, and if not, installs it using the `ensurepip` module.

#### Commands

1. [`python:install`](python.md#install)

2. `python -m ensurepip`

### <a id='add'>`add`</a> : Add a Python package using Pip

#### Commands

1. [`venv:init`](venv.md#init)

2. `if [ -z {{.VERSION}} ] ...`

3. `.venv/bin/pip freeze > requirements.txt`

### <a id='remove'>`remove`</a> : Remove a Python package using Pip

#### Commands

1. [`venv:init`](venv.md#init)

2. `.venv/bin/pip uninstall {{.PACKAGE}}`

3. `.venv/bin/pip freeze > requirements.txt`

### <a id='install'>`install`</a> : Install all Python packages in `requirements.txt` using Pip

#### Sources

- `requirements.txt`

#### Commands

1. [`venv:init`](venv.md#init)

2. `.venv/bin/pip install -r requirements.txt`

### <a id='clean'>`clean`</a> : Delete package installation artifacts created by Pip

Removes the `.venv` directory.

#### Command

1. [`venv:clean`](venv.md#clean)

### <a id='purge'>`purge`</a> : Remove Pip and associated files from the project

Deletes `.venv`. Does NOT remove `python` from `.tool-versions` or delete
`requirements.txt`.

#### Command

1. [`clean`](#clean)
