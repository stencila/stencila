This example shows how different types of outputs when using the Stencila R microkernel.

Because R is "vectorized", to return just a single value, for example to pass on to other microkernels use the `unbox` function. For example, this output is an `Array` node:

```r exec
# @impure
42
```

But this one is an `Integer`:

```r exec
# @impure
unbox(42)
```

Text printed to stdout is output as a single `Text` node:

```r exec
# @impure
cat("First\nSecond\n")
```

The `print` function is overridden to allow output of nodes (rather than a blob of `Text`). For example, here we output an `Array` of integers.

```r exec
plot(rnorm(1000))
```

But if you want to bypass this you can always use `base::print` directly:

```r exec
Sys.sleep(10)
```
