# Python and Stencila

## 👋 Introduction

This folder groups the python code for the Stencila project.
The code is organized in the following folders:

- `stencila_types`: Python types generated from the schema-gen crate, and associated shortcuts and utilities. This package is required by the following two packages.
- `stencila`: This rust-extension for python (using pyo3) that exposes some of the core Stencila API to python.
- `stencila_plugin`: This is a plugin to aid writing Stencila plugins in Python. It provides a base classes for implementing plugins for kernels (other APIs are coming).

Please see the individual folders for more information.

## 🛠️ Develop

We use [pdm](https://pdm-project.org/latest/) for managing dependencies.
Installation instructions are [here](https://pdm-project.org/latest/#installation).

We develop using the lowest version of Python that is supported by Stencila, which is Python 3.10.
You will need a copy of Python 3.10 installed on your system where `pdm` can find it (pyenv, or conda, or a system installation).
`make install` in this folder will create a virtual environment for each of the packages.

## Release

This is currently manually done for the python packages.

In addition, we use the [pdm-bump](https://github.com/carstencodes/pdm-bump) plugin to bump versions.
After installing `pdm`, run:

```bash
pdm plugin add pdm-bump
```
