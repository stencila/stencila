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
* [`stencila agents`↴](#stencila-agents)
* [`stencila generate`↴](#stencila-generate)
* [`stencila repl`↴](#stencila-repl)

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
* `agents` — List the available AI agents
* `generate` — Generate text using an AI agents
* `repl` — A read-evaluate-print loop for AI agents

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




## `stencila agents`

List the available AI agents

**Usage:** `stencila agents`



## `stencila generate`

Generate text using an AI agents

Mainly intended for testing. This command runs the same code as when you create an instruction within a document.

**Usage:** `stencila generate [OPTIONS] <INSTRUCTION>`

###### **Arguments:**

* `<INSTRUCTION>` — An instruction of what the agent should generate

###### **Options:**

* `-i`, `--image` — Generate an image rather than text
* `-a`, `--agent <AGENT>` — The name of the agent to use
* `-p`, `--prompt <PROMPT_NAME>` — The name of the prompt to use
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
* `--num-predict <NUM_PREDICT>` — Maximum number of tokens to predict when generating text
* `--top-k <TOP_K>` — Reduces the probability of generating nonsense
* `--top-p <TOP_P>` — Works together with top-k
* `--image-quality <IMAGE_QUALITY>` — The quality of the image that will be generated
* `--image-style <IMAGE_STYLE>` — The style of the generated images. Must be one of `vivid` or `natural`. Vivid causes the model to lean towards generating hyper-real and dramatic images. Natural causes the model to produce more natural, less hyper-real looking images. Supported by `openai/dall-e-3`



## `stencila repl`

A read-evaluate-print loop for AI agents

**Usage:** `stencila repl [OPTIONS]`

###### **Options:**

* `-a`, `--agent <AGENT>` — The name of the agent to interact with
* `-p`, `--prompt <PROMPT_NAME>` — The name of the prompt to use
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
* `--num-predict <NUM_PREDICT>` — Maximum number of tokens to predict when generating text
* `--top-k <TOP_K>` — Reduces the probability of generating nonsense
* `--top-p <TOP_P>` — Works together with top-k
* `--image-quality <IMAGE_QUALITY>` — The quality of the image that will be generated
* `--image-style <IMAGE_STYLE>` — The style of the generated images. Must be one of `vivid` or `natural`. Vivid causes the model to lean towards generating hyper-real and dramatic images. Natural causes the model to produce more natural, less hyper-real looking images. Supported by `openai/dall-e-3`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

