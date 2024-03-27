# `ForBlock` execution examples

## `standalone-*.md`

For each kernel, has a several standalone `ForBlock`s which iterate over different value types. Tests how different values are treated as an iterator, that the `content` is repeated in `iterations`, and that executable nodes with each iteration are executed with the `variable` set to the correct value.


## `assigned-*.md`

For each kernel, assigns an array in a `CodeChunk` and then iterates over it in the `ForBlock`. Tests that assigned variables are available to the expression of a `ForBlock`, that the `content` is repeated in `iterations`, and that executable nodes with each iteration are executed with the `variable` set to the correct value.

## `nested-python.md`

Has a `ForBlock` nested within another. Tests that this executes as expected.

## `otherwise-js.md`

Has a `ForBlock` with an `otherwise` property - content that will be shown if there are no items in the iterator. Tests that the otherwise property is compiled, and executed, if there are no items.
