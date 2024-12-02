# Command-Line Help for `stencila`

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila convert`↴](#stencila-convert)
* [`stencila sync`↴](#stencila-sync)
* [`stencila compile`↴](#stencila-compile)
* [`stencila execute`↴](#stencila-execute)
* [`stencila render`↴](#stencila-render)
* [`stencila preview`↴](#stencila-preview)
* [`stencila publish`↴](#stencila-publish)
* [`stencila serve`↴](#stencila-serve)
* [`stencila lsp`↴](#stencila-lsp)
* [`stencila prompts`↴](#stencila-prompts)
* [`stencila prompts list`↴](#stencila-prompts-list)
* [`stencila prompts show`↴](#stencila-prompts-show)
* [`stencila prompts select`↴](#stencila-prompts-select)
* [`stencila prompts update`↴](#stencila-prompts-update)
* [`stencila prompts reset`↴](#stencila-prompts-reset)
* [`stencila models`↴](#stencila-models)
* [`stencila models list`↴](#stencila-models-list)
* [`stencila models execute`↴](#stencila-models-execute)
* [`stencila kernels`↴](#stencila-kernels)
* [`stencila kernels list`↴](#stencila-kernels-list)
* [`stencila kernels info`↴](#stencila-kernels-info)
* [`stencila kernels packages`↴](#stencila-kernels-packages)
* [`stencila kernels execute`↴](#stencila-kernels-execute)
* [`stencila kernels evaluate`↴](#stencila-kernels-evaluate)
* [`stencila codecs`↴](#stencila-codecs)
* [`stencila codecs list`↴](#stencila-codecs-list)
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
* [`stencila upgrade`↴](#stencila-upgrade)
* [`stencila uninstall`↴](#stencila-uninstall)
* [`stencila config`↴](#stencila-config)

## `stencila`

CLI subcommands and global options

**Usage:** `stencila [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `new` — Create a new document with sidecar file
* `convert` — Convert a document to another format
* `sync` — Synchronize a document between formats
* `compile` — Compile a document
* `execute` — Execute a document
* `render` — Render a document
* `preview` — Preview a document or site
* `publish` — Publish a document or site
* `serve` — Run the HTTP/Websocket server
* `lsp` — Run the Language Server Protocol server
* `prompts` — Manage prompts
* `models` — Manage generative models
* `kernels` — Manage execution kernels
* `codecs` — Manage format conversion codecs
* `plugins` — Manage plugins
* `secrets` — Manage secrets
* `upgrade` — Upgrade to the latest version
* `uninstall` — Uninstall this command line tool
* `config` — 

###### **Options:**

* `--debug` — Display debug level logging and detailed error reports

   Equivalent to using `--log-level=debug`, `--log-format=pretty`, and `--error-details=all`
* `--trace` — Display trace level logging and detailed error reports

   Equivalent to using `--log-level=trace`, `--log-format=pretty`, and `--error-details=all`
* `--log-level <LOG_LEVEL>` — The minimum log level to output

  Default value: `info`

  Possible values: `trace`, `debug`, `info`, `warn`, `error`

* `--log-filter <LOG_FILTER>` — A filter for log entries

   Allows more fine-grained control over which log entries are shown. To additionally see lower level entries for a specific crates use syntax such as `tokio=debug`.

  Default value: `globset=warn,hyper=info,hyper_util=info,ignore=warn,mio=info,notify=warn,ort=error,reqwest=info,sled=info,tokio=info,tungstenite=info`
* `--log-format <LOG_FORMAT>` — The log format to use

   When `auto`, uses `simple` for terminals and `json` for non-TTY devices.

  Default value: `auto`

  Possible values: `auto`, `simple`, `compact`, `pretty`, `full`, `json`

* `--error-details <ERROR_DETAILS>` — The details to include in error reports

   `auto`, `all`, or a comma separated list including `location`, `span`, or `env`.

  Default value: `auto`
* `--error-link` — Output a link to more easily report an issue



## `stencila new`

Create a new document with sidecar file

**Usage:** `stencila new [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` — The path of the document to create

###### **Options:**

* `-f`, `--force` — Overwrite the document, and any sidecar file, if they already exist
* `-t`, `--type <TYPE>` — The type of document to create

  Default value: `article`

  Possible values: `article`, `prompt`

* `-s`, `--sidecar <SIDECAR>` — The format of the sidecar file

  Possible values: `json.zip`, `json`




## `stencila convert`

Convert a document to another format

**Usage:** `stencila convert [OPTIONS] [INPUT] [OUTPUT] [PASSTHROUGH_ARGS]...`

###### **Arguments:**

* `<INPUT>` — The path of the input file

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the output file

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for conversion (e.g. Pandoc)

###### **Options:**

* `-f`, `--from <FROM>` — The format to encode from (or codec to use)

   Defaults to inferring the format from the file name extension of the `input`. See `stencila codecs list` for available formats.
* `-t`, `--to <TO>` — The format to encode to (or codec to use)

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to JSON. See `stencila codecs list` for available formats.
* `-i`, `--input-losses <INPUT_LOSSES>` — What to do if there are losses when decoding from the input

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `warn`
* `-o`, `--output-losses <OUTPUT_LOSSES>` — What to do if there are losses when encoding to the output

   See help for `--input-losses` for details.

  Default value: `warn`
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-r`, `--render` — For executable nodes, only encode outputs, not source properties
* `-c`, `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
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
  - `archive`:
    Strip archive properties of node
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

Synchronize a document between formats

The direction of synchronization can be specified by appending the to the file path:

- `:in` only sync incoming changes from the file - `:out` only sync outgoing changes to the file - `:io` sync incoming and outgoing changes (default)

**Usage:** `stencila sync [OPTIONS] <DOC> [FILES]...`

###### **Arguments:**

* `<DOC>` — The path of the document to synchronize
* `<FILES>` — The files to synchronize with

###### **Options:**

* `-f`, `--format <FORMATS>` — The formats of the files (or the name of codecs to use)

   This option can be provided separately for each file.
* `-l`, `--losses <LOSSES>` — What to do if there are losses when either encoding or decoding between any of the files

  Default value: `warn`
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-r`, `--render` — For executable nodes, only encode outputs, not source properties
* `-c`, `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
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
  - `archive`:
    Strip archive properties of node
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

**Usage:** `stencila compile [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

###### **Arguments:**

* `<INPUT>` — The path of the file to execute

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the executed document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

###### **Options:**

* `-t`, `--to <TO>` — The format to encode to (or codec to use)

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to JSON. See `stencila codecs list` for available formats.
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-r`, `--render` — For executable nodes, only encode outputs, not source properties
* `-c`, `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
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
  - `archive`:
    Strip archive properties of node
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
* `--no-save` — Do not save the document after compiling it



## `stencila execute`

Execute a document

**Usage:** `stencila execute [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

###### **Arguments:**

* `<INPUT>` — The path of the file to execute

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the executed document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

###### **Options:**

* `-t`, `--to <TO>` — The format to encode to (or codec to use)

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to JSON. See `stencila codecs list` for available formats.
* `--force-all` — Re-execute all node types regardless of current state
* `--skip-code` — Skip executing code

   By default, code-based nodes in the document (e.g. `CodeChunk`, `CodeExpression`, `ForBlock`) nodes will be executed if they are stale. Use this flag to skip executing all code-based nodes.
* `--skip-instructions` — Skip executing instructions

   By default, instructions with no suggestions, or with suggestions that have been rejected will be executed. Use this flag to skip executing all instructions.
* `--retain-suggestions` — Retain existing suggestions for instructions

   By default, when you execute an instruction, the existing suggestions for the instruction are deleted. Use this flag to retain existing suggestions, for example, so that you can use a previous one if a revision is worse.
* `--force-unreviewed` — Re-execute instructions with suggestions that have not yet been reviewed

   By default, an instruction that has a suggestion that has not yet be reviewed (i.e. has a suggestion status that is empty) will not be re-executed. Use this flag to force these instructions to be re-executed.
* `--force-accepted` — Re-execute instructions with suggestions that have been accepted.

   By default, an instruction that has a suggestion that has been accepted, will not be re-executed. Use this flag to force these instructions to be re-executed.
* `--skip-rejected` — Skip re-executing instructions with suggestions that have been rejected

   By default, instructions that have a suggestion that has been rejected, will be re-executed. Use this flag to skip re-execution of these instructions.
* `--dry-run` — Prepare, but do not actually perform, execution tasks

   Currently only supported by instructions where it is useful for debugging the rendering of prompts without making a potentially slow generative model API request.
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-r`, `--render` — For executable nodes, only encode outputs, not source properties
* `-c`, `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
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
  - `archive`:
    Strip archive properties of node
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
* `--no-save` — Do not save the document after compiling it



## `stencila render`

Render a document

Equivalent to the `execute` command with the `--render` flag.

**Usage:** `stencila render [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

###### **Arguments:**

* `<INPUT>` — The path of the file to render

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the rendered document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

###### **Options:**

* `-t`, `--to <TO>` — The format to encode to (or codec to use)

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to Markdown. See `stencila codecs list` for available formats.
* `--force-all` — Re-execute all node types regardless of current state
* `--skip-code` — Skip executing code

   By default, code-based nodes in the document (e.g. `CodeChunk`, `CodeExpression`, `ForBlock`) nodes will be executed if they are stale. Use this flag to skip executing all code-based nodes.
* `--skip-instructions` — Skip executing instructions

   By default, instructions with no suggestions, or with suggestions that have been rejected will be executed. Use this flag to skip executing all instructions.
* `--retain-suggestions` — Retain existing suggestions for instructions

   By default, when you execute an instruction, the existing suggestions for the instruction are deleted. Use this flag to retain existing suggestions, for example, so that you can use a previous one if a revision is worse.
* `--force-unreviewed` — Re-execute instructions with suggestions that have not yet been reviewed

   By default, an instruction that has a suggestion that has not yet be reviewed (i.e. has a suggestion status that is empty) will not be re-executed. Use this flag to force these instructions to be re-executed.
* `--force-accepted` — Re-execute instructions with suggestions that have been accepted.

   By default, an instruction that has a suggestion that has been accepted, will not be re-executed. Use this flag to force these instructions to be re-executed.
* `--skip-rejected` — Skip re-executing instructions with suggestions that have been rejected

   By default, instructions that have a suggestion that has been rejected, will be re-executed. Use this flag to skip re-execution of these instructions.
* `--dry-run` — Prepare, but do not actually perform, execution tasks

   Currently only supported by instructions where it is useful for debugging the rendering of prompts without making a potentially slow generative model API request.
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-r`, `--render` — For executable nodes, only encode outputs, not source properties
* `-c`, `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
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
  - `archive`:
    Strip archive properties of node
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
* `--no-save` — Do not save the document after compiling it



## `stencila preview`

Preview a document or site

Opens a preview of a document or site in the browser. When `--sync=in` (the default) the preview will update when the document is saved to disk.

**Usage:** `stencila preview [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — The path to the document file or site directory to preview

   Defaults to the current directory.

  Default value: `.`

###### **Options:**

* `--sync <SYNC>` — Which direction(s) to sync the document

  Default value: `in`

  Possible values: `in`, `out`, `in-out`




## `stencila publish`

Publish a document or site

Currently only supports publishing a single document to the web via Stencila Cloud.

In the future, it is likely that other publication platforms will be supported.

**Usage:** `stencila publish [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — The path to the document file or site directory to publish

   Defaults to the current directory.

  Default value: `.`

###### **Options:**

* `-k`, `--key <KEY>` — Key or identifier required by the platform being published to
* `--dry-run` — Perform a dry run
* `--no-html` — Do not publish a HTML file
* `--no-jsonld` — Do not publish a JSON-LD file
* `--no-llmd` — Do not publish a LLM-Markdown file
* `--no-bots` — Disallow all bots
* `--no-ai-bots` — Disallow AI bots



## `stencila serve`

Run the HTTP/Websocket server

**Usage:** `stencila serve [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory to serve

   Defaults to the current working directory

  Default value: `.`

###### **Options:**

* `-a`, `--address <ADDRESS>` — The address to serve on

   Defaults to `127.0.0.1` (localhost), use `0.0.0.0` to listen on all addresses.

  Default value: `127.0.0.1`
* `-p`, `--port <PORT>` — The port to serve on

   Defaults to port 9000.

  Default value: `9000`
* `--no-auth` — Do not authenticate or authorize requests

   By default, requests to all routes (except `~static/*`) require an access token.
* `--raw` — Should files be served raw?

   When `true` and a request is made to a path that exists within `dir`, the file will be served with a `Content-Type` header corresponding to the file's extension.
* `--source` — Should `SourceMap` headers be sent?

   When `true`, then the `SourceMap` header will be set with the URL of the document that was rendered as HTML. Usually only useful if `raw` is also `true`.
* `--sync <SYNC>` — Whether and in which direction(s) to sync served documents

  Possible values: `in`, `out`, `in-out`




## `stencila lsp`

Run the Language Server Protocol server

**Usage:** `stencila lsp`



## `stencila prompts`

Manage prompts

**Usage:** `stencila prompts [COMMAND]`

###### **Subcommands:**

* `list` — List the prompts available
* `show` — Show a prompt
* `select` — Select a prompt
* `update` — Update builtin prompts
* `reset` — Reset builtin prompts



## `stencila prompts list`

List the prompts available

**Usage:** `stencila prompts list`



## `stencila prompts show`

Show a prompt

**Usage:** `stencila prompts show [OPTIONS] <ID>`

###### **Arguments:**

* `<ID>` — The id of the prompt to show

###### **Options:**

* `-t`, `--to <TO>` — The format to show the prompt in

  Default value: `yaml`



## `stencila prompts select`

Select a prompt

Useful for checking which prompt will be matched to a given instruction

**Usage:** `stencila prompts select <TYPE> <MESSAGE>`

###### **Arguments:**

* `<TYPE>` — The type of instruction
* `<MESSAGE>` — The instruction message



## `stencila prompts update`

Update builtin prompts

**Usage:** `stencila prompts update`



## `stencila prompts reset`

Reset builtin prompts

Re-initializes the builtin prompts directory to those prompts embedded in this version of Stencila

**Usage:** `stencila prompts reset`



## `stencila models`

Manage generative models

**Usage:** `stencila models [COMMAND]`

###### **Subcommands:**

* `list` — List the models available
* `execute` — Execute a model task



## `stencila models list`

List the models available

**Usage:** `stencila models list`



## `stencila models execute`

Execute a model task

Mainly intended for testing of model selection and routing.

**Usage:** `stencila models execute [OPTIONS] <PROMPT>`

###### **Arguments:**

* `<PROMPT>`

###### **Options:**

* `-m`, `--model <MODEL>` — The id pattern to specify the model to use
* `--dry-run` — Perform a dry run



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

**Usage:** `stencila kernels list [OPTIONS]`

###### **Options:**

* `-t`, `--type <TYPE>` — Only show languages of a particular type

  Possible values: `programming`, `templating`, `diagrams`, `math`, `styling`




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

   Only packages whose name contains this string will be included (case insensitive)



## `stencila kernels execute`

Execute code in a kernel

Creates a temporary kernel instance, executes one or more lines of code, and returns any outputs and execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels execute <NAME> <CODE>`

###### **Arguments:**

* `<NAME>` — The name of the kernel to execute code in
* `<CODE>` — The code to execute

   Escaped newline characters (i.e. "\n") in the code will be transformed into new lines before passing to the kernel.



## `stencila kernels evaluate`

Evaluate a code expression in a kernel

Creates a temporary kernel instance, evaluates the expression in it, and returns the output and any execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels evaluate <NAME> <CODE>`

###### **Arguments:**

* `<NAME>` — The name of the kernel to evaluate code in
* `<CODE>` — The code expression to evaluate



## `stencila codecs`

Manage format conversion codecs

**Usage:** `stencila codecs [COMMAND]`

###### **Subcommands:**

* `list` — List the codecs available



## `stencila codecs list`

List the codecs available

**Usage:** `stencila codecs list`



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
* `--installed` — Only list installed plugins
* `--installable` — Only list installable plugins
* `-o`, `--outdated` — Only list installed but outdated plugins
* `-e`, `--enabled` — Only list installed and enabled plugins



## `stencila plugins install`

Install a plugin

**Usage:** `stencila plugins install <NAME>`

###### **Arguments:**

* `<NAME>` — The name or URL of the plugin to install

   If a URL is supplied it should be a URL to the manifest TOML file of the plugin. e.g. https://example.org/plugin/stencila-plugin.toml



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

**Usage:** `stencila plugins check [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the plugin to install

###### **Options:**

* `--skip-codecs` — Skip checking codecs
* `--skip-kernels` — Skip checking kernels
* `--skip-models` — Skip checking models



## `stencila secrets`

Manage secrets

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

You will be prompted for the secret. Alternatively, you can echo the password into this command i.e. `echo <TOKEN> | stencila secrets set STENCILA_API_TOKEN`

**Usage:** `stencila secrets set <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila secrets delete`

Delete a secret previously set using Stencila

**Usage:** `stencila secrets delete <NAME>`

###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila upgrade`

Upgrade to the latest version

**Usage:** `stencila upgrade [OPTIONS]`

###### **Options:**

* `-f`, `--force` — Perform upgrade even if the current version is the latest
* `-c`, `--check` — Check for an available upgrade but do not install it



## `stencila uninstall`

Uninstall this command line tool

**Usage:** `stencila uninstall`



## `stencila config`

**Usage:** `stencila config [OPTIONS]`

###### **Options:**

* `--dir <DIR>`

  Default value: `config`

  Possible values: `config`, `cache`, `prompts`, `plugins`, `kernels`

* `--ensure`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

