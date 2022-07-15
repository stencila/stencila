<!-- Generated from Taskfile. Do not edit. -->

# `gitignore`: Tasks related to `gitignore`

## Tasks

### <a id='add'>`add`</a> : Add patterns to `.gitignore`

Adds each pattern in the variable `PATTERNS` as a line in the local `.gitignore` file
(if the line is not already present). Creates a `.gitignore` file if necessary.

#### Command

```sh
touch .gitignore
for PATTERN in {{.PATTERNS}}; do
  grep --quiet --line-regexp --fixed-strings $PATTERN .gitignore || echo $PATTERN >> .gitignore
done

```
