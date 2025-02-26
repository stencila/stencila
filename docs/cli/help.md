---
title: CLI Help
description: Help for the `stencila` CLI 
config:
  publish:
    ghost:
      slug: cli-help
      type: post
      state: publish
      tags:
        - '#doc'
        - CLI
---

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila init`↴](#stencila-init)
* [`stencila config`↴](#stencila-config)
* [`stencila status`↴](#stencila-status)
* [`stencila track`↴](#stencila-track)
* [`stencila untrack`↴](#stencila-untrack)
* [`stencila move`↴](#stencila-move)
* [`stencila remove`↴](#stencila-remove)
* [`stencila convert`↴](#stencila-convert)
* [`stencila sync`↴](#stencila-sync)
* [`stencila compile`↴](#stencila-compile)
* [`stencila lint`↴](#stencila-lint)
* [`stencila execute`↴](#stencila-execute)
* [`stencila render`↴](#stencila-render)
* [`stencila preview`↴](#stencila-preview)
* [`stencila publish`↴](#stencila-publish)
* [`stencila publish zenodo`↴](#stencila-publish-zenodo)
* [`stencila publish ghost`↴](#stencila-publish-ghost)
* [`stencila publish stencila`↴](#stencila-publish-stencila)
* [`stencila serve`↴](#stencila-serve)
* [`stencila lsp`↴](#stencila-lsp)
* [`stencila prompts`↴](#stencila-prompts)
* [`stencila prompts list`↴](#stencila-prompts-list)
* [`stencila prompts show`↴](#stencila-prompts-show)
* [`stencila prompts infer`↴](#stencila-prompts-infer)
* [`stencila prompts update`↴](#stencila-prompts-update)
* [`stencila prompts reset`↴](#stencila-prompts-reset)
* [`stencila models`↴](#stencila-models)
* [`stencila models list`↴](#stencila-models-list)
* [`stencila models run`↴](#stencila-models-run)
* [`stencila kernels`↴](#stencila-kernels)
* [`stencila kernels list`↴](#stencila-kernels-list)
* [`stencila kernels info`↴](#stencila-kernels-info)
* [`stencila kernels packages`↴](#stencila-kernels-packages)
* [`stencila kernels execute`↴](#stencila-kernels-execute)
* [`stencila kernels evaluate`↴](#stencila-kernels-evaluate)
* [`stencila kernels lint`↴](#stencila-kernels-lint)
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

# `stencila`

CLI subcommands and global options

**Usage:** `stencila [OPTIONS] <COMMAND>`

**Subcommands:**

* `new` — Create a new, tracked, document
* `init` — Initialize document config and tracking
* `config` — Display the configuration for a document
* `status` — Get the tracking status of documents
* `track` — Start tracking a document
* `untrack` — Stop tracking a document
* `move` — Move a tracked document
* `remove` — Remove a tracked document
* `convert` — Convert a document to another format
* `sync` — Synchronize a document between formats
* `compile` — Compile a document
* `lint` — Lint one or more documents
* `execute` — Execute a document
* `render` — Render a document
* `preview` — Preview a document or site
* `publish` — Publish one or more documents
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

**Options:**

* `--debug` — Display debug level logging and detailed error reports



# `stencila new`

Create a new, tracked, document

**Usage:** `stencila new [OPTIONS] <PATH>`

**Arguments:**

* `<PATH>` — The path of the document to create

**Options:**

* `-f`, `--force` — Overwrite the document, if it already exists
* `-t`, `--type <TYPE>` — The type of document to create

  Default value: `article`

  Possible values: `article`, `chat`, `prompt`




# `stencila init`

Initialize document config and tracking

**Usage:** `stencila init [OPTIONS] [DIR]`

**Arguments:**

* `<DIR>` — The directory to start document tracking in

   Defaults to the current directory.

  Default value: `.`

**Options:**

* `--gitignore` — Create a `.gitignore` file



# `stencila config`

Display the configuration for a document

**Usage:** `stencila config <FILE>`

**Arguments:**

* `<FILE>` — The path to the document to resolve



# `stencila status`

Get the tracking status of documents

**Usage:** `stencila status [OPTIONS] [FILES]...`

**Arguments:**

* `<FILES>` — The paths of the files to get status for

**Options:**

* `-a`, `--as <AS>` — Output the status as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila track`

