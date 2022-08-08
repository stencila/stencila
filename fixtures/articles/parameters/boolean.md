An example article mainly intended for testing that boolean parameters can be set in different kernels.

This boolean parameter `a`: /a/{bool def=true} should be echoed in the following `CodeChunk`s using different kernels:

```bash exec
echo $a
```

```calc exec
a
```

```js exec
a
```

```py exec
a
```

```r exec
unbox(a)
```

```zsh exec
echo $a
```
