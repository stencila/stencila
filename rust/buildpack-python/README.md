# 📦 Stencila buildpack for Python

## Detection

Matches against a project that has in its root folder:

  - a `.tool-versions` file with a `python` entry, or

  - a `runtime.txt` file, or

  - a `pyproject.toml` or `poetry.lock` file, or

  - a `requirements.txt` file, or

  - a `main.py` or `index.py` file.

## Python version

The version of Node.js to be installed is determined by (in descending order of precedence):

  - the `python` entry of any `.tool-versions` file,

  - the `tool.poetry.dependencies.python` entry of the `pyproject.toml` file,

  - the version specified in `runtime.txt`,

  - the latest version of R available for download.

## Python packages

Python packages are installed into a local virtual environment using either:

  - Poetry (`pyproject.toml` or `poetry.lock` is present), or 

  - Pip (if `requirements.txt` is present)