Start tracking a document

**Usage:** `stencila track <FILE> [URL]`

**Arguments:**

* `<FILE>` — The path to the local file to track
* `<URL>` — The URL of the remote to track



# `stencila untrack`

Stop tracking a document

**Usage:** `stencila untrack <FILE> [URL]`

**Arguments:**

* `<FILE>` — The path of the file to stop tracking
* `<URL>` — The URL of the remote to stop tracking



# `stencila move`

Move a tracked document

Moves the document file to the new path (if it still exists at the old path) and updates any tracking information.

**Usage:** `stencila move [OPTIONS] <FROM> <TO>`

**Arguments:**

* `<FROM>` — The old path of the file
* `<TO>` — The new path of the file

**Options:**

* `-f`, `--force` — Overwrite the destination path if it already exists



# `stencila remove`

Remove a tracked document

Deletes the document file (if it still exists) and removes any tracking data from the `.stencila` directory.

**Usage:** `stencila remove [OPTIONS] <FILE>`

**Arguments:**

* `<FILE>` — The path of the file to remove

**Options:**

* `-f`, `--force` — Do not ask for confirmation of removal



# `stencila convert`

Convert a document to another format

**Usage:** `stencila convert [OPTIONS] [INPUT] [OUTPUT] [PASSTHROUGH_ARGS]...`

**Arguments:**

* `<INPUT>` — The path of the input file

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the output file

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for conversion (e.g. Pandoc)

**Options:**

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
  - `temporary`:
    Strip temporary properties of a node
  - `code`:
    Strip code properties of executable nodes
  - `compilation`:
    Strip compilation related properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



# `stencila sync`

Synchronize a document between formats

The direction of synchronization can be specified by appending the to the file path:

- `:in` only sync incoming changes from the file - `:out` only sync outgoing changes to the file - `:io` sync incoming and outgoing changes (default)

**Usage:** `stencila sync [OPTIONS] <DOC> [FILES]...`

**Arguments:**

* `<DOC>` — The path of the document to synchronize
* `<FILES>` — The files to synchronize with

**Options:**

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
  - `temporary`:
    Strip temporary properties of a node
  - `code`:
    Strip code properties of executable nodes
  - `compilation`:
    Strip compilation related properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip



# `stencila compile`

Compile a document

**Usage:** `stencila compile [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

**Arguments:**

* `<INPUT>` — The path of the file to execute

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the executed document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

**Options:**

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
  - `temporary`:
    Strip temporary properties of a node
  - `code`:
    Strip code properties of executable nodes
  - `compilation`:
    Strip compilation related properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip
* `--no-save` — Do not save the document after compiling it



# `stencila lint`

Lint one or more documents

**Usage:** `stencila lint [OPTIONS] [FILES]...`

**Arguments:**

* `<FILES>` — The files to lint

**Options:**

* `--format` — Format the file if necessary
* `--fix` — Fix any linting issues
* `-a`, `--as <AS>` — Output any linting diagnostics as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila execute`

Execute a document

