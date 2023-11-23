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
* [`stencila serve`↴](#stencila-serve)

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
* `serve` — Serve

###### **Options:**

* `--log-level <LOG_LEVEL>` — The minimum log level to output

  Default value: `info`

  Possible values: `trace`, `debug`, `info`, `warn`, `error`

* `--log-filter <LOG_FILTER>` — A filter for log entries

  Default value: `hyper=info,mio=info,tokio=info,tungstenite=info`
* `--log-format <LOG_FORMAT>` — The log format to use

  Default value: `auto`

  Possible values: `auto`, `simple`, `compact`, `pretty`, `full`, `json`

* `--error-details <ERROR_DETAILS>` — The details to include in error reports

  Default value: `auto`
* `--error-link` — Output a link to more easily report an issue



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

  Possible values:
  - `article`
  - `html`
  - `jats`
  - `markdown`
  - `text`
  - `json`
  - `json5`
  - `json-ld`
  - `cbor`
  - `cbor-zst`
  - `yaml`
  - `gif`:
    Image formats
  - `jpeg`
  - `png`
  - `svg`
  - `web-p`
  - `aac`:
    Audio formats
  - `flac`
  - `mp3`
  - `ogg`
  - `wav`
  - `avi`:
    Video formats
  - `mkv`
  - `mp4`
  - `ogv`
  - `web-m`
  - `debug`
  - `unknown`

* `--codec <CODEC>` — The codec to use to decode the source
* `-o`, `--overwrite` — Overwrite the document if it already exists



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

  Default value: `id`



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
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-c`, `--compact` — Use compact form of encoding if possible
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible
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

  Default value: `id`



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
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-c`, `--compact` — Use compact form of encoding if possible
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible
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

  Default value: `id`



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
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `-c`, `--compact` — Use compact form of encoding if possible
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible
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

  Default value: `id`



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
* `--source` — Should `SourceMap` headers be sent?
* `--sync <SYNC>` — Whether and in which direction(s) to sync served documents

  Possible values: `in`, `out`, `in-out`




<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

