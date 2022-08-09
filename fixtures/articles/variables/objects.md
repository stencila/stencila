An example article mainly intended for testing that object variables can be passed between different kernels.

# Setting

These code blocks set object variables in each kernel:

```js exec
js = { a: 1, b: 2 }
```

```py exec
py = {"a": 1, "b": 2}
```

```r exec
r <- list(a=1, b=2)
```

# Getting

These code blocks get the variables from the other kernels:

```js exec
all = { js, py }
all
```

```py exec
dict(js=js, py=py, r=r)
```

```r exec
list(js=js, py=py, r=r)
```
