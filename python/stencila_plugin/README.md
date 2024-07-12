# Stencila Plugin

[![stencila_plugin](https://img.shields.io/pypi/v/stencila_plugin.svg?logo=python&label=stencila_plugin&style=for-the-badge&color=1d3bd1&logoColor=66ff66&labelColor=3219a8)](https://pypi.org/project/stencila_plugin/)

## Introduction

This package provides tools for writing a [Stencila](https://github.com/stencila/stencila) plugin.
Stencila plugins extend the functionality of Stencila, and are dynamically loaded at runtime.
They communicate with the core Stencila app via JSON-RPC over stdin/stdout or HTTP.
This package implements much of the core functionality for communication, and makes writing a plugin as simple as filling out a few python functions.

## ⚡ Usage

This package can be installed via `pip`.

`pip install stencila_plugin`

An example of how to write a plugin using this package can be seen in this [repo](https://github.com/stencila/plugin-python-template).

