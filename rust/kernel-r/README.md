# Stencila R Microkernel

This is a Stencila 'microkernel' for R.

## Installation

On Linux requires the `Cairo` package which in turn requires `libcairo2` and `libcairo2-dev`. In the future, `Cairo` might be made optional, but for now it is necessary for creating a kernel in forks.

## Development

During development it can be useful to maually test / debug the microkernel. You should be able to type lines of R code and get back results. Use `FORK` to create a fork e.g.:

```console
> Rscript src/r-kernel.r
READY
READY
1 + 1
[2]RESULT
TASK
TASK
plot(1)
{"type":"ImageObject","contentUrl":"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAk4AAAJOCAMAAAB2h3jHAAAABlBMVEUAAAD///+l2Z/dAAAACXBIWXMAABcRAAAXEQHKJvM
...
AjJiZCcCMmJkJwIyYmQnAjJiZCcCMmJkJwI/QfMbTv5figlIgAAAABJRU5ErkJggg=="}RESULT
TASK
TASK
FORK
26937TASK
```
