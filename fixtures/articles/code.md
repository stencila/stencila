This article fixture is focussed on the Markdown representation of executable code nodes such as `CodeChunk`, `CodeExpression`, and `Parameter` nodes.

## Inline code

Code expressions have a language and the `exec` keyword in curly braces, like this `1+1`{r exec} and this `2+2`{python exec}.

Non-executable code fragments, lack the `exec` keyword but can have a language e.g. `3+3`{r}.

## Block code

Code chunk use the `exec` keyword to differentiate them from code blocks,

```r exec
"Hello from R"
```

Non executable code blocks do not have the `exec` keyword,

```python
# Not executed
```