**Usage:** `stencila execute [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

**Arguments:**

* `<INPUT>` — The path of the file to execute

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the executed document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

**Options:**

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
  - `temporary`:
    Strip temporary properties of a node
  - `code`:
    Strip code properties of executable nodes
  - `compilation`:
    Strip compilation related properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip
* `--no-save` — Do not save the document after compiling it



# `stencila render`

Render a document

Equivalent to the `execute` command with the `--render` flag.

**Usage:** `stencila render [OPTIONS] <INPUT> [OUTPUT] [PASSTHROUGH_ARGS]...`

**Arguments:**

* `<INPUT>` — The path of the file to render

   If not supplied the input content is read from `stdin`.
* `<OUTPUT>` — The path of the file to write the rendered document to

   If not supplied the output content is written to `stdout`.
* `<PASSTHROUGH_ARGS>` — Arguments to pass through to any CLI tool delegated to for encoding to the output format (e.g. Pandoc)

**Options:**

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
  - `temporary`:
    Strip temporary properties of a node
  - `code`:
    Strip code properties of executable nodes
  - `compilation`:
    Strip compilation related properties of executable nodes
  - `execution`:
    Strip execution related properties of executable nodes
  - `output`:
    Strip output properties of executable nodes
  - `timestamps`:
    Strip timestamp properties

* `--strip-types <STRIP_TYPES>` — A list of node types to strip
* `--strip-props <STRIP_PROPS>` — A list of node properties to strip
* `--no-save` — Do not save the document after compiling it



# `stencila preview`

Preview a document or site

Opens a preview of a document or site in the browser. When `--sync=in` (the default) the preview will update when the document is saved to disk.

**Usage:** `stencila preview [OPTIONS] [PATH]`

**Arguments:**

* `<PATH>` — The path to the document file or site directory to preview

   Defaults to the current directory.

  Default value: `.`

**Options:**

* `--sync <SYNC>` — Which direction(s) to sync the document

  Default value: `in`

  Possible values: `in`, `out`, `in-out`




# `stencila publish`

Publish one or more documents

**Usage:** `stencila publish <COMMAND>`

**Subcommands:**

* `zenodo` — Publish to Zenodo
* `ghost` — Publish to Ghost
* `stencila` — Publish to Stencila Cloud



# `stencila publish zenodo`

Publish to Zenodo

**Usage:** `stencila publish zenodo [OPTIONS] [PATH]`
**Arguments:**

* `<PATH>` — Path to location of what to publish

  Default value: `.`

**Options:**

* `--token <TOKEN>` — Zenodo authentication token

   To create one, log into Zenodo, visit your account's page, then click "Applications", followed by "+ New Token" within the "Personal access tokens" section. Give the token a name and enable the "deposit:actions" the scope.

   Enable the "deposit:write" scope to enable the `--force` flag.
* `--sandbox` — Publish to the Zenodo Sandbox for testing

   The Zenodo Sandbox is available at https://sandbox.zenodo.org. It requires its own access key that is independent from the Zenodo production server.

   [default]

  Default value: `true`
* `--zenodo <ZENODO>` — Specify Zenodo instance, defaults to the public-facing production server

   Use this option to publish to a custom Zenodo instance. Provide just the domain name or IP address with an optional port, e.g. `zenodo.example.org` or `zenodo.example.org:8000`.

  Default value: `zenodo.org`
* `--lesson` — Upload document as a "lesson"
* `--reserve-doi` — Reserve a DOI for the deposition (overrides DOI in Article metadata, if any)
* `--doi <DOI>` — Supply an existing DOI

   Use this field to provide a DOI that has already been issued for the material you are depositing.
* `--publication-date <YYYY-MM-DD>` — Publication date

   Provide the date formatted as YYYY-MM-DD, e.g. 2012-03-10.

   When omitted, Zenodo will use today's date.
* `--title <TITLE>` — Title to use for the deposit

   Required when the information is not available within the document.
* `--description <DESCRIPTION>` — Description to use within the deposition

   Required when the information is not available within the document. HTML is allowed.
* `--license <LICENSE>` — License Identifier (examples: cc-by, cc0)
* `--closed` — Closed Access

   Public access of the deposition is not allowed.

   Shorthand for `--access-right=closed`.
* `--restricted` — Set `--access-right` to restricted
* `--embargoed <YYYY-MM-DD>` — Provide a date when the embargo ends
* `--access-conditions <ACCESS_CONDITIONS>` — Conditions to fulfill to access deposition

   Describe the conditions of access to the deposition for be accessed when --access-right=restricted. HTML is allowed.
* `--access-right <ACCESS_RIGHT>` — Access right

  Default value: `open`

  Possible values:
  - `open`:
    Open Access. Sets the default license to CC-BY, e.g. --license='cc-by'.
  - `embargoed`:
    Embargoed Access. Requires --access_conditions, --license, and --embargoed=<DATE>.
  - `restricted`:
    Restricted Access. Requires --access_conditions.
  - `closed`:
    Closed Access.

* `--keywords <KEYWORDS>` — Comma-delimited list of keywords

   To add multiple keywords, separate them with commas: --keywords=testing,software

   To include spaces in keywords, surround the list with quotes[*]: --keywords='testing,software,software testing'

   [*] The exact syntax will depend on your shell language.
* `--method <METHOD>` — Methodology

   Free-form description of the methodology used in this research. HTML is allowed.
* `--notes <NOTES>` — Additional Notes

   Any additional notes that to do not fit within the description. HTML is allowed.
* `--version <VERSION>` — Version of document

   NOTE: this is a free text field and all inputs are be accepted. However, the suggested format is a semantically versioned tag (see more details on semantic versioning at semver.org).
* `--publication <PUBLICATION_TYPE>` — Upload document as a "publication"

   Provide one of the publication types from Zenodo's controlled vocabulary.

  Default value: `preprint`

  Possible values: `annotation-collection`, `book`, `section`, `conference-paper`, `data-management-plan`, `article`, `patent`, `preprint`, `deliverable`, `milestone`, `proposal`, `report`, `software-documentation`, `taxonomic-treatment`, `technical-note`, `thesis`, `working-paper`, `other`

* `--force` — Publish the deposition immediately

   Requires that access token provided by the `--token` option has the "deposit:write" scope.

   WARNING: This is permanent. It will be impossible to review the deposition or make changes to it before it is publicly viewable. Publication cannot be revoked.
* `--dry-run` — Dry run mode - no actual upload



# `stencila publish ghost`

Publish to Ghost

**Usage:** `stencila publish ghost [OPTIONS] <PATHS>...`

**Arguments:**

* `<PATHS>` — Paths to the files to publish

**Options:**

* `--ghost <GHOST>` — The Ghost domain

   This is the domain name of your Ghost instance, with an optional port.

   Not required when pushing or pulling an existing post or page from Ghost but if supplied only document `identifiers` with this host will be used.
* `--key <KEY>` — The Ghost Admin API key

   To create one, create a new Custom Integration under the Integrations screen in Ghost Admin. Use the Admin API Key, rather than the Content API Key.

   You can also set the key as a secret so that it does not need to be entered here each time: `stencila secrets set GHOST_ADMIN_API_KEY`.
* `--page` — Create a page

   Does not apply when pushing to, or pulling from, and existing Ghost resource.

  Default value: `false`
* `--post` — Create a post

   Does not apply when pushing to, or pulling from, and existing Ghost resource.
* `--push` — Create or update Ghost post or page from a file

  Default value: `true`
* `--pull` — Update file from an existing Ghost post or page
* `--id <ID>` — Ghost id of the page or post
* `--title <TITLE>` — Title for page or post
* `--draft` — Mark page or post as draft

  Default value: `false`
* `--publish` — Publish page or post
* `--schedule <SCHEDULE>` — Schedule page or post
* `--slug <SLUG>` — Set slug(URL slug the page or post will be available at)
* `--tag <TAGS>` — Tags for page or post
* `--excerpt <EXCERPT>` — Excerpt for page or post

   Defaults to the article description
* `--featured` — Feature post or page
* `--inject-code-header <INJECT_CODE_HEADER>` — Inject HTML header
* `--inject-code-footer <INJECT_CODE_FOOTER>` — Inject HTML footer
* `--dry-run` — Dry run test

   When set, Stencila will perform the document conversion but skip the publication to Ghost.

  Default value: `false`



# `stencila publish stencila`

Publish to Stencila Cloud

**Usage:** `stencila publish stencila [OPTIONS] [PATH]`

**Arguments:**

* `<PATH>` — Path to the file or directory to publish

   Defaults to the current directory.

  Default value: `.`

**Options:**

* `-k`, `--key <KEY>` — The key for the site
* `--dry-run` — Perform a dry run only
* `--no-html` — Do not publish a HTML file
* `--no-jsonld` — Do not publish a JSON-LD file
* `--no-llmd` — Do not publish a LLM-Markdown file
* `--no-bots` — Disallow all bots
* `--no-ai-bots` — Disallow AI bots



# `stencila serve`

Run the HTTP/Websocket server

**Usage:** `stencila serve [OPTIONS] [DIR]`

**Arguments:**

* `<DIR>` — The directory to serve

   Defaults to the current working directory

  Default value: `.`

**Options:**

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




# `stencila lsp`

Run the Language Server Protocol server

**Usage:** `stencila lsp`



# `stencila prompts`

Manage prompts

**Usage:** `stencila prompts [COMMAND]`

**Subcommands:**

* `list` — List the prompts available
* `show` — Show a prompt
* `infer` — Infer a prompt from a query
* `update` — Update builtin prompts
* `reset` — Reset builtin prompts



# `stencila prompts list`

List the prompts available

**Usage:** `stencila prompts list [OPTIONS]`

**Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila prompts show`

