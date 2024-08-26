An insert instruction with no prompt:

::: new a 4x10 table <

An insert instruction with a prompt:

::: new @table top five highest mountains, with height in metres <

An insert instruction with a prompt but a blank instruction

::: new @code-chunk  <

An edit instruction (has content) with no prompt:

::: edit improve this paragraph >

The paragraph to be improved.

An edit instruction with a prompt:

::: edit @anne please improve this paragraph >

Another paragraph to be improved.

An insert instruction with a prompt and a suggestion:

::: new @code-chunk analyze data <

::: suggest >

```exec
some code
```


An edit instruction with a suggestion:

::: edit more succinct >

A rather long winded paragraph.

::: suggest >

A paragraph.

