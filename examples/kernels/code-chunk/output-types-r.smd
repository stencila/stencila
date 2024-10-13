The `r` kernel supports outputting most primitive node types. The `print` function is patched so that it outputs values as Stencila nodes:

```r exec
print(TRUE)
print(1)
print(2.34)
print("string")
print(c(1, 2, 3))
print(list(a=1, b=2))
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
