An example article mainly intended for testing that boolean variables can be passed between different kernels.

# Setting

These code blocks set boolean variables in each kernel:

```js exec
jsTrue = true
jsFalse = false
```

```py exec
py_true = True
py_false = False
```

```r exec
r_true <- TRUE
r_false <- FALSE
```

# Getting

These code blocks get the variables from the other kernels

```js exec
;[py_true, py_false, r_true, r_false]
```

```py exec
[r_true, r_false, jsTrue, jsFalse]
```

```r exec
c(py_true, py_false, jsTrue, jsFalse)
```
