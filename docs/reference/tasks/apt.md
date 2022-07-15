<!-- Generated from Taskfile. Do not edit. -->

# `apt`: Tasks related to Apt, the package manager used on Debian-based Linux

This `Taskfile` includes tasks for `add`-ing and `remove`-ing system packages (e.g. `git`) using `apt-get`.
It will also `detect` if there is an `Aptfile` in your project and `install` the system packages declared in it.

Note that, in contrast to most of the other tasks in this library, `add`, `remove` and `install` will alter the
system and as such will use `sudo` (if present).

## Template variables

- `SUDO`: `which sudo || true` (dynamic)

## Environment variables

- `DEBIAN_FRONTEND`: `noninteractive`

## Tasks

### <a id='detect'>`detect`</a> : Detect which Apt tasks are needed for the project

Detects if there is an `Aptfile` in the root of the project and require thee `apt:install` if it is.

#### Command

```sh
test -f Aptfile && echo apt:install >> .stencila/tasks/detected
```

### <a id='add'>`add`</a> : Add a system package using Apt

Installs the system package and adds it the the project's `Aptfile` (creating
one if necessary).

#### Commands

1. `{{.SUDO}} apt-get -y install {{.PACKAGE}}`

2. `touch Aptfile && grep '^{{.PACKAGE}}' Aptfile || echo {{.PACKAGE}} >> Aptfile`

### <a id='remove'>`remove`</a> : Remove a system package using Apt

Uninstalls the system package and removes it from the project's `Aptfile`.

#### Commands

1. `{{.SUDO}} apt-get -y remove {{.PACKAGE}}`

2. `sed -i '/^{{.PACKAGE}}\.*/d' Aptfile`

### <a id='install'>`install`</a> : Install all Apt packages in `Aptfile`

Installs the packages specified on each line of the `Aptfile`.

#### Sources

- `Aptfile`

#### Command

```sh
if [ -f Aptfile ]
then
  cat Aptfile | while read LINE
  do
    dpkg --status $PACKAGE > /dev/null
    INSTALLED=$?
    if [ $INSTALLED -ne 0 ]; then
     {{.SUDO}} apt-get -y install $PACKAGE
    fi
  done
fi

```

### <a id='install-packages'>`install-packages`</a> : Install a list of system packages using Apt

Installs the packages without affecting the project `Aptfile`.
Intended for use by other task to install their system dependencies.

#### Command

```sh
{{.SUDO}} apt-get -y install {{.PACKAGES}}
```
