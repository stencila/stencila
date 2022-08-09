An example article mainly intended for testing that string variables can be passed between different kernels.

# Setting

These code blocks set string variables in each kernel:

```bash exec
bash="Bash"
```

```js exec
let js = 'JavaScript'
```

```py exec
py = "Python"
```

```r exec
r <- "R"
```

```zsh exec
zsh="ZSH"
```

# Getting

These code blocks get the variables from the other kernels:

```bash exec
echo "$bash, $js, $py, $r, $zsh"
```

```js exec
;[bash, js, py, r, zsh]
```

```py exec
[bash, js, py, r, zsh]
```

```r exec
c(bash, js, py, r, zsh)
```

```zsh exec
echo "$bash, $js, $py, $r, $zsh"
```
