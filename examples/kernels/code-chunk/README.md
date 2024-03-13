# `CodeChunk` execution examples

## `hello-world-*.md`

Has a single `CodeChunk` that outputs the `String` "Hello world!".

## `output-types-*.md`

Has a `CodeChunk`s that output various types of nodes. Useful for testing the rendering of outputs.

## `sleeps-*.md`

Has several `CodeChunk`s which sleep for increasingly longer durations. Useful for testing incremental status updates during execution of a document and that execution of `CodeChunks` can be interrupted (while executing) or cancelled (before execution starts).
