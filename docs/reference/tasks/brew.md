<!-- Generated from Taskfile. Do not edit. -->

# `brew`: Tasks related to `brew`

## Tasks

### <a id='add'>`add`</a> : Add system packages using Homebrew

Install the packages specified in variable `PACKAGES` using `brew`.
At present, does not check whether the packages are installed yet.

#### Command

```sh
brew install {{.PACKAGES}}
```
