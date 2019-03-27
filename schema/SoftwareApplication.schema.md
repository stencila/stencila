# `SoftwareApplication`

## Extends

`SoftwareApplication` extends `CreativeWork`.

## About

`SoftwareApplication` represents a software application that can be located by name and version. It may be fetched in binary format or some package manager (npm, pip, etc) could fetch and compile the source code.

## Parsing/Interpretation

Although by using the `softwareRequirements` property it is possible to represent a full hierarchy of required packages, normally it is adequate to only list the immediate dependencies and allow the system/language package manager to resolve the rest.