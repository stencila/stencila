# `IncludeBlock` execution examples

## `files.md`

Has several `IncludeBlock`s that include files in this directory, in subdirectories and in sibling directories. Tests that paths to included files are relative to the directory of the including file.

## `http.md`

Has an `IncludeBlock` that includes a file fetched from GitHub. Tests that the `source` property of `IncludeBlock` nodes can be HTTP URLs.
