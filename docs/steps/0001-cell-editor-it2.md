# CodeEditor - Iteration II

## Goal

Provide a generalized API for functionality needed by the `CodeEditor`. It should work indpendently from Engine and running execution contexts.

This is a preparation for the next iterations, implementing `Function Usage Help`, `Auto-Complete`, etc.

**Note**: it is intentional to use a rudimentary solution for code-analysis at this level, because while editing an expression, it
is very often in an incomplete state. Still the `CodeEditor` should be providing contextual help etc.

## Tasks

- Use PrismJS with customized language definitions to create an `analyseCode()` helper, that gives us a rudimentary code analysis with variabes, cell references, and function calls being detected, at least. I.e. this is a tokenizer plus a very rudimentary parser for extracting funtion call nodes.

```
let { tokens, nodes } = analyzeCode(code, 'mini')
```

`tokens` are turned into `Markers`. A `token` looks like this:
```
{
  type: 'function',
  text: 'sum',
  start: 4,
  end: 7
}
```

`nodes` are the result of the rudimentary parsing. A node for a function call looks like this :

```
{
  type: 'call',
  name: 'sum',
  start: '4',
  end: '21',
  args: [{ start: 5, end: 8 }, { start: 9, end: 15 }, { start: 16, end: 20}]
}
```

- Execute the code analysis just in the `CodeEditor` whenever the expression has changed

> TODO: it is not clear where to store the result, in a universal way so that `Commands` etc. can also access it

- Turn `tokens` to markers for rudimentary syntax-highlighting
