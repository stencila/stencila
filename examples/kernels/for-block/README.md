# Examples with `ForBlock` nodes

## `standalone-*.md`

For each kernel, has a several standalone `ForBlock`s which iterate over different value types. Tests how different values are treated as an iterator, that the `content` is repeated in `iterations`, and that executable nodes with each iteration are executed with the `variable` set to the correct value.


## `assigned-*.md`

For each kernel, assigns an array in a `CodeChunk` and then iterates over it in the `ForBlock`. Tests that assigned variables are available to the expression of a `ForBlock`, that the `content` is repeated in `iterations`, and that executable nodes with each iteration are executed with the `variable` set to the correct value.
