An example article mainly intended for testing that integer parameters can be set in different kernels.

This integer parameter `a`: /a/{int def=6 min=0 max=100} should have three added to it in the following `CodeChunk`s using different kernels:

```bash exec
expr $a + 3
```

```calc exec
a + 3
```

```js exec
a + 3
```

```py exec
a + 3
```

```r exec
unbox(a + 3)
```

```zsh exec
expr $a + 3
```