Show a prompt

**Usage:** `stencila prompts show [OPTIONS] <NAME>`

**Arguments:**

* `<NAME>` — The name of the prompt to show

**Options:**

* `-t`, `--to <TO>` — The format to show the prompt in

  Default value: `md`



# `stencila prompts infer`

Infer a prompt from a query

Useful for checking which prompt will be matched to a given instruction type, node types, and/or query

**Usage:** `stencila prompts infer [OPTIONS] [QUERY]`

**Arguments:**

* `<QUERY>` — The query

**Options:**

* `-i`, `--instruction-type <INSTRUCTION_TYPE>` — The instruction type
* `-n`, `--node-types <NODE_TYPES>` — The node types



# `stencila prompts update`

Update builtin prompts

**Usage:** `stencila prompts update`



# `stencila prompts reset`

Reset builtin prompts

Re-initializes the builtin prompts directory to those prompts embedded in this version of Stencila

**Usage:** `stencila prompts reset`



# `stencila models`

Manage generative models

**Usage:** `stencila models [COMMAND]`

**Subcommands:**

* `list` — List the models available
* `run` — Run a model task



# `stencila models list`

List the models available

**Usage:** `stencila models list [OPTIONS]`

**Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila models run`

Run a model task

Mainly intended for testing of model selection and routing. Displays the task sent to the model and the generated output returned from it.

**Usage:** `stencila models run [OPTIONS] <PROMPT>`

**Arguments:**

* `<PROMPT>`

**Options:**

* `-m`, `--model <MODEL>` — The id pattern to specify the model to use
* `--dry-run` — Perform a dry run



# `stencila kernels`

Manage execution kernels

**Usage:** `stencila kernels [COMMAND]`

**Subcommands:**

* `list` — List the kernels available
* `info` — Get information about a kernel
* `packages` — List packages available to a kernel
* `execute` — Execute code in a kernel
* `evaluate` — Evaluate a code expression in a kernel
* `lint` — Lint code using the linting tool/s associated with a kernel



# `stencila kernels list`

List the kernels available

**Usage:** `stencila kernels list [OPTIONS]`

**Options:**

* `-t`, `--type <TYPE>` — Only list kernels of a particular type

  Possible values: `programming`, `templating`, `diagrams`, `math`, `styling`

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila kernels info`

