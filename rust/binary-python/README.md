# 📦 Stencila buildpack for Python

## Detection

Matches against a project that has:

  - a `.tool-versions` file with a `python` entry, or

  - any of a `pyproject.toml`, `runtime.txt`, `poetry.lock`, `requirements.txt`, `main.py`, or `index.py` in its root folder

## Python version

The version of Node.js to be installed is determined by (in descending order of precedence):

  - the `python` entry of any `.tool-versions` file,

  - the content of any `runtime.txt` file,

  - the `tool.poetry.dependencies.python` property of any `pyproject.toml`, or else

  - the latest version of Python.

## PyPI packages

PyPi packages are installed into a virtual environment folder (usually `.venv`) within the project folder. Which PyPI packages and their versions to install is determined by (in descending order of precedence):

  - if a `pyproject.toml` or `poetry.lock` file is present, then `poetry install` will be used to install the versions of packages specified in those files into the local `.venv` folder (see the Poetry [docs](https://python-poetry.org/docs/cli/#install) for more on the exact behavior).

  - if a `requirements.txt` file is present, then a `pip install` will be used to install the versions of packages in that file into the virtual environment. If a Python virtual environment named `.venv`, `venv`, or `env` already exists then it will be used, otherwise a new `.venv` folder will be created.
