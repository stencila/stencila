<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->


# Execution

Stencila documents can contain executable code as part of the document itself. This makes it possible to combine narrative writing with analysis, computation, generated outputs, and reusable executable document components.

> [!tip]
> If you only want to show code without running it, see [Code](code).

# Executable code chunks

Add the `exec` flag to a fenced code block to make it executable:

````smd
```python exec
# Some python code
a = 3
```
````

When Stencila executes a code chunk, it runs the code using an appropriate kernel for the language and attaches any resulting outputs to the document. Those outputs can include text, tables, figures, and other rendered results.

You can also omit the language:

````smd
```exec
No language
```
````

# Execution modes

Execution modes can be added after `exec`:

````smd
```js exec always
// Javascript
```
````

Execution modes control when or whether a chunk runs. For example, `always` indicates that the chunk should be rerun whenever execution is performed, while modes such as `lock` are useful when you want to preserve existing outputs instead of recomputing them automatically.

# Inline code expressions

Inline code expressions let you embed computed values directly in prose:

```smd
With a number output `6 * 7`{python exec}.
```

These are useful for values that should stay synchronized with your analysis, such as counts, parameter values, summary statistics, or short derived strings that appear inside narrative text.

```smd
With a string output `'a string'`{python exec}.
```

```smd
With an array output `[1, 2, 3]`{python exec}.
```

```smd
With an object output `dict(a=1,b=dict())`{python exec}.
```

You can also omit the language or specify an execution mode:

```smd
With no language specified and no output `a + b`{exec}.
```

```smd
With execution mode specified and no output `c * d`{javascript exec lock}.
```

# Labelled chunks

Executable chunks can be wrapped in a labelled chunk block:

````smd
::: chunk 1A

```exec
# Chunk 1A
```

:::
````

# Executable figures and tables

Executable code can also be wrapped in figures and tables so that outputs have captions and labels:

````smd
::: figure 1

```r exec
plot(y~x)
```
Y against X.

:::
````

````smd
::: table 1

```r exec
head(mtcars)
```
Some cars.

:::
````

# Parameters

Parameters define values that can be supplied to executable documents:

```smd
A boolean parameter &[par_bool_1]{bool}, with a default &[par_bool_2]{bool def=true}, with a default and value &[par_bool_3]{bool def=false}
```

Parameters are especially useful when a document is intended to be reused with different inputs. Other documents can then provide those values using [Includes and calls](include-call), allowing one executable document to act like a reusable, parameterized component.

> [!tip]
> For includes and document calls, see [Includes and calls](include-call).

> [!tip]
> If you ended up here but only need static code examples, language-tagged code fragments, or ordinary fenced code blocks, see [Code](code).

