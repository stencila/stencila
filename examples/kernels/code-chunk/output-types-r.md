The `r` kernel supports outputting most primitive node types. The `print` function is patched so that each argument is a separate output:

```r exec
print(TRUE, 1, 2.34, "string", c(1, 2, 3), list(a=1, b=2))
```

There is also support for outputting of data frames:

```r exec
head(mtcars)
```

Base graphics plots:

```r exec
plot(mpg~cyl, data=mtcars)
```

As well as `ggplot` plots:

```r exec
library(ggplot2)

ggplot(mtcars, aes(x=cyl, y=mpg, color=carb)) + geom_point()
```
