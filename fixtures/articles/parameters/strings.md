An example article mainly intended for testing that string parameters can be set in different kernels.

This string parameter `a`: /a/{str def="Hello world"} should be echoed in the following `CodeChunk`s using different kernels:

```bash exec
echo $a
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
