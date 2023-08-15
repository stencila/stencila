# Command-Line Help for `stencila`

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila import`↴](#stencila-import)
* [`stencila export`↴](#stencila-export)
* [`stencila sync`↴](#stencila-sync)
* [`stencila history`↴](#stencila-history)
* [`stencila inspect`↴](#stencila-inspect)
* [`stencila convert`↴](#stencila-convert)
* [`stencila codecs`↴](#stencila-codecs)

## `stencila`

CLI subcommands and global options

**Usage:** `stencila [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `new` — Create a new document
* `import` — Import a file in another format into a new or existing document
* `export` — Export a document to a file in another format
* `sync` — Synchronize a document with one of more other files in other formats
* `history` — Display the history of commits to the document
* `inspect` — Inspect a document as JSON
* `convert` — Convert a document between formats
* `codecs` — Get available format conversion codecs

###### **Options:**

* `--log-level <LOG_LEVEL>` — The minimum log level to output

  Default value: `info`

  Possible values: `trace`, `debug`, `info`, `warn`, `error`

* `--log-filter <LOG_FILTER>` — A filter for log entries

  Default value: ``
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

  Possible values: `debug`, `jats`, `json`, `json5`, `html`, `md`, `ron`, `yaml`

* `--codec <CODEC>` — The codec to use to decode the source
* `-o`, `--overwrite` — Overwrite the document if it already exists



## `stencila import`

Import a file in another format into a new or existing document

**Usage:** `stencila import [OPTIONS] <DOC> <SOURCE>`

###### **Arguments:**

* `<DOC>` — The path of the document to create or import to
* `<SOURCE>` — The source file to import from

###### **Options:**

* `-f`, `--format <FORMAT>` — The format of the source file

  Possible values: `debug`, `jats`, `json`, `json5`, `html`, `md`, `ron`, `yaml`

* `--codec <CODEC>` — The codec to use to decode the source
* `-t`, `--type <TYPE>` — The type of document to import

  Possible values: `article`

* `-l`, `--losses <LOSSES>` — What to do if there are losses when decoding

  Default value: `warn`

  Possible values:
  - `ignore`:
    Ignore the losses; do nothing
  - `trace`:
    Log losses as separate log entries with the `TRACE` severity level
  - `debug`:
    Log losses as separate log entries with the `DEBUG` severity level
  - `info`:
    Log losses as separate log entries with the `INFO` severity level
  - `warn`:
    Log losses as separate log entries with the `WARN` severity level
  - `error`:
    Log losses as separate log entries with the `ERROR` severity level
  - `abort`:
    Abort the current function call by returning a `Err` result with the losses enumerated




## `stencila export`

Export a document to a file in another format

**Usage:** `stencila export [OPTIONS] <DOC> [DEST]`

###### **Arguments:**

* `<DOC>` — The path of the document to export from
* `<DEST>` — The destination file to export to

###### **Options:**

* `-f`, `--format <FORMAT>` — The format of the destination file

  Possible values: `debug`, `jats`, `json`, `json5`, `html`, `md`, `ron`, `yaml`

* `--codec <CODEC>` — The codec to use to encode to the destination
* `-l`, `--losses <LOSSES>` — What to do if there are losses when encoding

  Default value: `warn`

  Possible values:
  - `ignore`:
    Ignore the losses; do nothing
  - `trace`:
    Log losses as separate log entries with the `TRACE` severity level
  - `debug`:
    Log losses as separate log entries with the `DEBUG` severity level
  - `info`:
    Log losses as separate log entries with the `INFO` severity level
  - `warn`:
    Log losses as separate log entries with the `WARN` severity level
  - `error`:
    Log losses as separate log entries with the `ERROR` severity level
  - `abort`:
    Abort the current function call by returning a `Err` result with the losses enumerated

* `-c`, `--compact` — Use compact form of encoding if possible
* `--no-strip-id` — Do not strip the id property of nodes when encoding
* `--strip-code` — Strip the code of executable nodes when encoding
* `--strip-execution` — Strip derived properties of executable nodes when encoding
* `--strip-outputs` — Strip the outputs of executable nodes when encoding



## `stencila sync`

Synchronize a document with one of more other files in other formats

**Usage:** `stencila sync [OPTIONS] <DOC> [FILES]...`

###### **Arguments:**

* `<DOC>` — The path of the document to synchronize
* `<FILES>` — The files to synchronize with

###### **Options:**

* `-f`, `--format <FORMATS>` — The formats of the files (or the name of codecs to use)
* `-d`, `--dir <DIRECTIONS>` — The synchronization directions to use for each file

  Possible values: `in`, `out`, `in-out`

* `-l`, `--losses <LOSSES>` — What to do if there are losses when either encoding or decoding between any of the files

  Default value: `warn`

  Possible values:
  - `ignore`:
    Ignore the losses; do nothing
  - `trace`:
    Log losses as separate log entries with the `TRACE` severity level
  - `debug`:
    Log losses as separate log entries with the `DEBUG` severity level
  - `info`:
    Log losses as separate log entries with the `INFO` severity level
  - `warn`:
    Log losses as separate log entries with the `WARN` severity level
  - `error`:
    Log losses as separate log entries with the `ERROR` severity level
  - `abort`:
    Abort the current function call by returning a `Err` result with the losses enumerated

* `-c`, `--compact` — Use compact form of encoding if possible
* `--no-strip-id` — Do not strip the id property of nodes when encoding
* `--strip-code` — Strip the code of executable nodes when encoding
* `--strip-execution` — Strip derived properties of executable nodes when encoding
* `--strip-outputs` — Strip the outputs of executable nodes when encoding



## `stencila history`

Display the history of commits to the document

**Usage:** `stencila history <DOC>`

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
* `-l`, `--losses <LOSSES>` — What to do if there are losses when either decoding from the input, or encoding to the output

  Default value: `warn`

  Possible values:
  - `ignore`:
    Ignore the losses; do nothing
  - `trace`:
    Log losses as separate log entries with the `TRACE` severity level
  - `debug`:
    Log losses as separate log entries with the `DEBUG` severity level
  - `info`:
    Log losses as separate log entries with the `INFO` severity level
  - `warn`:
    Log losses as separate log entries with the `WARN` severity level
  - `error`:
    Log losses as separate log entries with the `ERROR` severity level
  - `abort`:
    Abort the current function call by returning a `Err` result with the losses enumerated

* `-c`, `--compact` — Use compact form of encoding if possible
* `--no-strip-id` — Do not strip the id property of nodes when encoding
* `--strip-code` — Strip the code of executable nodes when encoding
* `--strip-execution` — Strip derived properties of executable nodes when encoding
* `--strip-outputs` — Strip the outputs of executable nodes when encoding



## `stencila codecs`

Get available format conversion codecs

**Usage:** `stencila codecs [NAME]`

###### **Arguments:**

* `<NAME>` — The name of the codec to show details for



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

