# Python and Stencila

This folder groups the python code for the Stencila project.
The code is organized in the following folders:

- `stencila_types`: Python types generated from the schema-gen crate, and associated shortcuts and utilities. This package is required by the following two.
- `stencila`: This rust-extension for python (using pyo3) that exposes some of the core Stencila API to python.
- `stencila_plugin`: This is a plugin to aid writing Stencila plugins in Python. It provides a base classes for implementing plugins for kernels (other APIs are coming).
