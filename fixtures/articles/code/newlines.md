An example article mainly intended for testing that string in `CodeChunks` can contain newlines and that those will be represented in outputs.

# Bash

```bash exec
printf "one\ntwo\nthree"
```

# JavaScript

Using escapes:

```js exec
'one\ntwo\nthree'
```

Using multi-line strings:

```js exec
;`
one
two
three
`
```

# Python

Using escapes:

```py exec
"one\ntwo\nthree"
```

Using multi-line strings:

```py exec
# At present, multi-line strings must be output like so
# (because the Python microkernel determines output value by
# trying to compile the the last line of code)
value = """
one
two
three
"""

value
```

# R

```r exec
unbox("one\ntwo\nthree")
```

# ZSH

```zsh exec
printf "one\ntwo\nthree"
```
