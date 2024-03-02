# Examples with `IfBlock` nodes

## `standalone-*.md`

For each kernel, has a standalone `IfBlock` with a single clause, evaluated in a single kernel. Tests that clauses can be evaluated in the kernel, that various value types (e.g. empty strings) are treated as being falsy, and that executable nodes within the `content` of the active clause (in this case the final `else` clause) are also evaluated.

## `standalone-polyglot.md`

Has a standalone `IfBlock` where each clause is evaluated in a different kernel. Tests that a single `IfBlock` can have clauses evaluated in different kernels.

## `assigned-*.md`

For each kernel, assigns an integer in a `CodeChunk` and then uses it in the clauses of an `IfBlock`. Tests that, assigned variables are available to clauses of an `IfBlock`, executable nodes within the `content` of inactive clauses are **not** executed, and that executable nodes within the `content` of the active clause (in this case the final `else` clause) **are** executed.
