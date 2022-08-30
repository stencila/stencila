An example article mainly intended for testing that changes to arrays generate HTML patches that are properly applied to `Array` nodes in both block and inline contexts. Try adding, deleting, replacing and moving items.

In `CodeChunk`s an array output is represented as an `<ol>` with `<li>`s:

```js exec
;[1, 2, 3]
```

In `CodeExpression`s an array output is represented as nested `<span>`s: `[1,2,3, 4,5]`{js exec}.
