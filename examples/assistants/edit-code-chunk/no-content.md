---
# Test using `edit-code-chunk` in a document with no other content
---

::: do @edit-code-chunk add comments
::: with

```r exec
data <- read.csv('data.csv')
plot(height~age, data)
```

:::


::: do @edit-code-chunk simplify
::: with

```r exec
data <- read.table(
    'data.csv',
    header = TRUE,
    sep = ','
)
```

:::


::: do @edit-code-chunk color points by species
::: with

```r exec
ggplot(data, aes(x=age, y=height)) + geom_point()
```

:::
