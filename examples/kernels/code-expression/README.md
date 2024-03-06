# `CodeExpression` execution examples

## `standalone-*.md`

Has a single `CodeExpression` with no dependencies that outputs an integer. Tests using the backtick syntax for expressions where the programming language needs to be specified.

## `assigned-*.md`

For each kernel, assigns a number variable in a `CodeChunk` and then uses it in a `CodeExpression`. Tests using the double brace "moustaches" syntax for expressions where the programming language is not specified (and thus defaults to using the first kernel in the document).
