# Command-Line Help for `stencila`

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila import`↴](#stencila-import)
* [`stencila export`↴](#stencila-export)
* [`stencila sync`↴](#stencila-sync)
* [`stencila log`↴](#stencila-log)
* [`stencila inspect`↴](#stencila-inspect)
* [`stencila convert`↴](#stencila-convert)
* [`stencila execute`↴](#stencila-execute)
* [`stencila serve`↴](#stencila-serve)
* [`stencila assistants`↴](#stencila-assistants)
* [`stencila repl`↴](#stencila-repl)
* [`stencila test`↴](#stencila-test)
* [`stencila kernels`↴](#stencila-kernels)
* [`stencila kernels list`↴](#stencila-kernels-list)
* [`stencila kernels info`↴](#stencila-kernels-info)
* [`stencila kernels packages`↴](#stencila-kernels-packages)
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
* `log` — Display the history of commits to the document
* `inspect` — Inspect a document as JSON
* `convert` — Convert a document between formats
* `execute` — Execute a document
* `serve` — Serve
* `assistants` — List the available AI assistants
* `repl` — A read-evaluate-print loop for AI assistants
* `test` — 
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

**Usage:** `stencila new [OPTIONS] [TYPE] [PATH]`

###### **Arguments:**

* `<TYPE>` — The type of document to create

  Default value: `article`

  Possible values: `article`

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
* `-t`, `--type <TYPE>` — The type of document to import

  Possible values: `article`

* `-l`, `--losses <LOSSES>` — What to do if there are losses when decoding

  Default value: `warn`
* `--strip-scopes <STRIP_SCOPES>` — Scopes defining which properties of nodes should be stripped

  Possible values:
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

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila log`

Display the history of commits to the document

**Usage:** `stencila log <DOC>`

###### **Arguments:**

* `<DOC>` — The path of the document to display the history for



## `stencila inspect`

Inspect a document as JSON

This command is mostly intended for debugging issues with loading a document from file storage.

**Usage:** `stencila inspect <DOC>`

###### **Arguments:**

* `<DOC>` — The path of the document to inspect



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

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



## `stencila execute`

Execute a document

**Usage:** `stencila execute <INPUT> [OUTPUT]`

###### **Arguments:**

* `<INPUT>` — The path of the file to execute
* `<OUTPUT>` — The path of the file to write the executed document to



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

List the available AI assistants

**Usage:** `stencila assistants`



## `stencila repl`

A read-evaluate-print loop for AI assistants

Mainly intended for prompt engineering during development of Stencila.

**Usage:** `stencila repl [OPTIONS] [TRANSFORM_NODES] [FILTER_NODES] [TAKE_NODES] [ASSERT_NODES]`

###### **Arguments:**

* `<TRANSFORM_NODES>` — The type of node that each decoded node should be transformed to
* `<FILTER_NODES>` — The pattern for the type of node that filtered for after transform in applied
* `<TAKE_NODES>` — The number of nodes to take after filtering
* `<ASSERT_NODES>` — A pattern for the type and number of nodes that should be generated

###### **Options:**

* `-d`, `--document <DOCUMENT>` — The path of the document to use in the context
* `--assistant <ASSISTANT>` — The name of the assistant to use
* `--mirostat <MIROSTAT>` — Enable Mirostat sampling for controlling perplexity
* `--mirostat-eta <MIROSTAT_ETA>` — Influences how quickly the algorithm responds to feedback from the generated text
* `--mirostat-tau <MIROSTAT_TAU>` — Controls the balance between coherence and diversity of the output
* `--num-ctx <NUM_CTX>` — Sets the size of the context window used to generate the next token
* `--num-gqa <NUM_GQA>` — The number of GQA groups in the transformer layer
* `--num-gpu <NUM_GPU>` — The number of layers to send to the GPU(s)
* `--num-thread <NUM_THREAD>` — Sets the number of threads to use during computation
* `--repeat-last-n <REPEAT_LAST_N>` — Sets how far back for the model to look back to prevent repetition
* `--repeat-penalty <REPEAT_PENALTY>` — Sets how strongly to penalize repetitions
* `--temperature <TEMPERATURE>` — The temperature of the model
* `--seed <SEED>` — Sets the random number seed to use for generation
* `--stop <STOP>` — Sets the stop sequences to use
* `--max-tokens <MAX_TOKENS>` — The maximum number of tokens to generate
* `--tfs-z <TFS_Z>` — Tail free sampling is used to reduce the impact of less probable tokens from the output
* `--top-k <TOP_K>` — Reduces the probability of generating nonsense
* `--top-p <TOP_P>` — Works together with top-k
* `--image-quality <IMAGE_QUALITY>` — The quality of the image that will be generated
* `--image-style <IMAGE_STYLE>` — The style of the generated images. Must be one of `vivid` or `natural`



## `stencila test`

**Usage:** `stencila test [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` — The path of test directory or file

###### **Options:**

* `-n`, `--reps <REPS>` — The number of repetitions

  Default value: `1`



## `stencila kernels`

Manage execution kernels

**Usage:** `stencila kernels [COMMAND]`

###### **Subcommands:**

* `list` — List the kernels available
* `info` — Get information about a kernel
* `packages` — List packages available to a kernel



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

