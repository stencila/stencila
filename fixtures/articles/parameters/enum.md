An example article mainly intended for testing that enum parameters can be set in different kernels.

This enum parameter `a`: /a/{enum vals=\["Apples","Pears","Oranges"] def="Apples"} should be echoed in the following `CodeChunk`s using different kernels:

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
