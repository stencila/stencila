An example article mainly intended for testing that number variables can be passed between different kernels.

# Setting

These code blocks set number variables in each kernel:

```calc exec
calc = 12.34
```

```js exec
js = 12.34
```

```py exec
py = 12.34
```

```r exec
r <- 12.34
```

# Getting

These code blocks get the variables from the other kernels:

```calc exec
calc + js + py + r
```

```js exec
;[calc, js, py, r]
```

```py exec
[calc, js, py, r]
```

```r exec
c(calc, js, py, r)
```