Get information about a kernel

Mainly used to check the version of the kernel runtime and operating system for debugging purpose.

**Usage:** `stencila kernels info <NAME>`

**Arguments:**

* `<NAME>` — The name of the kernel to get information for



# `stencila kernels packages`

List packages available to a kernel

Mainly used to check libraries available to a kernel for debugging purpose.

**Usage:** `stencila kernels packages <NAME> [FILTER]`

**Arguments:**

* `<NAME>` — The name of the kernel to list packages for
* `<FILTER>` — A filter on the name of the kernel

   Only packages whose name contains this string will be included (case insensitive)



# `stencila kernels execute`

Execute code in a kernel

Creates a temporary kernel instance, executes one or more lines of code, and returns any outputs and execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels execute <NAME> <CODE>`

**Arguments:**

* `<NAME>` — The name of the kernel to execute code in
* `<CODE>` — The code to execute

   Escaped newline characters (i.e. "\n") in the code will be transformed into new lines before passing to the kernel.



# `stencila kernels evaluate`

Evaluate a code expression in a kernel

Creates a temporary kernel instance, evaluates the expression in it, and returns the output and any execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels evaluate <NAME> <CODE>`

**Arguments:**

* `<NAME>` — The name of the kernel to evaluate code in
* `<CODE>` — The code expression to evaluate



# `stencila kernels lint`

Lint code using the linting tool/s associated with a kernel

