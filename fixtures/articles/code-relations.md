This fixture is focussed on dependency relations between `CodeChunk`, `CodeExpression` and `Parameter` nodes within a document. Relations are established by assignment and usage of variables. For simplicity, it only uses `Calc` as a language in code nodes.

A `CodeChunk` that assigns a variable `a`, and which is always autorun when downstream dependents are run. When manual testing you may wish to set `a = now()` to see the effect of this (for deterministic test snapshots this is not done by default).

```calc exec
# @autorun always
a = 1
a
```

Another chunk that uses `a`:

```calc exec
a * 2
```

and some `CodeExpression`s that also use it: `a * 3`{calc exec} and `a * 4`{calc exec}.

A chunk that derives another variable `b` from `a` and never gets autorun (must be explicitly run):

```calc exec
# @autorun never
b = a + 1
b
```

and some expressions that also use it: `b * 1`{calc exec} and `b * 2`{calc exec}.

A `Parameter` that sets a third symbol `c` : /c/{num default=1 min=0 max=10}

And a code chunk that uses all three variables:

```calc exec
a + b + c
```
