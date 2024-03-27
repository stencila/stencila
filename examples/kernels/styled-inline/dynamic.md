A Bash code chunk that assigns the colour variable:

```bash exec
color=pink
```

A Python code chunk that other variables:

```py exec
styles = {
  'bg': 100,
  'txt': 800,
  'pad': 1
}
```

A paragraph with [some dynamically styled content]{bg-$color-{{styles.bg}} text-$color-{{styles.txt}} p-{{styles.pad}}}.

A paragraph with [some content]{bg-$foo-100} that attempts to use some non-existent variable.