Note that this does not affect the file. It only prints how it would be formatted/fixed and any diagnostics.

Mainly intended for testing of linting by kernels during development of Stencila.

**Usage:** `stencila kernels lint [OPTIONS] <FILE>`

**Arguments:**

* `<FILE>` — The file to lint

**Options:**

* `--format` — Format the code
* `--fix` — Fix warnings and errors where possible



# `stencila codecs`

Manage format conversion codecs

**Usage:** `stencila codecs [COMMAND]`

**Subcommands:**

* `list` — List the codecs available



# `stencila codecs list`

List the codecs available

**Usage:** `stencila codecs list [OPTIONS]`

**Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




# `stencila plugins`

Manage plugins

**Usage:** `stencila plugins [COMMAND]`

**Subcommands:**

* `list` — List plugins
* `install` — Install a plugin
* `uninstall` — Uninstall a plugin
* `link` — Link to a local plugin
* `enable` — Enable a plugin
* `disable` — Disable a plugin
* `show` — Show details of a plugin
* `check` — Check a plugin



# `stencila plugins list`

List plugins

**Usage:** `stencila plugins list [OPTIONS]`

**Options:**

* `-r`, `--refresh` — Force refresh of plugin manifests
* `--installed` — Only list installed plugins
* `--installable` — Only list installable plugins
* `-o`, `--outdated` — Only list installed but outdated plugins
* `-e`, `--enabled` — Only list installed and enabled plugins



# `stencila plugins install`

Install a plugin

**Usage:** `stencila plugins install <NAME>`

**Arguments:**

* `<NAME>` — The name or URL of the plugin to install

   If a URL is supplied it should be a URL to the manifest TOML file of the plugin. e.g. https://example.org/plugin/stencila-plugin.toml



# `stencila plugins uninstall`

Uninstall a plugin

**Usage:** `stencila plugins uninstall <NAME>`

**Arguments:**

* `<NAME>` — The name of the plugin to uninstall



# `stencila plugins link`

Link to a local plugin

**Usage:** `stencila plugins link <DIRECTORY>`

**Arguments:**

* `<DIRECTORY>` — The directory to link to



# `stencila plugins enable`

Enable a plugin

**Usage:** `stencila plugins enable <NAME>`

**Arguments:**

* `<NAME>` — The name of the plugin to enable



# `stencila plugins disable`

Disable a plugin

**Usage:** `stencila plugins disable <NAME>`

**Arguments:**

* `<NAME>` — The name of the plugin to disable



# `stencila plugins show`

Show details of a plugin

**Usage:** `stencila plugins show <NAME>`

**Arguments:**

* `<NAME>` — The name of the plugin to install



# `stencila plugins check`

Check a plugin

**Usage:** `stencila plugins check [OPTIONS] <NAME>`

**Arguments:**

* `<NAME>` — The name of the plugin to install

**Options:**

* `--skip-codecs` — Skip checking codecs
* `--skip-kernels` — Skip checking kernels
* `--skip-models` — Skip checking models



# `stencila secrets`

Manage secrets

**Usage:** `stencila secrets [COMMAND]`

**Subcommands:**

* `list` — List the secrets used by Stencila
* `set` — Set a secret used by Stencila
* `delete` — Delete a secret previously set using Stencila



# `stencila secrets list`

List the secrets used by Stencila

**Usage:** `stencila secrets list`



# `stencila secrets set`

Set a secret used by Stencila

You will be prompted for the secret. Alternatively, you can echo the password into this command i.e. `echo <TOKEN> | stencila secrets set STENCILA_API_TOKEN`

**Usage:** `stencila secrets set <NAME>`

**Arguments:**

* `<NAME>` — The name of the secret



# `stencila secrets delete`

Delete a secret previously set using Stencila

**Usage:** `stencila secrets delete <NAME>`

**Arguments:**

* `<NAME>` — The name of the secret



# `stencila upgrade`

Upgrade to the latest version

**Usage:** `stencila upgrade [OPTIONS]`

**Options:**

* `-f`, `--force` — Perform upgrade even if the current version is the latest
* `-c`, `--check` — Check for an available upgrade but do not install it



# `stencila uninstall`

Uninstall this command line tool

**Usage:** `stencila uninstall`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>


