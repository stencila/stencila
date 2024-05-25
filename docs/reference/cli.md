# Command-Line Help for `stencila`

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila import`↴](#stencila-import)
* [`stencila export`↴](#stencila-export)
* [`stencila sync`↴](#stencila-sync)
* [`stencila convert`↴](#stencila-convert)
* [`stencila compile`↴](#stencila-compile)
* [`stencila execute`↴](#stencila-execute)
* [`stencila serve`↴](#stencila-serve)
* [`stencila assistants`↴](#stencila-assistants)
* [`stencila assistants list`↴](#stencila-assistants-list)
* [`stencila assistants execute`↴](#stencila-assistants-execute)
* [`stencila kernels`↴](#stencila-kernels)
* [`stencila kernels list`↴](#stencila-kernels-list)
* [`stencila kernels info`↴](#stencila-kernels-info)
* [`stencila kernels packages`↴](#stencila-kernels-packages)
* [`stencila kernels execute`↴](#stencila-kernels-execute)
* [`stencila kernels evaluate`↴](#stencila-kernels-evaluate)
* [`stencila plugins`↴](#stencila-plugins)
* [`stencila plugins list`↴](#stencila-plugins-list)
* [`stencila plugins install`↴](#stencila-plugins-install)
* [`stencila plugins uninstall`↴](#stencila-plugins-uninstall)
* [`stencila plugins link`↴](#stencila-plugins-link)
* [`stencila plugins enable`↴](#stencila-plugins-enable)
* [`stencila plugins disable`↴](#stencila-plugins-disable)
* [`stencila plugins show`↴](#stencila-plugins-show)
* [`stencila plugins check`↴](#stencila-plugins-check)
* [`stencila secrets`↴](#stencila-secrets)
* [`stencila secrets list`↴](#stencila-secrets-list)
* [`stencila secrets set`↴](#stencila-secrets-set)
* [`stencila secrets delete`↴](#stencila-secrets-delete)
* [`stencila config`↴](#stencila-config)
* [`stencila upgrade`↴](#stencila-upgrade)
* [`stencila uninstall`↴](#stencila-uninstall)

## `stencila`

CLI subcommands and global options

**Usage:** `stencila [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `new` — Create a new document
* `import` — Import a file in another format into a new or existing document
* `export` — Export a document to a file in another format
* `sync` — Synchronize a document with one of more other files in other formats
* `convert` — Convert a document between formats
* `compile` — Compile a document
* `execute` — Execute a document
* `serve` — Serve
* `assistants` — Manage assistants
* `kernels` — Manage execution kernels
* `plugins` — Manage plugins
* `secrets` — Manage secrets used by Stencila (e.g. API keys)
* `config` — 
* `upgrade` — Upgrade to the latest version
* `uninstall` — Uninstall this command line tool

###### **Options:**

* `--log-level <LOG_LEVEL>` — The minimum log level to output

  Default value: `info`

  Possible values: `trace`, `debug`, `info`, `warn`, `error`

* `--log-filter <LOG_FILTER>` — A filter for log entries

  Default value: `globset=warn,hyper=info,ignore=warn,mio=info,notify=warn,ort=error,reqwest=info,tokio=info,tungstenite=info`
* `--log-format <LOG_FORMAT>` — The log format to use

  Default value: `auto`

  Possible values: `auto`, `simple`, `compact`, `pretty`, `full`, `json`

* `--error-details <ERROR_DETAILS>` — The details to include in error reports

  Default value: `auto`
* `--error-link` — Output a link to more easily report an issue

  Possible values: `true`, `false`




## `stencila new`

Create a new document

**Usage:** `stencila new [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` — The path of the document to create

###### **Options:**

* `-s`, `--source <SOURCE>` — The source file to import from
* `-f`, `--format <FORMAT>` — The format of the source file
* `--codec <CODEC>` — The codec to use to decode the source
* `-o`, `--overwrite` — Overwrite the document if it already exists

  Possible values: `true`, `false`




## `stencila import`

Import a file in another format into a new or existing document

**Usage:** `stencila import [OPTIONS] <DOC> <SOURCE>`

###### **Arguments:**

* `<DOC>` — The path of the document to create or import to
* `<SOURCE>` — The source file to import from

###### **Options:**

* `-f`, `--from <FROM>` — The format of the source file
* `-l`, `--losses <LOSSES>` — What to do if there are losses when decoding

  Default value: `warn`
* `--strip-scopes <STRIP_SCOPES>` — Scopes defining which properties of nodes should be stripped

  Possible values:
  - `authors`:
    Strip authorship properties of nodes
  - `provenance`:
    Strip provenance properties of nodes
  - `metadata`:
    Strip metadata properties of nodes
  - `content`:
    Strip content properties of nodes
  - `code`:
    Strip code properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila export`

Export a document to a file in another format

**Usage:** `stencila export [OPTIONS] <DOC> [DEST]`

###### **Arguments:**

* `<DOC>` — The path of the document to export from
* `<DEST>` — The destination file to export to

###### **Options:**

* `-t`, `--to <TO>` — The format of the destination file
* `-l`, `--losses <LOSSES>` — What to do if there are losses when encoding

  Default value: `warn`
* `--standalone` — Encode as a standalone document

  Possible values: `true`, `false`

* `--not-standalone` — Do not encode as a standalone document when writing to file

  Possible values: `true`, `false`

* `-c`, `--compact` — Use compact form of encoding if possible

  Possible values: `true`, `false`

* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

  Possible values: `true`, `false`

* `--strip-scopes <STRIP_SCOPES>` — Scopes defining which properties of nodes should be stripped

  Possible values:
  - `authors`:
    Strip authorship properties of nodes
  - `provenance`:
    Strip provenance properties of nodes
  - `metadata`:
    Strip metadata properties of nodes
  - `content`:
    Strip content properties of nodes
  - `code`:
    Strip code properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila sync`

Synchronize a document with one of more other files in other formats

**Usage:** `stencila sync [OPTIONS] <DOC> [FILES]...`

###### **Arguments:**

* `<DOC>` — The path of the document to synchronize
* `<FILES>` — The files to synchronize with

###### **Options:**

* `-f`, `--format <FORMATS>` — The formats of the files (or the name of codecs to use)
* `-l`, `--losses <LOSSES>` — What to do if there are losses when either encoding or decoding between any of the files

  Default value: `warn`
* `--standalone` — Encode as a standalone document

  Possible values: `true`, `false`

* `--not-standalone` — Do not encode as a standalone document when writing to file

  Possible values: `true`, `false`

* `-c`, `--compact` — Use compact form of encoding if possible

  Possible values: `true`, `false`

* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

  Possible values: `true`, `false`

* `--strip-scopes <STRIP_SCOPES>` — Scopes defining which properties of nodes should be stripped

  Possible values:
  - `authors`:
    Strip authorship properties of nodes
  - `provenance`:
    Strip provenance properties of nodes
  - `metadata`:
    Strip metadata properties of nodes
  - `content`:
    Strip content properties of nodes
  - `code`:
    Strip code properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila convert`

Convert a document between formats

**Usage:** `stencila convert [OPTIONS] [INPUT] [OUTPUT]`

###### **Arguments:**

* `<INPUT>` — The path of the input file
* `<OUTPUT>` — The path of the output file

###### **Options:**

* `-f`, `--from <FROM>` — The format to encode from (or codec to use)
* `-t`, `--to <TO>` — The format to encode to (or codec to use)
* `-i`, `--input-losses <INPUT_LOSSES>` — What to do if there are losses when decoding from the input

  Default value: `warn`
* `-o`, `--output-losses <OUTPUT_LOSSES>` — What to do if there are losses when encoding to the output

  Default value: `warn`
* `--standalone` — Encode as a standalone document

  Possible values: `true`, `false`

* `--not-standalone` — Do not encode as a standalone document when writing to file

  Possible values: `true`, `false`

* `-c`, `--compact` — Use compact form of encoding if possible

  Possible values: `true`, `false`

* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

  Possible values: `true`, `false`

* `--strip-scopes <STRIP_SCOPES>` — Scopes defining which properties of nodes should be stripped

  Possible values:
  - `authors`:
    Strip authorship properties of nodes
  - `provenance`:
    Strip provenance properties of nodes
  - `metadata`:
    Strip metadata properties of nodes
  - `content`:
    Strip content properties of nodes
  - `code`:
    Strip code properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila compile`

Compile a document

**Usage:** `stencila compile [OPTIONS] <INPUT> [OUTPUT]`

###### **Arguments:**

* `<INPUT>` — The path of the file to execute
* `<OUTPUT>` — The path of the file to write the compiled document to

###### **Options:**

* `-t`, `--to <TO>` — The format to encode to (or codec to use)



## `stencila execute`

Execute a document

**Usage:** `stencila execute [OPTIONS] <INPUT> [OUTPUT]`

###### **Arguments:**

* `<INPUT>` — The path of the file to execute
* `<OUTPUT>` — The path of the file to write the executed document to

###### **Options:**

* `-t`, `--to <TO>` — The format to encode to (or codec to use)
* `--force-all` — Re-execute all node types regardless of current state

  Possible values: `true`, `false`

* `--skip-code` — Skip executing code

  Possible values: `true`, `false`

* `--skip-instructions` — Skip executing instructions

  Possible values: `true`, `false`

* `--force-unreviewed` — Re-execute instructions with suggestions that have not yet been reviewed

  Possible values: `true`, `false`

* `--force-accepted` — Re-execute instructions with suggestions that have been accepted

  Possible values: `true`, `false`

* `--skip-rejected` — Skip re-executing instructions with suggestions that have been rejected

  Possible values: `true`, `false`

* `--dry-run` — Prepare, but do not actually perform, execution tasks

  Possible values: `true`, `false`




## `stencila serve`

Serve

**Usage:** `stencila serve [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory to serve

  Default value: `.`

###### **Options:**

* `-a`, `--address <ADDRESS>` — The address to serve on

  Default value: `127.0.0.1`
* `-p`, `--port <PORT>` — The port to serve on

  Default value: `9000`
* `--raw` — Should files be served raw?

  Possible values: `true`, `false`

* `--source` — Should `SourceMap` headers be sent?

  Possible values: `true`, `false`

* `--sync <SYNC>` — Whether and in which direction(s) to sync served documents

  Possible values: `in`, `out`, `in-out`




## `stencila assistants`

Manage assistants

**Usage:** `stencila assistants [COMMAND]`

###### **Subcommands:**

* `list` — List the assistant available
* `execute` — Execute an instruction with an assistant



## `stencila assistants list`

List the assistant available

**Usage:** `stencila assistants list`



## `stencila assistants execute`

Execute an instruction with an assistant

Mainly intended for quick testing of assistants during development.

**Usage:** `stencila assistants execute <NAME> <INSTRUCTION>`

###### **Arguments:**

* `<NAME>` — The name of the assistant to execute the instruction
* `<INSTRUCTION>` — The instruction to execute



## `stencila kernels`

Manage execution kernels

**Usage:** `stencila kernels [COMMAND]`

###### **Subcommands:**

* `list` — List the kernels available
* `info` — Get information about a kernel
* `packages` — List packages available to a kernel
* `execute` — Execute code in a kernel
* `evaluate` — Evaluate a code expression in a kernel



## `stencila kernels list`

List the kernels available

**Usage:** `stencila kernels list`



## `stencila kernels info`

Get information about a kernel

Mainly used to check the version of the kernel runtime and operating system for debugging purpose.

**Usage:** `stencila kernels info <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the kernel to get information for



## `stencila kernels packages`

List packages available to a kernel

Mainly used to check libraries available to a kernel for debugging purpose.

**Usage:** `stencila kernels packages <NAME> [FILTER]`

###### **Arguments:**

* `<NAME>` — The name of the kernel to list packages for
* `<FILTER>` — A filter on the name of the kernel



## `stencila kernels execute`

Execute code in a kernel

Creates a temporary kernel instance, executes one or more lines of code, and returns any outputs and execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels execute <NAME> <CODE>`

###### **Arguments:**

* `<NAME>` — The name of the kernel to execute code in
* `<CODE>` — The code to execute



## `stencila kernels evaluate`

Evaluate a code expression in a kernel

Creates a temporary kernel instance, evaluates the expression in it, and returns the output and any execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels evaluate <NAME> <CODE>`

###### **Arguments:**

* `<NAME>` — The name of the kernel to evaluate code in
* `<CODE>` — The code expression to evaluate



## `stencila plugins`

Manage plugins

**Usage:** `stencila plugins [COMMAND]`

###### **Subcommands:**

* `list` — List plugins
* `install` — Install a plugin
* `uninstall` — Uninstall a plugin
* `link` — Link to a local plugin
* `enable` — Enable a plugin
* `disable` — Disable a plugin
* `show` — Show details of a plugin
* `check` — Check a plugin



## `stencila plugins list`

List plugins

**Usage:** `stencila plugins list [OPTIONS]`

###### **Options:**

* `-r`, `--refresh` — Force refresh of plugin manifests

  Possible values: `true`, `false`

* `--installed` — Only list installed plugins

  Possible values: `true`, `false`

* `--installable` — Only list installable plugins

  Possible values: `true`, `false`

* `-o`, `--outdated` — Only list installed but outdated plugins

  Possible values: `true`, `false`

* `-e`, `--enabled` — Only list installed and enabled plugins

  Possible values: `true`, `false`




## `stencila plugins install`

Install a plugin

**Usage:** `stencila plugins install <NAME>`

###### **Arguments:**

* `<NAME>` — The name or URL of the plugin to install



## `stencila plugins uninstall`

Uninstall a plugin

**Usage:** `stencila plugins uninstall <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to uninstall



## `stencila plugins link`

Link to a local plugin

**Usage:** `stencila plugins link <DIRECTORY>`

###### **Arguments:**

* `<DIRECTORY>` — The directory to link to



## `stencila plugins enable`

Enable a plugin

**Usage:** `stencila plugins enable <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to enable



## `stencila plugins disable`

Disable a plugin

**Usage:** `stencila plugins disable <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to disable



## `stencila plugins show`

Show details of a plugin

**Usage:** `stencila plugins show <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to install



## `stencila plugins check`

Check a plugin

**Usage:** `stencila plugins check <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to install



## `stencila secrets`

Manage secrets used by Stencila (e.g. API keys)

**Usage:** `stencila secrets [COMMAND]`

###### **Subcommands:**

* `list` — List the secrets used by Stencila
* `set` — Set a secret used by Stencila
* `delete` — Delete a secret previously set using Stencila



## `stencila secrets list`

List the secrets used by Stencila

**Usage:** `stencila secrets list`



## `stencila secrets set`

Set a secret used by Stencila

You will be prompted for the secret

**Usage:** `stencila secrets set <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila secrets delete`

Delete a secret previously set using Stencila

**Usage:** `stencila secrets delete <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila config`

**Usage:** `stencila config [OPTIONS]`

###### **Options:**

* `--dir <DIR>`

  Default value: `config`

  Possible values: `config`, `cache`, `assistants`, `plugins`, `kernels`

* `--ensure`

  Possible values: `true`, `false`




## `stencila upgrade`

Upgrade to the latest version

**Usage:** `stencila upgrade [OPTIONS]`

###### **Options:**

* `-f`, `--force` — Perform upgrade even if the current version is the latest

  Possible values: `true`, `false`

* `-c`, `--check` — Check for an available upgrade but do not install it

  Possible values: `true`, `false`




## `stencila uninstall`

Uninstall this command line tool

**Usage:** `stencila uninstall`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

