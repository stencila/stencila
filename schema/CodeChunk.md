---
title: CodeChunk
---

# Structure

## Source properties

A `CodeChunk` has two 'source' properties, `text` and `language`, from which it's other properties are derived during compilation (e.g. `import`, `declares`) or execution (e.g. `outputs`).

### `text` and `language`

All `CodeChunk`s are required to have a `text` property, containing the actual code, and most will also specify the programming `language`. If `language` is not specified then it defaults to the language of the previous `CodeChunk` or `CodeExpression` in the document. So, if you are writing a document that only uses one language, you only need to  `For more on these properties see [`Code`](./Code.html).

An example of a simple `CodeChunk`,

```json import=helloWorld
{
  "type": "CodeChunk",
  "language": "python",
  "text": "print('Hello world')"
}
```

## Compilation properties

The following properties of a `CodeChunk` are populated during compilation. You can also override the compiler and specify them manually.

### `imports`

The `imports` property lists the packages that a `CodeChunk` imports into the execution context. It is used by the compiler to populate the `requires` property of the document containing the chunk so that it can build an execution environment for it (e.g. a Docker container).

- dokta regex style
- AST walking

```json import=importPython
{
  "type": "CodeChunk",
  "language": "python",
  "text": "import matplotlib"
}
```

```yaml import=importJavascript
type: CodeChunk
language: javascript
text: |
  import * as d3 from 'd3'
  const kmeans = require('ml-kmeans');
imports:
  - d3
  - ml-kmeans
```

You can manually add packages to the `imports` property of a `CodeChunk`. When the chunk is compiled, only missing packages are added to required. If for some reason you want full control and do not want the compiler to add anything to `imports`, set the first value to the empty string.

For example, in R you can call a function in a package without first importing via a call to `library`:

```yaml import=importJavascript
type: CodeChunk
language: r
text: |
  superdoopa::func("beep", "boop")
imports:
  - ''
  - superdoopa
```

[//] TODO: Reallife example with dplyr or something

### `declares`

The `declares` property lists the variables that a `CodeChunk` declares. It is used by the compiler to build a graph of the dependencies among `CodeChunk`s and `CodeExpressions`. This in turn allows for reactivity. When a user changes a chunk containing a declaration, all of the other chunks or expressions that use that variable will be rexecuted.

```yaml import=pythonFunction
type: CodeChunk
language: python
text: |
  def greet(who: str):
    return 'Hello %s!' % who
declares:
  - type: Function
    name: greet
    parameters:
      - type: Parameter
        name: who
        schema:
          - type: StringSchema
```

### Code comment overrides

#### R

In R code chunk properties can be specified using the `@property` tags in comments:

```r
#' @imports package1, package2
#' @uses variable1, variable2
#' @declares variable3, variable4
#' @alters variable5, variable6
```

One situation in R where you may want to use a comment override is to enable reactivity for a chunk that uses a function with [non-standard evaluation](http://adv-r.had.co.nz/Computing-on-the-language.html), for example `subset`, or many of the functions in the `tidyverse`. The Stencila R compiler ignores some of the arguments supplied to these functions because of the peculiar scoping rules. e.g.

```r
#' @uses begin, end
data <- subset(all_data, year >= begin && year <= end)
```

See `stencila::nse_funcs` for a list of registered NSE functions.

Another situation is where you have a function that reads files from the file system but is not one of the registered "readers" (see `stencila::reader_funcs` for a list of registered reader functions). In this case, supply a comma separated list of the files read e.g.

```r
#' @reads ./data/all.csv, ./data/categories.csv
all_data <- special_read_csv('./data/all.csv', './data/categories.csv')
```


## Execution properties

The following properties of a `CodeChunk` are populated during execution.

## `outputs`

When a code chunk is executed...

[//] TODO:

## `errors`

[//] TODO:

## `duration`

[//] TODO:
