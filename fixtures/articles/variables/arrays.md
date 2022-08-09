An example article mainly intended for testing that array variables can be passed between different kernels.

# Setting

These code blocks set array variables in each kernel:

```bash exec
bash="[1,2,3]"
```

```js exec
js = [1, 2, 3]
```

```py exec
py = [1, 2, 3]
```

```r exec
r <- 1:3
```

```zsh exec
zsh="[1,2,3]"
```

# Getting

These code blocks get the variables from the other kernels:

```js exec
;({ bash, js, py, r, zsh })
```

```py exec
dict(bash=bash, js=js, py=py, r=r, zsh=zsh)
```

```r exec
list(bash=bash, js=js, py=py, r=r, zsh=zsh)
```
