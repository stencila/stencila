<!-- Generated from Taskfile. Do not edit. -->

# `renv`: Tasks related to `renv`

## Includes

Other `Taskfile`s included:

- [`gitignore`](gitignore.md)
- [`r`](r.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which `renv` related tasks are needed

Adds the `renv:install` task if there is a `renv.lock` file.

#### Command

```sh
test -f renv.lock && echo renv:install >> .stencila/tasks/detected
```

### <a id='ensure'>`ensure`</a> : Ensure `renv` is installed

Checks whether the R package `renv` is installed and installs it if necessary.

#### Commands

1. [`r:install`](r.md#install)

2. `R --slave -e "install.packages('renv', repos = 'https://cloud.r-project.org')"`

### <a id='gitignore'>`gitignore`</a> : Update `.gitignore` for use with `renv`

Adds a line to the local `.gitignore` file (creating one if necessary) so that
the `renv` folder is ignored by Git.

#### Command

1. [`gitignore:add`](gitignore.md#add) `PATTERNS=renv`

### <a id='init'>`init`</a> : Initialize `renv` in a directory

#### Commands

1. [`ensure`](#ensure)

2. [`gitignore`](#gitignore)

3. `R --slave -e "renv::init(repos = c( ...`

### <a id='add'>`add`</a> : Add an R package using `renv`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `R --slave -e "renv::install('{{.PACKAGE}}'); renv::snapshot(type='all')"`

### <a id='remove'>`remove`</a> : Remove an R package using `renv`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. `R --slave -e "renv::remove('{{.PACKAGE}}'); renv::snapshot(type='all')"`

### <a id='install'>`install`</a> : Install all R packages in `renv.lock` using `renv`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `R --slave -e "renv::install()"`

### <a id='update'>`update`</a> : Update all R packages in `renv.lock` using `renv`

#### Commands

1. [`ensure`](#ensure)

2. [`init`](#init)

3. [`gitignore`](#gitignore)

4. `R --slave -e "renv::update()"`

### <a id='clean'>`clean`</a> : Delete package installation artifacts created by `renv`

Removes the `renv` directory.

#### Command

```sh
rm -rf renv
```

### <a id='purge'>`purge`</a> : Remove `renv` and associated files from the project

Deletes `renv.lock` and `renv`.

#### Commands

1. `rm -f renv.lock`

2. [`clean`](#clean)
