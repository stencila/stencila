This example is primarily for testing generation, and rendering, of notifications from different kernels. Currently notifications are represented as `CodeError` nodes, in the `errors` property of `CodeChunk`s and `CodeExpression`s. In the future, these are likely to be generalized to `CodeNotification` and `notifications` respectively, given that they also encompass informational and warning notifications.

# Calc

Errors in a code chunk:

```calc exec
1 !*
foo
```

An error in a code expression: `foo`{calc exec}.

# JavaScript

Various notification levels in a code chunk:

```js exec
console.info('An info message')
console.warn('A warning message')
console.error('An error message')
throw new Error('A thrown error')
```

An error in a code expression: `foo`{js exec}.

# Python

Errors in a code chunk:

```py exec
raise RuntimeError("A runtime error")
```

A division-by-zero error in a code expression: `1/0`{py exec}.

# R

Various notification levels in a code chunk:

```r exec
# A custom function in R microkernel for generating an info message
info("An info message")

# The standard R functions for generating warnings and errors...
warning("A warning message")
stop("An error message")
```

A `RuntimeError` in a code expression: `foo`{r exec}.

# Shells

An error from Bash:

```bash exec
foo
```

And one from ZSH:

```zsh exec
foo
```
