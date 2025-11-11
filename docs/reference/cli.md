# Command-Line Help for `stencila`

This document contains the help content for the `stencila` command-line program.

**Command Overview:**

* [`stencila`↴](#stencila)
* [`stencila new`↴](#stencila-new)
* [`stencila init`↴](#stencila-init)
* [`stencila config`↴](#stencila-config)
* [`stencila status`↴](#stencila-status)
* [`stencila move`↴](#stencila-move)
* [`stencila track`↴](#stencila-track)
* [`stencila untrack`↴](#stencila-untrack)
* [`stencila clean`↴](#stencila-clean)
* [`stencila convert`↴](#stencila-convert)
* [`stencila merge`↴](#stencila-merge)
* [`stencila sync`↴](#stencila-sync)
* [`stencila push`↴](#stencila-push)
* [`stencila pull`↴](#stencila-pull)
* [`stencila watch`↴](#stencila-watch)
* [`stencila unwatch`↴](#stencila-unwatch)
* [`stencila compile`↴](#stencila-compile)
* [`stencila lint`↴](#stencila-lint)
* [`stencila execute`↴](#stencila-execute)
* [`stencila render`↴](#stencila-render)
* [`stencila query`↴](#stencila-query)
* [`stencila open`↴](#stencila-open)
* [`stencila publish`↴](#stencila-publish)
* [`stencila publish zenodo`↴](#stencila-publish-zenodo)
* [`stencila publish ghost`↴](#stencila-publish-ghost)
* [`stencila publish stencila`↴](#stencila-publish-stencila)
* [`stencila demo`↴](#stencila-demo)
* [`stencila db`↴](#stencila-db)
* [`stencila db new`↴](#stencila-db-new)
* [`stencila db add`↴](#stencila-db-add)
* [`stencila db remove`↴](#stencila-db-remove)
* [`stencila db query`↴](#stencila-db-query)
* [`stencila db migrate`↴](#stencila-db-migrate)
* [`stencila db migrations`↴](#stencila-db-migrations)
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
* [`stencila linters`↴](#stencila-linters)
* [`stencila linters list`↴](#stencila-linters-list)
* [`stencila linters lint`↴](#stencila-linters-lint)
* [`stencila formats`↴](#stencila-formats)
* [`stencila formats list`↴](#stencila-formats-list)
* [`stencila formats structuring`↴](#stencila-formats-structuring)
* [`stencila themes`↴](#stencila-themes)
* [`stencila themes list`↴](#stencila-themes-list)
* [`stencila themes show`↴](#stencila-themes-show)
* [`stencila themes new`↴](#stencila-themes-new)
* [`stencila themes remove`↴](#stencila-themes-remove)
* [`stencila secrets`↴](#stencila-secrets)
* [`stencila secrets list`↴](#stencila-secrets-list)
* [`stencila secrets set`↴](#stencila-secrets-set)
* [`stencila secrets delete`↴](#stencila-secrets-delete)
* [`stencila tools`↴](#stencila-tools)
* [`stencila tools list`↴](#stencila-tools-list)
* [`stencila tools show`↴](#stencila-tools-show)
* [`stencila tools install`↴](#stencila-tools-install)
* [`stencila tools env`↴](#stencila-tools-env)
* [`stencila tools run`↴](#stencila-tools-run)
* [`stencila serve`↴](#stencila-serve)
* [`stencila snap`↴](#stencila-snap)
* [`stencila lsp`↴](#stencila-lsp)
* [`stencila cloud`↴](#stencila-cloud)
* [`stencila cloud status`↴](#stencila-cloud-status)
* [`stencila cloud signin`↴](#stencila-cloud-signin)
* [`stencila cloud signout`↴](#stencila-cloud-signout)
* [`stencila cloud logs`↴](#stencila-cloud-logs)
* [`stencila signin`↴](#stencila-signin)
* [`stencila signout`↴](#stencila-signout)
* [`stencila logs`↴](#stencila-logs)
* [`stencila upgrade`↴](#stencila-upgrade)
* [`stencila uninstall`↴](#stencila-uninstall)

## `stencila`

CLI subcommands and global options

**Usage:** `stencila [OPTIONS] <COMMAND>`

Examples
  # Get help on all available commands
  stencila --help

  # Create a new document
  stencila new article.md

  # Convert a document to another format
  stencila convert input.md output.pdf

  # Check available formats
  stencila formats list

  # Execute a document
  stencila execute notebook.myst

  # Open a document in the browser
  stencila open document.md


###### **Subcommands:**

* `new` — Create a new, tracked, document
* `init` — Initialize a workspace
* `config` — Display the configuration for a document
* `status` — Get the tracking status of documents
* `move` — Move a tracked document
* `track` — Start tracking a document
* `untrack` — Stop tracking a document
* `clean` — Clean the current workspace
* `convert` — Convert a document to another format
* `merge` — Merge changes from another format
* `sync` — Synchronize a document between formats
* `push` — Push a document to a remote service
* `pull` — Pull a document from a remote service
* `watch` — Enable automatic sync between a document and its remote
* `unwatch` — Disable automatic sync for a document
* `compile` — Compile a document
* `lint` — Lint one or more documents
* `execute` — Execute a document
* `render` — Render a document
* `query` — Query a workspace database
* `open` — Open a document in the browser
* `publish` — Publish one or more documents
* `demo` — Run a terminal demonstration from a document
* `db` — Manage the workspace and other document databases
* `prompts` — Manage prompts
* `models` — Manage and interact with generative AI models
* `kernels` — Manage execution kernels
* `linters` — Manage linters
* `formats` — List and inspect supported formats
* `themes` — Manage themes
* `secrets` — Manage secrets
* `tools` — Manage tools and environments used by Stencila
* `serve` — Run the HTTP/Websocket server
* `snap` — Capture screenshots and measurements of documents served by Stencila
* `lsp` — Run the Language Server Protocol server
* `cloud` — Manage Stencila Cloud account
* `signin` — Sign in to Stencila Cloud
* `signout` — Sign out from Stencila Cloud
* `logs` — Display logs from Stencila Cloud workspace sessions
* `upgrade` — Upgrade to the latest version
* `uninstall` — Uninstall this command line tool

###### **Options:**

* `-h`, `--help` — Print help: `-h` for brief help, `--help` for more details

  Possible values: `true`, `false`

* `--yes` — Assume the answer `yes` to any interactive prompts

   The unlisted options `--no` and `--cancel` (and corresponding env vars) are also available.
* `--debug` — Display debug level logging and detailed error reports

   For trace level logging, use the unlisted --trace option. See documentation for other unlisted logging options --log-level, --log-format, log-filter.
* `--no-color` — Do not color any output



## `stencila new`

Create a new, tracked, document

**Usage:** `stencila new [OPTIONS] <PATH>`

Examples
  # Create a new article (default)
  stencila new my-article.md

  # Create a new chat document
  stencila new conversation.md --type chat

  # Create a new AI prompt
  stencila new template.md --type prompt

  # Create a document in a subdirectory
  stencila new docs/report.md

  # Overwrite an existing document
  stencila new existing.md --force


###### **Arguments:**

* `<PATH>` — The path of the document to create

###### **Options:**

* `-f`, `--force` — Overwrite the document, if it already exists
* `-t`, `--type <TYPE>` — The type of document to create

  Default value: `article`

  Possible values: `article`, `chat`, `prompt`




## `stencila init`

Initialize a workspace

**Usage:** `stencila init [OPTIONS] [DIR]`


  # Initialize current directory as a Stencila workspace
  stencila init

  # Initialize a specific directory
  stencila init ./my-project

  # Initialize without creating .gitignore
  stencila init --no-gitignore

Note
  This creates a .stencila directory for workspace configuration
  and document tracking. A .gitignore file is created by default
  to exclude tracking and cache files.


###### **Arguments:**

* `<DIR>` — The workspace directory to initialize

   Defaults to the current directory.

  Default value: `.`

###### **Options:**

* `--no-gitignore` — Do not create a `.gitignore` file



## `stencila config`

Display the configuration for a document

**Usage:** `stencila config <FILE>`

Examples
  # Show configuration for a document
  stencila config document.md

Note
  Shows both the configuration sources (from workspace,
  user, and document-specific configs) and the final
  merged configuration that will be used for the document.


###### **Arguments:**

* `<FILE>` — The path to the document to resolve



## `stencila status`

Get the tracking status of documents

**Usage:** `stencila status [OPTIONS] [FILES]...`

Examples
  # Show status of all tracked documents (includes watch details by default)
  stencila status

  # Show status of specific documents
  stencila status document.md report.md

  # Output status as JSON
  stencila status --as json

  # Skip fetching remote status (faster)
  stencila status --no-remotes
  
  # Skip fetching watch status (faster)
  stencila status --no-watches


###### **Arguments:**

* `<FILES>` — The paths of the files to get status for

###### **Options:**

* `-a`, `--as <AS>` — Output the status as JSON or YAML

  Possible values: `json`, `yaml`

* `--no-remotes` — Skip fetching remote status
* `--no-watches` — Skip fetching watch status



## `stencila move`

Move a tracked document

Moves the document file to the new path (if it still exists at the old path) and updates any tracking information.

**Usage:** `stencila move [OPTIONS] <FROM> <TO>`

Examples
  # Move a tracked document
  stencila move old-name.md new-name.md

  # Move to a different directory
  stencila move document.md docs/document.md

  # Force overwrite if destination exists
  stencila move source.md target.md --force

  # Use the mv alias
  stencila mv old.md new.md

Note
  This updates both the file system and tracking
  information. If the destination already exists,
  you'll be prompted unless --force is used.


###### **Arguments:**

* `<FROM>` — The old path of the file
* `<TO>` — The new path of the file

###### **Options:**

* `-f`, `--force` — Overwrite the destination path if it already exists



## `stencila track`

Start tracking a document

**Usage:** `stencila track <FILE> [URL]`

Examples
  # Start tracking a local document
  stencila track document.md

  # Track a document with remote URL
  stencila track document.md https://example.com/api/docs/123

  # Track multiple documents
  stencila track *.md

Note
  Tracking enables version control, synchronization,
  and change detection for documents. Remote URLs allow
  syncing with external systems.


###### **Arguments:**

* `<FILE>` — The path to the local file to track
* `<URL>` — The URL of the remote to track



## `stencila untrack`

Stop tracking a document

**Usage:** `stencila untrack <FILE> [URL]`

Examples
  # Stop tracking a document
  stencila untrack document.md

  # Stop tracking a remote URL for a document
  stencila untrack document.md https://example.com/api/docs/123

  # Stop tracking all tracked files
  stencila untrack all

Note
  This removes the document from tracking but does not
  delete the file itself.


###### **Arguments:**

* `<FILE>` — The path of the file to stop tracking

   Use "deleted" to untrack all files that have been deleted.
* `<URL>` — The URL of the remote to stop tracking



## `stencila clean`

Clean the current workspace

Un-tracks any deleted files and removes any unnecessary cache files, and all artifact directories, from the .stencila folder in the current workspace.

**Usage:** `stencila clean`

Examples
  # Clean the .stencila folder for the current workspace
  stencila clean




## `stencila convert`

Convert a document to another format

**Usage:** `stencila convert [OPTIONS] [INPUT] [OUTPUTS]... [-- <TOOL_ARGS>...]`

Examples
  # Convert Stencila Markdown to MyST Markdown
  stencila convert document.smd document.myst

  # Convert to multiple output formats
  stencila convert input.smd output.html output.pdf output.docx

  # Specify input and output formats explicitly
  stencila convert input.txt output.json --from plain --to json

  # Convert with specific codec options
  stencila convert doc.md doc.html --standalone

  # Convert only specific pages from a PDF
  stencila convert document.pdf extract.md --pages 1,3,5-10

  # Convert all pages except specific ones
  stencila convert report.pdf content.md --exclude-pages 5,15

  # Convert only odd pages from a document
  stencila convert book.pdf odd-pages.md --pages odd

  # Use an external tool like Pandoc
  stencila convert doc.md doc.tex --tool pandoc

  # Pass arguments to external tool
  stencila convert doc.md doc.pdf --tool pandoc -- --pdf-engine=xelatex

  # Convert from stdin to stdout (defaults to JSON)
  echo "# Hello" | stencila convert


###### **Arguments:**

* `<INPUT>` — The path, URL or other identifier for the input file

   If not supplied, or if "-", the input content is read from `stdin`.
* `<OUTPUTS>` — The paths of desired output files

   Each output may be of a different format (inferred from the extension). If the `--to` format option is used it will apply to all outputs. If no output paths supplied, or if "-", the output content is written to `stdout`.
* `<TOOL_ARGS>` — Arguments to pass through to the tool using for encoding

   Only supported for formats that use external tools for encoding and ignored otherwise. Note: these arguments are not used for decoding from the input, only for encoding to the output.

###### **Options:**

* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

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
    Strip archive properties of nodes
  - `temporary`:
    Strip temporary properties of nodes
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
* `-t`, `--to <TO>` — The format of the output/s

   If not supplied, and outputting to a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--template <TEMPLATE>` — The template document to use

   Only supported by some formats (e.g. DOCX).
* `--reproducible` — Encode executable nodes so that they are reproducible

   Encode links to the source of executable nodes so that edits made to rendered documents can be merged back to the source document.

   Only supported by some formats, and may be the default for those.
* `--highlight` — Highlight the rendered outputs of executable nodes

   Only supported by some formats (e.g. DOCX and ODT). Defaults to `true` when `--reproducible` flag is used.
* `--no-highlight` — Do not highlight the rendered outputs of executable nodes
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `--theme <THEME>` — The CSS theme to use when encoding to HTML and HTML-derived formats

   Use this option to specify the theme for HTML and HTML-derived (e.g. PDF) formats.
* `--view <VIEW>` — The document view to use when encoding to HTML and HTML-derived formats

   Stencila provides alternatives views with alternative ways of interacting with a document (e.g. "dynamic", "static", "none").
* `--embed-media` — Embed media files as data URIs

   When enabled, external media files (images, audio, video) referenced in the document will be converted to data URIs and embedded directly in the output. This creates a self-contained document but may increase file size significantly. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--extract-media`.
* `--extract-media <FOLDER>` — Extract embedded media to a folder

   Depending on the format, this is often the default when encoding to files. When provided, any data URIs in the document will be extracted to files in the specified directory, and the references will be updated to point to these external files. This reduces document size but creates external dependencies. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--embed-media`.
* `--embed-supplements` — Embed supplemental files directly into the document

   When enabled, supplemental files referenced in the document will be decoded and embedded directly into the output. Supports CSV, DOCX, XLSX, PDF, Jupyter notebooks, LaTeX, and media files. This creates a self-contained document but may increase file size significantly. Should not be used with `--extract-supplements`.
* `--extract-supplements <FOLDER>` — Extract embedded supplemental content to separate files

   When provided, any supplemental content embedded in the document will be extracted to files in the specified directory. Supplements are saved as `supplement-<N>.czst` files. This reduces document size but creates external file dependencies. Should not be used with `--embed-supplements`.
* `--recursive` — Recursively encode the content of `IncludeBlock`s to their source file

   Only supported when encoding to a path.
* `--compact` — Use a compact form of encoding if available

   Use this flag to produce a compact form of encoding if the format supports it. For formats such as JSON and HTML, this usually means no indentation. For Markdown-based formats, this means that embedded Base64 media will NOT be written to separate files in a media folder (the default behavior).
* `--pretty` — Use a "pretty" form of encoding if available

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
* `--output-losses <OUTPUT_LOSSES>` — Action when there are losses encoding to output files

   See help for `--input-losses` for details.

  Default value: `debug`
* `--from-tool <FROM_TOOL>` — The tool to use for decoding inputs

   Only supported for formats that use alternative external tools for decoding inputs and ignored otherwise. Use `--tool` for specifying the tool to use for encoding outputs.
* `--tool <TOOL>` — The tool to use for encoding outputs (e.g. pandoc)

   Only supported for formats that use alternative external tools for encoding and ignored otherwise. Use `--from-tool` for specifying the tool to use for decoding inputs.



## `stencila merge`

Merge changes from another format

**Usage:** `stencila merge [OPTIONS] <EDITED>`

Examples
  # Merge changes from an edited DOCX back to Stencila Markdown
  stencila merge edited.docx --original document.smd

  # Merge with both original and unedited versions specified
  stencila merge edited.docx --original source.qmd --unedited generated.docx

  # Merge changes from a specific Git commit
  stencila merge edited.docx --original document.myst --commit abc123

  # Merge with custom working directory for inspection
  stencila merge edited.docx --original document.md --workdir ./merge-work


###### **Arguments:**

* `<EDITED>` — The edited version of the document

###### **Options:**

* `--original <ORIGINAL>` — The original source of the document

   This file may be in a different same format to the edited version.
* `--unedited <UNEDITED>` — The unedited version of the document

   This file should be in the same format as the edited version.
* `--commit <COMMIT>` — The commit at which the edited document was generated from the original
* `--no-rebase` — Do not rebase edits using the unedited version of the document
* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

* `-t`, `--to <TO>` — The format of the output/s

   If not supplied, and outputting to a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--template <TEMPLATE>` — The template document to use

   Only supported by some formats (e.g. DOCX).
* `--reproducible` — Encode executable nodes so that they are reproducible

   Encode links to the source of executable nodes so that edits made to rendered documents can be merged back to the source document.

   Only supported by some formats, and may be the default for those.
* `--highlight` — Highlight the rendered outputs of executable nodes

   Only supported by some formats (e.g. DOCX and ODT). Defaults to `true` when `--reproducible` flag is used.
* `--no-highlight` — Do not highlight the rendered outputs of executable nodes
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `--theme <THEME>` — The CSS theme to use when encoding to HTML and HTML-derived formats

   Use this option to specify the theme for HTML and HTML-derived (e.g. PDF) formats.
* `--view <VIEW>` — The document view to use when encoding to HTML and HTML-derived formats

   Stencila provides alternatives views with alternative ways of interacting with a document (e.g. "dynamic", "static", "none").
* `--embed-media` — Embed media files as data URIs

   When enabled, external media files (images, audio, video) referenced in the document will be converted to data URIs and embedded directly in the output. This creates a self-contained document but may increase file size significantly. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--extract-media`.
* `--extract-media <FOLDER>` — Extract embedded media to a folder

   Depending on the format, this is often the default when encoding to files. When provided, any data URIs in the document will be extracted to files in the specified directory, and the references will be updated to point to these external files. This reduces document size but creates external dependencies. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--embed-media`.
* `--embed-supplements` — Embed supplemental files directly into the document

   When enabled, supplemental files referenced in the document will be decoded and embedded directly into the output. Supports CSV, DOCX, XLSX, PDF, Jupyter notebooks, LaTeX, and media files. This creates a self-contained document but may increase file size significantly. Should not be used with `--extract-supplements`.
* `--extract-supplements <FOLDER>` — Extract embedded supplemental content to separate files

   When provided, any supplemental content embedded in the document will be extracted to files in the specified directory. Supplements are saved as `supplement-<N>.czst` files. This reduces document size but creates external file dependencies. Should not be used with `--embed-supplements`.
* `--recursive` — Recursively encode the content of `IncludeBlock`s to their source file

   Only supported when encoding to a path.
* `--compact` — Use a compact form of encoding if available

   Use this flag to produce a compact form of encoding if the format supports it. For formats such as JSON and HTML, this usually means no indentation. For Markdown-based formats, this means that embedded Base64 media will NOT be written to separate files in a media folder (the default behavior).
* `--pretty` — Use a "pretty" form of encoding if available

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
* `--output-losses <OUTPUT_LOSSES>` — Action when there are losses encoding to output files

   See help for `--input-losses` for details.

  Default value: `debug`



## `stencila sync`

Synchronize a document between formats

The direction of synchronization can be specified by appending the to the file path:

- `:in` only sync incoming changes from the file - `:out` only sync outgoing changes to the file - `:io` sync incoming and outgoing changes (default)

**Usage:** `stencila sync [OPTIONS] <DOC> [FILES]...`

Examples
  # Sync a Markdown document with HTML (bidirectional)
  stencila sync document.md preview.html

  # Sync with multiple formats
  stencila sync source.md output.html output.pdf

  # Sync only incoming changes from HTML
  stencila sync document.md edited.html:in

  # Sync only outgoing changes to PDF
  stencila sync document.md output.pdf:out

  # Mixed sync directions
  stencila sync main.md preview.html:out edits.docx:in

  # Sync with custom encoding options
  stencila sync doc.md output.html --standalone

Sync Directions
  • :in - Only accept incoming changes from the file
  • :out - Only push outgoing changes to the file
  • :io - Bidirectional sync (default)

Note
  The sync command runs continuously, watching for changes.
  Press Ctrl+C to stop synchronization.


###### **Arguments:**

* `<DOC>` — The path of the document to synchronize
* `<FILES>` — The files to synchronize with

###### **Options:**

* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

* `-t`, `--to <TO>` — The format of the output/s

   If not supplied, and outputting to a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--template <TEMPLATE>` — The template document to use

   Only supported by some formats (e.g. DOCX).
* `--reproducible` — Encode executable nodes so that they are reproducible

   Encode links to the source of executable nodes so that edits made to rendered documents can be merged back to the source document.

   Only supported by some formats, and may be the default for those.
* `--highlight` — Highlight the rendered outputs of executable nodes

   Only supported by some formats (e.g. DOCX and ODT). Defaults to `true` when `--reproducible` flag is used.
* `--no-highlight` — Do not highlight the rendered outputs of executable nodes
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `--theme <THEME>` — The CSS theme to use when encoding to HTML and HTML-derived formats

   Use this option to specify the theme for HTML and HTML-derived (e.g. PDF) formats.
* `--view <VIEW>` — The document view to use when encoding to HTML and HTML-derived formats

   Stencila provides alternatives views with alternative ways of interacting with a document (e.g. "dynamic", "static", "none").
* `--embed-media` — Embed media files as data URIs

   When enabled, external media files (images, audio, video) referenced in the document will be converted to data URIs and embedded directly in the output. This creates a self-contained document but may increase file size significantly. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--extract-media`.
* `--extract-media <FOLDER>` — Extract embedded media to a folder

   Depending on the format, this is often the default when encoding to files. When provided, any data URIs in the document will be extracted to files in the specified directory, and the references will be updated to point to these external files. This reduces document size but creates external dependencies. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--embed-media`.
* `--embed-supplements` — Embed supplemental files directly into the document

   When enabled, supplemental files referenced in the document will be decoded and embedded directly into the output. Supports CSV, DOCX, XLSX, PDF, Jupyter notebooks, LaTeX, and media files. This creates a self-contained document but may increase file size significantly. Should not be used with `--extract-supplements`.
* `--extract-supplements <FOLDER>` — Extract embedded supplemental content to separate files

   When provided, any supplemental content embedded in the document will be extracted to files in the specified directory. Supplements are saved as `supplement-<N>.czst` files. This reduces document size but creates external file dependencies. Should not be used with `--embed-supplements`.
* `--recursive` — Recursively encode the content of `IncludeBlock`s to their source file

   Only supported when encoding to a path.
* `--compact` — Use a compact form of encoding if available

   Use this flag to produce a compact form of encoding if the format supports it. For formats such as JSON and HTML, this usually means no indentation. For Markdown-based formats, this means that embedded Base64 media will NOT be written to separate files in a media folder (the default behavior).
* `--pretty` — Use a "pretty" form of encoding if available

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
* `--output-losses <OUTPUT_LOSSES>` — Action when there are losses encoding to output files

   See help for `--input-losses` for details.

  Default value: `debug`
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
    Strip archive properties of nodes
  - `temporary`:
    Strip temporary properties of nodes
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



## `stencila push`

Push a document to a remote service

**Usage:** `stencila push [OPTIONS] [PATH] [TARGET] [-- <ARGS>...]`

Examples
  # Push all files with tracked remotes
  stencila push

  # Push a document to Google Docs
  stencila push document.smd gdoc

  # Push a document to Microsoft 365
  stencila push document.smd m365

  # Push to file to all tracked remotes
  stencila push document.smd

  # Push to specific remote
  stencila push document.smd https://docs.google.com/document/d/abc123

  # Push with execution first
  stencila push report.smd gdoc -- arg1=value1

  # Force create new document
  stencila push document.smd gdoc --force-new


###### **Arguments:**

* `<PATH>` — The path of the document to push

   If omitted, pushes all tracked files that have remotes.
* `<TARGET>` — The target to push to

   Can be a full URL (e.g., https://docs.google.com/document/d/...) or a service shorthand (e.g "gdoc" or "m365"). Omit to push to all tracked remotes for the path.
* `<ARGS>` — Arguments to pass to the document for execution

   If provided, the document will be executed with these arguments before being pushed. Use -- to separate these from other options.

###### **Options:**

* `-n`, `--force-new` — Create a new document instead of updating an existing one

   By default, if a remote is already tracked for the document, it will be updated. Use this flag to create a new document.
* `--no-execute` — Do not execute the document before pushing it

   By default, the document will be executed to ensure that it is up-to-date before pushing it. Use this flag to skip execution.
* `-w`, `--watch` — Enable watch after successful push

   Creates a watch in Stencila Cloud to automatically sync changes between the remote and repository via pull requests.
* `-d`, `--direction <DIRECTION>` — The sync direction (only used with --watch)

  Possible values:
  - `bi`:
    Bi-directional sync: changes from remote create PRs, changes to repo push to remote
  - `from-remote`:
    One-way sync from remote: only remote changes create PRs
  - `to-remote`:
    One-way sync to remote: only repo changes push to remote

* `-p`, `--pr-mode <PR_MODE>` — The GitHub PR mode (only used with --watch)

  Possible values:
  - `draft`:
    Create PRs as drafts (default)
  - `ready`:
    Create PRs ready for review

* `--debounce-seconds <DEBOUNCE_SECONDS>` — Debounce time in seconds (10-86400, only used with --watch)

   Time to wait after detecting changes before syncing to avoid too frequent updates. Minimum 10 seconds, maximum 24 hours (86400 seconds).



## `stencila pull`

Pull a document from a remote service

**Usage:** `stencila pull [OPTIONS] <PATH> [TARGET]`

Examples
  # Pull from the tracked remote (if only one exists)
  stencila pull document.smd

  # Pull from tracked Google Doc
  stencila pull document.smd gdoc

  # Pull from tracked Microsoft 365 document
  stencila pull document.smd m365

  # Pull from specific URL
  stencila pull document.smd https://docs.google.com/document/d/abc123

  # Pull without merging (replace local file)
  stencila pull document.smd gdoc --no-merge


###### **Arguments:**

* `<PATH>` — The path to the local document
* `<TARGET>` — The target to pull from

   Can be a full URL (e.g., https://docs.google.com/document/d/...) or a service shorthand (e.g "gdoc" or "m365"). Omit to use the tracked remote (errors if multiple remotes are tracked).

###### **Options:**

* `--no-merge` — Do not merge, just replace

   By default, the pulled document will be merged with the local version. Use this flag to skip merging and just replace the local file.



## `stencila watch`

Enable automatic sync between a document and its remote

Creates a watch in Stencila Cloud that automatically syncs changes between a remote (Google Docs or M365) and a GitHub repository. When changes are detected in the remote, a pull request will be created or updated in the repository.

**Usage:** `stencila watch [OPTIONS] <PATH> [TARGET]`

Examples
  # Enable watch on the tracked remote
  stencila watch report.md

  # Watch a specific remote (if document has multiple)
  stencila watch report.md gdoc
  stencila watch report.md https://docs.google.com/document/d/abc123

  # Enable watch with one-way sync from remote
  stencila watch report.md gdoc --direction from-remote

  # Enable watch with ready-for-review PRs
  stencila watch report.md gdoc --pr-mode ready

  # Note: The document must already be pushed to a remote
  stencila push report.md gdoc
  stencila watch report.md


###### **Arguments:**

* `<PATH>` — The path to the document to watch
* `<TARGET>` — The target remote to watch

   If the document has multiple remotes (e.g., both Google Docs and M365), you must specify which one to watch. Can be the full URL or a service shorthand: "gdoc" or "m365".

###### **Options:**

* `-d`, `--direction <DIRECTION>` — The sync direction

  Possible values:
  - `bi`:
    Bi-directional sync: changes from remote create PRs, changes to repo push to remote
  - `from-remote`:
    One-way sync from remote: only remote changes create PRs
  - `to-remote`:
    One-way sync to remote: only repo changes push to remote

* `-p`, `--pr-mode <PR_MODE>` — The GitHub PR mode

  Possible values:
  - `draft`:
    Create PRs as drafts (default)
  - `ready`:
    Create PRs ready for review

* `--debounce-seconds <DEBOUNCE_SECONDS>` — Debounce time in seconds (10-86400)

   Time to wait after detecting changes before syncing to avoid too frequent updates. Minimum 10 seconds, maximum 24 hours (86400 seconds).



## `stencila unwatch`

Disable automatic sync for a document

Removes the watch from Stencila Cloud, stopping automatic sync.

**Usage:** `stencila unwatch <PATH> [TARGET]`

Examples
  # Disable watch for a document
  stencila unwatch report.md

  # Unwatch a specific remote (if document has multiple)
  stencila unwatch report.md gdoc

  # Note: Remote linkage is preserved, you can re-enable watch later
  stencila unwatch report.md
  stencila watch report.md


###### **Arguments:**

* `<PATH>` — The path to the document to unwatch
* `<TARGET>` — The target remote to unwatch

   If the document has multiple watched remotes, you must specify which one to unwatch. Can be the full URL or a service shorthand: "gdoc" or "m365".



## `stencila compile`

Compile a document

**Usage:** `stencila compile [OPTIONS] <INPUT>`

Examples
  # Compile a document to check for errors
  stencila compile document.md

  # Compile without updating in document store
  stencila compile temp.md --no-store

Note
  Compiling a document checks for source path errors in
  include and call blocks and prepares the document for
  execution without actually running any code.


###### **Arguments:**

* `<INPUT>` — The path of the document to compile

###### **Options:**

* `--no-save` — Do not save the document after compiling it
* `--no-store` — Do not store the document after compiling it
* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹




## `stencila lint`

Lint one or more documents

**Usage:** `stencila lint [OPTIONS] [FILES]...`

Examples
  # Lint a single document
  stencila lint document.smd

  # Lint multiple documents
  stencila lint *.qmd docs/*

  # Auto-format documents during linting
  stencila lint report.myst --format

  # Auto-fix linting issues
  stencila lint article.smd --fix

  # Output diagnostics as YAML
  stencila lint article.myst --as yaml


###### **Arguments:**

* `<FILES>` — The files to lint

###### **Options:**

* `--format` — Format the file if necessary
* `--fix` — Fix any linting issues
* `--no-store` — Do not store the document after formatting and/or fixing it

   Only applies when using `--format` or `--fix`, both of which will write a modified version of the source document back to disk and by default, a new cache of the document to the store. This flag prevent the store being updated.
* `-a`, `--as <AS>` — Output any linting diagnostics as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila execute`

Execute a document

**Usage:** `stencila execute [OPTIONS] <INPUT>`

Examples
  # Execute a Stencila Markdown document
  stencila execute report.smd

  # Execute without updating the document store
  stencila execute temp.md --no-store

  # Force re-execution of all code
  stencila execute cached.ipynb --force-all

  # Execute using the shorthand alias
  stencila exec script.r


###### **Arguments:**

* `<INPUT>` — The path of the document to execute

###### **Options:**

* `--no-save` — Do not save the document after executing it
* `--no-store` — Do not store the document after executing it
* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

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



## `stencila render`

Render a document

**Usage:** `stencila render [OPTIONS] [INPUT] [OUTPUTS]... [-- <ARGUMENTS>...]`

Examples
  # Render a document and preview in browser
  stencila render document.smd

  # Render to a specific output format
  stencila render report.md report.docx

  # Render to multiple formats
  stencila render analysis.md output.html output.pdf

  # Render from stdin to stdout
  echo "# Hello" | stencila render --to html

  # Render with document parameters
  stencila render template.md output.html -- --name="John" --year=2024

  # Render ignoring execution errors
  stencila render notebook.md report.pdf --ignore-errors

  # Render without updating the document store
  stencila render temp.md output.html --no-store


###### **Arguments:**

* `<INPUT>` — The path of the document to render

   If not supplied, or if "-", the input content is read from `stdin` and assumed to be Markdown (but can be specified with the `--from` option). Note that the Markdown parser should handle alternative flavors so it may not be necessary to use the `--from` option for MyST, Quarto or Stencila Markdown.
* `<OUTPUTS>` — The paths of desired output files

   If an input was supplied, but no outputs, and the `--to` format option is not used, the document will be rendered in a browser window. If no outputs are supplied and the `--to` option is used the document will be rendered to `stdout` in that format.
* `<ARGUMENTS>` — Arguments to pass to the document

   The name of each argument is matched against the document's parameters. If a match is found, then the argument value is coerced to the expected type of the parameter. If no corresponding parameter is found, then the argument is parsed as JSON and set as a variable in the document's default kernel (usually the first programming language used in the document).

###### **Options:**

* `--ignore-errors` — Ignore any errors while executing document
* `--no-store` — Do not store the document after executing it
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
* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

* `-t`, `--to <TO>` — The format of the output/s

   If not supplied, and outputting to a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--template <TEMPLATE>` — The template document to use

   Only supported by some formats (e.g. DOCX).
* `--reproducible` — Encode executable nodes so that they are reproducible

   Encode links to the source of executable nodes so that edits made to rendered documents can be merged back to the source document.

   Only supported by some formats, and may be the default for those.
* `--highlight` — Highlight the rendered outputs of executable nodes

   Only supported by some formats (e.g. DOCX and ODT). Defaults to `true` when `--reproducible` flag is used.
* `--no-highlight` — Do not highlight the rendered outputs of executable nodes
* `--standalone` — Encode as a standalone document
* `--not-standalone` — Do not encode as a standalone document when writing to file
* `--theme <THEME>` — The CSS theme to use when encoding to HTML and HTML-derived formats

   Use this option to specify the theme for HTML and HTML-derived (e.g. PDF) formats.
* `--view <VIEW>` — The document view to use when encoding to HTML and HTML-derived formats

   Stencila provides alternatives views with alternative ways of interacting with a document (e.g. "dynamic", "static", "none").
* `--embed-media` — Embed media files as data URIs

   When enabled, external media files (images, audio, video) referenced in the document will be converted to data URIs and embedded directly in the output. This creates a self-contained document but may increase file size significantly. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--extract-media`.
* `--extract-media <FOLDER>` — Extract embedded media to a folder

   Depending on the format, this is often the default when encoding to files. When provided, any data URIs in the document will be extracted to files in the specified directory, and the references will be updated to point to these external files. This reduces document size but creates external dependencies. Currently respected for Markdown-flavors, LaTeX, HTML, and CBOR. Should not be used with `--embed-media`.
* `--embed-supplements` — Embed supplemental files directly into the document

   When enabled, supplemental files referenced in the document will be decoded and embedded directly into the output. Supports CSV, DOCX, XLSX, PDF, Jupyter notebooks, LaTeX, and media files. This creates a self-contained document but may increase file size significantly. Should not be used with `--extract-supplements`.
* `--extract-supplements <FOLDER>` — Extract embedded supplemental content to separate files

   When provided, any supplemental content embedded in the document will be extracted to files in the specified directory. Supplements are saved as `supplement-<N>.czst` files. This reduces document size but creates external file dependencies. Should not be used with `--embed-supplements`.
* `--recursive` — Recursively encode the content of `IncludeBlock`s to their source file

   Only supported when encoding to a path.
* `--compact` — Use a compact form of encoding if available

   Use this flag to produce a compact form of encoding if the format supports it. For formats such as JSON and HTML, this usually means no indentation. For Markdown-based formats, this means that embedded Base64 media will NOT be written to separate files in a media folder (the default behavior).
* `--pretty` — Use a "pretty" form of encoding if available

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).
* `--output-losses <OUTPUT_LOSSES>` — Action when there are losses encoding to output files

   See help for `--input-losses` for details.

  Default value: `debug`
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
    Strip archive properties of nodes
  - `temporary`:
    Strip temporary properties of nodes
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



## `stencila query`

Query a workspace database

**Usage:** `stencila query [OPTIONS] <FILE> <QUERY> [OUTPUT]`

Examples
  # Query a specific document
  stencila query article.qmd "paragraphs().sample(3)"

  # Query with output to file
  stencila query report.myst "headings(.level == 1)" headings.md

  # Use Cypher query language
  stencila query doc.ipynb --cypher "MATCH (h:Heading) WHERE h.level = 1 RETURN h"


###### **Arguments:**

* `<FILE>` — The document to query

   Use the path to a file to create a temporary database for that file to query.
* `<QUERY>` — The DocsQL or Cypher query to run

   If the query begins with the word `MATCH` it will be assumed to be cypher. Use the `--cypher` flag to force this.
* `<OUTPUT>` — The path of the file to output the result to

   If not supplied the output content is written to `stdout`.

###### **Options:**

* `-c`, `--cypher` — Use Cypher as the query language (instead of DocsQL the default)
* `--no-compile` — Do not compile the document before querying it

   By default, the document is compiled before it is loaded into the database. This means that if it has any `IncludeBlock` nodes that their included content will be included in the database. Use this flag to turn off this behavior.
* `-t`, `--to <TO>` — The format to output the result as

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to JSON. See `stencila codecs list` for available formats.
* `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).



## `stencila open`

Open a document in the browser

Opens a document in the browser. If the path supplied is a folder then the first file with name `index.*`, `main.*`, or `readme.*` will be opened.

By default, opens both a local preview server and any tracked remote URLs (e.g., Google Docs, Microsoft 365). Use the `target` argument to open only a specific remote (by service shorthand like "gdoc" or "m365", or by full URL), or use "local" to open only the local preview. Alternatively, use `--no-local` or `--no-remotes` to open only one or the other.

When `--sync=in` (the default) the local preview will update when the document is changed and saved to disk.

**Usage:** `stencila open [OPTIONS] [PATH] [TARGET]`

Examples
  # Open a specific document (all remotes + local)
  stencila open document.md

  # Open current directory (finds index/main/readme)
  stencila open

  # Open only Google Docs remote
  stencila open document.md gdoc

  # Open only Microsoft 365 remote
  stencila open document.md m365

  # Open only local preview server
  stencila open document.md local

  # Open a specific remote URL
  stencila open document.md https://docs.google.com/document/d/abc123

  # Open only tracked remotes (skip local preview)
  stencila open document.md --no-local

  # Open only local preview (skip remotes)
  stencila open document.md --no-remotes


###### **Arguments:**

* `<PATH>` — The path to the document or parent folder

   Defaults to the current folder.

  Default value: `.`
* `<TARGET>` — The target to open

   Can be a full URL (e.g., https://docs.google.com/document/d/...), a service shorthand (e.g., "gdoc" or "m365"), or "local" to open only the local preview server. If omitted, opens all tracked remotes and the local preview server.

###### **Options:**

* `--sync <SYNC>` — Which direction(s) to sync the document

  Default value: `in`

  Possible values: `in`, `out`, `in-out`

* `--no-local` — Do not open the local preview server
* `--no-remotes` — Do not open tracked remote URLs



## `stencila publish`

Publish one or more documents

**Usage:** `stencila publish <COMMAND>`

###### **Subcommands:**

* `zenodo` — Publish to Zenodo
* `ghost` — Publish to Ghost
* `stencila` — Publish to Stencila Cloud



## `stencila publish zenodo`

Publish to Zenodo

**Usage:** `stencila publish zenodo [OPTIONS] [PATH]`


Further information

Authentication

To deposit a document at Zenodo, you must first have an authentication token that has the deposit:actions scope enabled.

To create an authentication token, log into Zenodo, visit your account's dashboard, then click Applications, followed by + New Token within the Personal access tokens  section. Give the token a name and enable the deposit:actions the scope. Enable the deposit:write scope to enable the --force flag.

To inform Stencila about the new token, add it as the STENCILA_ZENODO_TOKEN environment variable or include it as the --token <TOKEN> option.

Recommended workflow

We recommend starting with the Zenodo Sandbox at <https://sandbox.zenodo.org/>.

    $ export STENCILA_ZENODO_TOKEN="<TOKEN>" # from https://sandbox.zenodo.org/
    $ stencila publish zenodo <DOCUMENT_PATH>
    🎉 Draft deposition submitted
    🌐 URL: https://sandbox.zenodo.org/deposit/<deposit-id> (visit to check details and publish)
    📑 DOI: 10.5282/zenodo.<deposit-id>
    Note: This deposit has been submitted to the Zenodo Sandbox.
    Note: Use the --zenodo flag to resubmit to the production Zenodo server.

You should now review the deposit, make any corrections and then click publish from Zenodo's web interface when you're happy. If you wish to skip the review process and publish immediately, then use the --force flag.

Now that you have an understanding of the process, you should move to the Zenodo production server at <https://zenodo.org/>. This involves creating an authentication token there, informing Stencila about it and then adding the --zenodo flag as a command-line argument.

    $ export STENCILA_ZENODO_TOKEN="<TOKEN>" # from https://zenodo.org/
    $ stencila publish zenodo --zenodo <DOCUMENT_PATH>
    🎉 Draft deposition submitted
    🌐 URL: https://zenodo.org/deposit/<deposit-id> (visit to check details and publish)
    📑 DOI: 10.5281/zenodo.<deposit-id>

Metadata

Metadata for the deposition is provided by command-line arguments, falling back to metadata found within the document, then Stencila's defaults.

Zenodo requires that deposits have metadata such as title and description. It also requires that you describe which resource type and/or publication type the deposit is.

By default, Stencila describes your document as a publication, with the preprint sub-type. You can use the --lesson flag to describe your document as a lesson. To use another publication sub-type, review the list in the documentation above and provide it as the --publication=[<PUBLICATION_TYPE>] option.

Every source format has its own mechanism for providing metadata. For example, within Stencila Markdown (.smd files), you add YAML front matter:

  ---
  title: Example Stencila Markdown
  description: An example of a Stencila Markdown document with embedded metadata
  ---


###### **Arguments:**

* `<PATH>` — Path to location of what to publish

  Default value: `.`

###### **Options:**

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



## `stencila publish ghost`

Publish to Ghost

**Usage:** `stencila publish ghost [OPTIONS] <PATHS>...`

###### **Arguments:**

* `<PATHS>` — Paths to the files to publish

###### **Options:**

* `--domain <DOMAIN>` — The Ghost domain

   This is the domain name of your Ghost instance, with an optional port.

   Not required when pushing or pulling an existing post or page from Ghost but if supplied only document `identifiers` with this host will be used.
* `--key <KEY>` — The Ghost Admin API key

   To create one, create a new Custom Integration under the Integrations screen in Ghost Admin. Use the Admin API Key, rather than the Content API Key.

   You can also set the key as a secret so that it does not need to be entered here each time: `stencila secrets set GHOST_ADMIN_API_KEY`.
* `--post` — Create a post

   Does not apply when pushing to, or pulling from, and existing Ghost resource.

  Default value: `true`
* `--page` — Create a page

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



## `stencila publish stencila`

Publish to Stencila Cloud

**Usage:** `stencila publish stencila [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — Path to the file or directory to publish

   Defaults to the current directory.

  Default value: `.`

###### **Options:**

* `-k`, `--key <KEY>` — The key for the site
* `--dry-run` — Perform a dry run only
* `--no-html` — Do not publish a HTML file
* `--no-jsonld` — Do not publish a JSON-LD file
* `--no-llmd` — Do not publish a LLM-Markdown file
* `--no-bots` — Disallow all bots
* `--no-ai-bots` — Disallow AI bots



## `stencila demo`

Run a terminal demonstration from a document

**Usage:** `stencila demo [OPTIONS] <INPUT> [OUTPUT] [-- <AGG_ARGS>...]`

Examples
  # Demo a document in the terminal (uses natural preset by default)
  stencila demo document.md

  # Record a demo to an animated GIF
  stencila demo document.md demo.gif

  # Use fast preset for quick, smooth typing
  stencila demo document.md --preset fast

  # Use fast preset but add some typing variance
  stencila demo document.md --preset fast --speed-variance 0.2

  # Use fast preset but extend the maximum duration of running times
  stencila demo document.md --preset fast --min-running 2000 --max-running 4000

  # Use instant preset for immediate results
  stencila demo document.md --preset instant

  # Disable syntax highlighting for code blocks
  stencila demo document.md --no-highlighting

  # Demo only specific slides (slides are delimited by ***)
  stencila demo document.md --slides 2-4

  # Demo multiple slide ranges
  stencila demo document.md --slides "1,3-5,7-"


###### **Arguments:**

* `<INPUT>` — The path of the document to demo
* `<OUTPUT>` — The path of the recording to generate

   Supported output formats are GIF, MP4 and ASCIICast and will be determined from the file extension.
* `<AGG_ARGS>` — Arguments to pass through to `agg` when recoding to GIF

   See `agg --help`, or `stencila tools run agg --help`

###### **Options:**

* `--preset <PRESET>` — Preset for demo style

  Default value: `natural`

  Possible values:
  - `slow`:
    Slower typing with some typos and hesitation
  - `natural`:
    Average WPM, typo and hesitation rate
  - `fast`:
    200 WPM, no hesitation, no typos, consistent code running time
  - `instant`:
    Very high WPM and zero code running times

* `--speed <SPEED>` — Typing speed in words per minute

  Default value: `100`
* `--speed-variance <SPEED_VARIANCE>` — Variance in typing speed (0.0 to 1.0)

  Default value: `0.3`
* `--punctuation-pause <PUNCTUATION_PAUSE>` — How long to pause after punctuation (milliseconds)

  Default value: `200`
* `--typo-rate <TYPO_RATE>` — Probability of making a typo (0.0 to 1.0)

  Default value: `0`
* `--typo-pause <TYPO_PAUSE>` — How long to pause after typos before correcting (milliseconds)

  Default value: `500`
* `--hesitation-rate <HESITATION_RATE>` — Probability of brief hesitation (0.0 to 1.0)

  Default value: `0`
* `--hesitation-duration <HESITATION_DURATION>` — Hesitation duration in milliseconds

  Default value: `100`
* `--no-highlighting` — Do not apply syntax highlighting to code
* `--min-running <MIN_RUNNING>` — Minimum duration for running spinner in milliseconds

   The execution duration of executable nodes will be used for the spinner duration, but will be clamped to this minimum value.

  Default value: `500`
* `--max-running <MAX_RUNNING>` — Maximum duration for running spinner in milliseconds

   The execution duration of executable nodes will be used for the spinner duration, but will be clamped to this maximum value.

  Default value: `5000`
* `--slides <SLIDES>` — Specify which slides to demo

   Slides are delimited by thematic breaks (---). Examples: - "2" - only slide 2 - "2-4" - slides 2 through 4 - "2-" - slide 2 to the end - "-3" - slides 1 through 3 - "1,3-5,7-" - slides 1, 3 through 5, and 7 to the end
* `--ignore-errors` — Ignore any errors while executing document
* `--no-execute` — Do not execute the document before running the demo
* `--no-store` — Do not store the document after executing it
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



## `stencila db`

Manage the workspace and other document databases

**Usage:** `stencila db <COMMAND>`

Examples
  # Run pending migrations on workspace database
  stencila db migrate

  # Check migration status
  stencila db migrations status

  # Validate migrations without applying
  stencila db migrate --dry-run

  # Work with a specific database
  stencila db migrate /path/to/database.db


###### **Subcommands:**

* `new` — Create a new document database
* `add` — Add documents to the workspace database
* `remove` — Remove documents from the workspace database
* `query` — Query a workspace database
* `migrate` — Run pending database migrations
* `migrations` — Show applied and pending migrations



## `stencila db new`

Create a new document database

**Usage:** `stencila db new [OPTIONS] [PATH]`

Examples
  # Create a document database in the current workspace
  stencila db new

  # Create a document database at a specific path
  stencila db new path/to/my-database.kuzu

  # Overwrite the database if it already exists
  stencila db new temp.kuzu --force


###### **Arguments:**

* `<PATH>` — Path to the database file

   If not specified, creates a new workspace database.

###### **Options:**

* `-f`, `--force` — Overwrite the database if it already exists



## `stencila db add`

Add documents to the workspace database

**Usage:** `stencila db add [OPTIONS] <DOCUMENTS>... [-- <TOOL_ARGS>...]`

Examples
  # Add a single document to workspace database
  stencila db add document.md

  # Add multiple local Markdown documents
  stencila db add *.md docs/*.md

  # Add all local Markdown documents
  stencila db add **/*.md

  # Add a bioRxiv preprint using its DOI
  stencila db add https://doi.org/10.1101/2021.11.24.469827

  # Add specific pages from a PDF document
  stencila db add report.pdf --pages 1,3,5-10

  # Add PDF excluding cover and appendix pages
  stencila db add book.pdf --pages 2- --exclude-pages 50-

  # Add only even pages from a document
  stencila db add manuscript.pdf --pages even

Note
  This adds documents to the workspace database for
  indexing and querying. Files must be within the
  workspace directory to be added. Page selection
  options are available for multi-page formats like PDF.


###### **Arguments:**

* `<DOCUMENTS>` — The documents to add to the workspace database
* `<TOOL_ARGS>` — Arguments to pass through to the tool using for decoding

   Only supported for formats that use external tools for decoding and ignored otherwise.

###### **Options:**

* `-f`, `--from <FROM>` — The format of the input/s

   If not supplied, and inputting from a file, is inferred from the extension. See `stencila formats list` for available formats.
* `--fine` — Use fine decoding if available for input format

   Use this flag to decode content to the finest level of granularity supported by the format. This is the default for most formats.
* `--coarse` — Use coarse decoding if available for input format

   Use this flag to decode content to the coarsest level of granularity supported by the format. Useful for decoding formats that are not fully supported to avoid loss of structure.
* `--cache <CACHE>` — Reconstitute nodes from a cache

   Only useful when reconstituting a document from a file previously encoded with the `--reproducible` option and where a JSON cache of the document was encoded at the same times.

   Only supported for some formats (.e.g DOCX, ODT). At present, the cache must be the path to a JSON file.
* `--pages <PAGES>` — Pages to include when decoding multi-page documents

   Supports 1-based page selectors: single pages (N), ranges (N-M), open ranges (N- or -M), and keywords (odd, even). Multiple selectors can be combined with commas. Examples: --pages 1,3,5-7 or --pages 2-10,15-
* `--exclude-pages <EXCLUDE_PAGES>` — Pages to exclude when decoding multi-page documents

   Uses the same syntax as --pages but excludes the specified pages. Applied after --pages selection, allowing fine-grained control. Example: --pages 1-10 --exclude-pages 3,7 includes pages 1,2,4,5,6,8,9,10
* `--ignore-artifacts` — Ignore cached artifacts and force re-processing

   When decoding documents, Stencila caches intermediate artifacts (downloads, OCR results, etc.) in the nearest `.stencila` folder. Use this flag to ignore existing cached artifacts and re-download or re-process everything from scratch. Useful for getting updated data or retrying failed processing.
* `--no-artifacts` — Prevent creating artifacts during decoding

   By default, Stencila saves intermediate artifacts like downloads, OCR outputs, and extracted media to a `.stencila/artifacts` folder for reuse in future runs. Use this flag to disable artifacts entirely. Existing cached artifacts may still be used unless `--ignore-artifacts` is also specified.
* `--input-losses <INPUT_LOSSES>` — Action when there are losses decoding from input files

   Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or a filename to write the losses to (only `json` or `yaml` file extensions are supported).

  Default value: `debug`
* `--include-structuring <INCLUDE_OPS>` — Structuring operations to include (comma-separated)

   If not specified will default to those appropriate for the input format. Generally, less structuring is done for formats that are already well structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no operations. Example: heading-to-title,section-to-abstract

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--exclude-structuring <EXCLUDE_OPS>` — Structuring operations to exclude (comma-separated)

   Defaults to empty. Use this to prevent operations used by default for the input format. Use 'all' to exclude all operations, 'none' to exclude nothing. Example: remove-empty-text,remove-empty-paragraphs

  Possible values:
  - `none`:
    No structuring operations
  - `all`:
    All structuring operations
  - `sections-to-keywords`:
    Extract keywords from the "Keywords" section
  - `sections-to-abstract`:
    Extract abstract from the "Abstract" section
  - `sections-to-references`:
    Extract references from "References" section
  - `headings-to-title`:
    Extract document title from the first heading
  - `heading1-to-title`:
    Extract document title from the very first level 1 heading
  - `headings-decrement`:
    Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
  - `headings-primary-level1`:
    Ensure that all "primary" headings have level 1
  - `headings-to-sections`:
    Create a section for each heading
  - `headings-to-paragraphs`:
    Transform headings to paragraphs if appropriate
  - `paragraphs-to-keywords`:
    Extract keywords from paragraphs starting with "Keywords"
  - `paragraphs-to-headings`:
    Transform paragraphs to headings if appropriate
  - `paragraphs-to-sentences`:
    Split paragraphs into individual sentences
  - `figures-with-captions`:
    Combine an image with a figure caption before or after it
  - `tables-with-captions`:
    Combine a table caption with the following table or datatable
  - `table-images-to-rows`:
    Convert table images to table rows using OCR
  - `tables-to-datatables`:
    Transform tables into datatables if possible
  - `unwrap-media-objects`:
    Unwrap media objects from paragraphs to block level
  - `unwrap-quote-blocks`:
    Unwrap quote blocks containing more than two child blocks
  - `text-to-citations`:
    Convert text to structured citations
  - `text-to-links`:
    Convert URL text to structured links
  - `math-to-citations`:
    Convert math to structured citations
  - `math-images-to-tex`:
    Convert math images to TeX code using OCR
  - `links-to-citations`:
    Convert links to citations
  - `normalize-citations`:
    Normalize citation formatting and grouping
  - `remove-pre-primary`:
    Remove content before the first primary heading
  - `remove-frontmatter-duplicates`:
    Remove front matter that duplicates article metadata
  - `remove-empty-headings`:
    Remove empty headings
  - `remove-empty-tables`:
    Remove empty tables and datatables
  - `remove-empty-lists`:
    Remove empty lists
  - `remove-empty-paragraphs`:
    Remove empty paragraphs
  - `remove-empty-text`:
    Remove empty text

* `--citation-style <CITATION_STYLE>` — The citation style to assume for text-to-citation structuring.

   If not specified, will be determined automatically based on whether references are numbered and the relative frequency of detected styles within text. Only relevant if the `text-to-citations` operation is enabled.

  Possible values:
  - `author-year`:
    Author-year citations like (Smith, 2023)
  - `bracketed-numeric`:
    Bracketed numeric citations like [1]
  - `parenthetic-numeric`:
    Parenthetic numeric citations like (1)
  - `superscripted-numeric`:
    Superscripted numeric citations like ¹

* `--tool <TOOL>` — The tool to use for decoding inputs

   Only supported for formats that use alternative external tools for decoding and ignored otherwise.
* `--no-canonicalize` — Do not canonicalize the document



## `stencila db remove`

Remove documents from the workspace database

**Usage:** `stencila db remove <DOCUMENTS>...`

Examples
  # Remove a document from workspace database
  stencila db remove document.md

  # Remove multiple documents
  stencila db remove *.md docs/*.md

  # Use the rm alias
  stencila db rm old-document.md

Note
  This removes documents from the workspace database
  but does not delete the actual files. The files
  will no longer be indexed or queryable.


###### **Arguments:**

* `<DOCUMENTS>` — The document to remove from the workspace database



## `stencila db query`

Query a workspace database

**Usage:** `stencila db query [OPTIONS] <QUERY> [OUTPUT]`

Examples
  # Query the workspace database
  stencila db query "workspace.paragraphs()"

  # Use Cypher query language
  stencila db query --cypher "MATCH (h:Heading) WHERE h.level = 1 RETURN h"


###### **Arguments:**

* `<QUERY>` — The DocsQL or Cypher query to run

   If the query begins with the word `MATCH` it will be assumed to be cypher. Use the `--cypher` flag to force this.
* `<OUTPUT>` — The path of the file to output the result to

   If not supplied the output content is written to `stdout`.

###### **Options:**

* `-c`, `--cypher` — Use Cypher as the query language (instead of DocsQL the default)
* `-t`, `--to <TO>` — The format to output the result as

   Defaults to inferring the format from the file name extension of the `output`. If no `output` is supplied, defaults to JSON. See `stencila codecs list` for available formats.
* `--compact` — Use compact form of encoding if possible

   Use this flag to produce the compact forms of encoding (e.g. no indentation) which are supported by some formats (e.g. JSON, HTML).
* `-p`, `--pretty` — Use a "pretty" form of encoding if possible

   Use this flag to produce pretty forms of encoding (e.g. indentation) which are supported by some formats (e.g. JSON, HTML).



## `stencila db migrate`

Run pending database migrations

**Usage:** `stencila db migrate [OPTIONS] [DB]`

Examples
  # Apply pending migrations to workspace database
  stencila db migrate

  # Preview what migrations would be applied
  stencila db migrate --dry-run

  # Apply migrations to a specific database
  stencila db migrate path/to/my-database.kuzu


###### **Arguments:**

* `<DB>` — Path to the database file

   If not specified, uses the workspace database

###### **Options:**

* `-d`, `--dry-run` — Preview migrations without applying them



## `stencila db migrations`

Show applied and pending migrations

**Usage:** `stencila db migrations [OPTIONS] [DB]`

Examples
  # Show applied and pending migrations for the workspace database
  stencila db migrations

  # Output migrations as JSON
  stencila db migrations --as json

  # Show migrations for a specific database
  stencila db migrations path/to/database.kuzu


###### **Arguments:**

* `<DB>` — Path to the database file

###### **Options:**

* `-a`, `--as <AS>` — Output format

  Possible values: `json`, `yaml`




## `stencila prompts`

Manage prompts

**Usage:** `stencila prompts [COMMAND]`

Examples
  # List all available prompts
  stencila prompts

  # Show details about a specific prompt
  stencila prompts show edit-text

  # Infer which prompt would be used for a query
  stencila prompts infer --instruction-type create "Make a table"

  # Update builtin prompts from remote
  stencila prompts update

  # Reset prompts to embedded defaults
  stencila prompts reset


###### **Subcommands:**

* `list` — List the prompts available
* `show` — Show a prompt
* `infer` — Infer a prompt from a query
* `update` — Update builtin prompts
* `reset` — Reset builtin prompts



## `stencila prompts list`

List the prompts available

Shows all available prompts with their names, descriptions, and versions.

**Usage:** `stencila prompts list [OPTIONS]`

Examples
  # List all prompts in table format
  stencila prompts list

  # Output prompts as JSON
  stencila prompts list --as json


###### **Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila prompts show`

Show a prompt

Displays the full content and metadata of a specific prompt in the requested format.

**Usage:** `stencila prompts show [OPTIONS] <NAME>`

Examples
  # Show a prompt as Markdown
  stencila prompts show edit-text

  # Show a prompt as JSON
  stencila prompts show create-table --to json


###### **Arguments:**

* `<NAME>` — The name of the prompt to show

###### **Options:**

* `-t`, `--to <TO>` — The format to show the prompt in

  Default value: `md`



## `stencila prompts infer`

Infer a prompt from a query

Useful for checking which prompt will be matched to a given instruction type, node types, and/or query

**Usage:** `stencila prompts infer [OPTIONS] [QUERY]`

Examples
  # Infer prompt with a specific query
  stencila prompts infer "Update this paragraph based on latest data"

  # Infer for a specific instruction type
  stencila prompts infer --instruction-type create "list of top regions"


###### **Arguments:**

* `<QUERY>` — The query

###### **Options:**

* `-i`, `--instruction-type <INSTRUCTION_TYPE>` — The instruction type
* `-n`, `--node-types <NODE_TYPES>` — The node types



## `stencila prompts update`

Update builtin prompts

Downloads the latest versions of builtin prompts from the Stencila prompts repository. This adds new prompts and updates existing ones while preserving any custom modifications you may have made.

**Usage:** `stencila prompts update`

Examples
  # Update builtin prompts from https://github.com/stencila/stencila
  stencila prompts update




## `stencila prompts reset`

Reset builtin prompts

Re-initializes the builtin prompts directory to those prompts embedded in this version of Stencila

**Usage:** `stencila prompts reset`

Examples
  # Reset prompts to embedded defaults
  stencila prompts reset

Warning
  This will overwrite any custom modifications you have
  made to builtin prompts, restoring them to the versions
  embedded in this Stencila release.




## `stencila models`

Manage and interact with generative AI models

**Usage:** `stencila models [COMMAND]`

Examples
  # List all available models
  stencila models

  # List models as JSON
  stencila models list --as json

  # Test a model with a prompt
  stencila models run "Explain photosynthesis"

  # Test a specific model
  stencila models run "Write a poem" --model gpt-4o

  # Run with multiple text arguments
  stencila models run "Analyze this:" "Some data here"

  # Mix text and file arguments
  stencila models run "Summarize this file:" document.txt

  # Multiple files and text
  stencila models run "Compare these files:" file1.txt file2.txt

  # Dry run to see task construction
  stencila models run "Hello" --dry-run

Model Types
  • builtin - Built into Stencila
  • local - Running locally (e.g. Ollama)
  • remote - Cloud-based APIs
  • router - Routes to other models
  • proxied - Proxied through another service


###### **Subcommands:**

* `list` — List available models with their status and capabilities
* `run` — Execute a task using a generative AI model



## `stencila models list`

List available models with their status and capabilities

**Usage:** `stencila models list [OPTIONS] [PREFIX]`

Examples
  # List all models in table format
  stencila models list

  # Filter models by ID prefix
  stencila models list google/gemini

  # Output models as YAML
  stencila models list --as yaml


###### **Arguments:**

* `<PREFIX>` — Filter models by ID prefix (e.g., "ollama/gemma")

###### **Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila models run`

Execute a task using a generative AI model

Primarily intended for testing model selection and routing. This command constructs a task from the provided inputs, selects an appropriate model, and displays both the constructed task and the generated output.

**Usage:** `stencila models run [OPTIONS] [ARGS]...`

Examples
  # Run with automatic model selection
  stencila models run "Explain quantum computing"

  # Run with a specific model
  stencila models run "Write a haiku" --model gpt-3.5-turbo

  # Multiple text arguments
  stencila models run "Analyze this data:" "temperature: 23°C, humidity: 65%"

  # Mix text and file paths (files detected automatically)
  stencila models run "Summarize:" report.txt

  # Multiple files and text
  stencila models run "Compare these:" version1.py version2.py

  # Run a dry run to see task construction
  stencila models run "Hello world" --dry-run

  # Use the execute alias
  stencila models execute "Summarize this text"

Note
  Arguments are automatically detected as file paths (if they exist) or treated as
  text content. Images are detected by file extension. This command is primarily
  for testing model routing and selection.


###### **Arguments:**

* `<ARGS>` — Text prompts and/or file paths (automatically detected)

###### **Options:**

* `-m`, `--model <MODEL>` — Model id or pattern to select a specific model (e.g., "gpt-4o", "ollama/")
* `-f`, `--format <FORMAT>` — Output format for generated content (json, markdown, yaml, etc.)
* `-s`, `--schema <SCHEMA>` — JSON schema name for structured output validation (e.g., "math-block-tex")
* `--system <SYSTEM>` — System message to set context or behavior for the model
* `-o`, `--output <OUTPUT>` — Write generated output to the specified file instead of stdout
* `--dry-run` — Show task construction and model selection without executing



## `stencila kernels`

Manage execution kernels

**Usage:** `stencila kernels [COMMAND]`

Examples
  # List all available kernels
  stencila kernels

  # Get information about a specific kernel
  stencila kernels info python

  # List packages available to a kernel
  stencila kernels packages r

  # Execute code in a kernel
  stencila kernels execute python "print('Hello')"


###### **Subcommands:**

* `list` — List the kernels available
* `info` — Get information about a kernel
* `packages` — List packages available to a kernel
* `execute` — Execute code in a kernel
* `evaluate` — Evaluate a code expression in a kernel



## `stencila kernels list`

List the kernels available

**Usage:** `stencila kernels list [OPTIONS]`

Examples
  # List all available kernels
  stencila kernels list

  # List only math kernels
  stencila kernels list --type math

  # Output kernel list as YAML
  stencila kernels list --as yaml


###### **Options:**

* `-t`, `--type <TYPE>` — Only list kernels of a particular type

  Possible values: `programming`, `database`, `templating`, `diagrams`, `visualization`, `math`, `styling`

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila kernels info`

Get information about a kernel

Mainly used to check the version of the kernel runtime and operating system for debugging purpose.

**Usage:** `stencila kernels info <NAME>`

Examples
  # Get information about the Python kernel
  stencila kernels info python

  # Get information about the R kernel
  stencila kernels info r

  # Get information about the JavaScript kernel
  stencila kernels info javascript


###### **Arguments:**

* `<NAME>` — The name of the kernel to get information for



## `stencila kernels packages`

List packages available to a kernel

Mainly used to check libraries available to a kernel for debugging purpose.

**Usage:** `stencila kernels packages <NAME> [FILTER]`

Examples
  # List all packages available to Python kernel
  stencila kernels packages python

  # Filter packages by name (case insensitive)
  stencila kernels packages python numpy

  # List R packages containing 'plot'
  stencila kernels packages r plot


###### **Arguments:**

* `<NAME>` — The name of the kernel to list packages for
* `<FILTER>` — A filter on the name of the kernel

   Only packages whose name contains this string will be included (case insensitive)



## `stencila kernels execute`

Execute code in a kernel

Creates a temporary kernel instance, executes one or more lines of code, and returns any outputs and execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels execute [OPTIONS] <NAME> <CODE>`

Examples
  # Execute Python code
  stencila kernels execute python "print('Hello World')"

  # Execute multi-line code with escaped newlines
  stencila kernels execute python "x = 5\nprint(x * 2)"

  # Execute code in a sandboxed environment
  stencila kernels execute python "import os\nprint(os.environ)" --box

  # Use the exec alias
  stencila kernels exec r "print(mean(c(1,2,3,4,5)))"


###### **Arguments:**

* `<NAME>` — The name of the kernel to execute code in
* `<CODE>` — The code to execute

   Escaped newline characters (i.e. "\n") in the code will be transformed into new lines before passing to the kernel.

###### **Options:**

* `-b`, `--box` — Execute code in a kernel instance with `Box` execution bounds



## `stencila kernels evaluate`

Evaluate a code expression in a kernel

Creates a temporary kernel instance, evaluates the expression in it, and returns the output and any execution messages.

Mainly intended for quick testing of kernels during development.

**Usage:** `stencila kernels evaluate <NAME> <CODE>`

Examples
  # Evaluate a Python expression
  stencila kernels evaluate python "2 + 2"

  # Evaluate an R expression
  stencila kernels evaluate r "sqrt(16)"

  # Evaluate a JavaScript expression
  stencila kernels evaluate javascript "Math.PI * 2"

  # Use the eval alias
  stencila kernels eval python "sum([1, 2, 3, 4, 5])"


###### **Arguments:**

* `<NAME>` — The name of the kernel to evaluate code in
* `<CODE>` — The code expression to evaluate



## `stencila linters`

Manage linters

**Usage:** `stencila linters [COMMAND]`

Examples
  # List all available linters
  stencila linters

  # Lint a file using a linter
  stencila linters lint script.py


###### **Subcommands:**

* `list` — List the linters available
* `lint` — Lint a file



## `stencila linters list`

List the linters available

**Usage:** `stencila linters list [OPTIONS]`

Examples
  # List all available linters
  stencila linters list

  # List only Python linters
  stencila linters list --lang py

  # List only citation linters
  stencila linters list --node-type Citation

  # Output linter list as YAML
  stencila linters list --as yaml


###### **Options:**

* `-l`, `--language <LANGUAGE>` — Only list linter that support a specific language/format
* `-n`, `--node-type <NODE_TYPE>` — Only list linter that support a specific node type
* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila linters lint`

Lint a file

Mainly intended for testing linters during development of Stencila. To lint a document use `stencila lint`.

**Usage:** `stencila linters lint [OPTIONS] <FILE>`

Examples
  # Lint a Python file
  stencila linters lint script.py

  # Lint and format a JavaScript file
  stencila linters lint app.js --format

  # Lint and fix issues where possible
  stencila linters lint code.r --fix

  # Lint with both formatting and fixing
  stencila linters lint code.py --format --fix


###### **Arguments:**

* `<FILE>` — The file to lint

###### **Options:**

* `-l`, `--linter <LINTER>` — The name of the linter to use
* `--format` — Format the content of the file
* `--fix` — Fix warnings and errors in the file where possible



## `stencila formats`

List and inspect supported formats

**Usage:** `stencila formats [COMMAND]`


  # List all supported formats
  stencila formats list

  # Output formats as JSON
  stencila formats list --as json

Format Support
  • From: Whether the format can be read/imported
  • To: Whether the format can be written/exported
  • Lossless: Whether conversion preserves all data


###### **Subcommands:**

* `list` — List the support for formats
* `structuring` — Get a list of all structuring operations, or those that are the default for a format



## `stencila formats list`

List the support for formats

**Usage:** `stencila formats list [OPTIONS]`


  # List all supported formats in table format
  stencila formats list

  # Export format information as JSON
  stencila formats list --as json

Columns
  • Name: The format name
  • Extension: Default file extension
  • From: Can read/import this format
  • To: Can write/export this format
  • Lossless: Whether conversion preserves all data


###### **Options:**

* `-a`, `--as <AS>` — Output the list as JSON or YAML

  Possible values: `json`, `yaml`




## `stencila formats structuring`

Get a list of all structuring operations, or those that are the default for a format

**Usage:** `stencila formats structuring [OPTIONS] [FORMAT]`


  # List all structuring operations
  stencila formats structuring

  # List the default structuring operations for DOCX
  stencila formats structuring docx

  # List all structuring operations with details for each
  stencila formats structuring --verbose


###### **Arguments:**

* `<FORMAT>` — The format to show default structuring operations for

###### **Options:**

* `-v`, `--verbose` — Provide longer details on each structuring operation



## `stencila themes`

Manage themes

**Usage:** `stencila themes [COMMAND]`

Examples
  # List all available themes
  stencila themes

  # Show the default resolved theme
  stencila themes show

  # Show a specific theme
  stencila themes show tufte

  # Create a new workspace theme
  stencila themes new

  # Create a named user theme
  stencila themes new my-theme

  # Remove a user theme
  stencila themes remove my-theme


###### **Subcommands:**

* `list` — List the available themes
* `show` — Show the resolved theme CSS
* `new` — Create a new theme
* `remove` — Remove a user theme



## `stencila themes list`

List the available themes

**Usage:** `stencila themes list`

Examples
  # List all available themes
  stencila themes list




## `stencila themes show`

Show the resolved theme CSS

**Usage:** `stencila themes show [OPTIONS] [NAME]`

Examples
  # Show the default resolved theme
  stencila themes show

  # Show a specific theme by name
  stencila themes show tufte

  # Show a user theme
  stencila themes show my-theme

  # Show theme with resolved CSS variables
  stencila themes show stencila --verbose


###### **Arguments:**

* `<NAME>` — The name of the theme to show

   If not provided, shows the default resolved theme following the resolution order: workspace theme.css → user default.css → builtin stencila.css

###### **Options:**

* `-v`, `--verbose` — Show resolved CSS variables



## `stencila themes new`

Create a new theme

**Usage:** `stencila themes new [OPTIONS] [NAME]`

Examples
  # Create a new workspace theme in the current folder
  stencila themes new

  # Create a named user theme in the config folder
  stencila themes new my-theme

  # Force overwrite an existing user theme
  stencila themes new my-theme --force


###### **Arguments:**

* `<NAME>` — The name of the theme to create

   If not provided, creates `theme.css` in the current directory. Otherwise, creates in the themes config directory.

###### **Options:**

* `-f`, `--force` — Overwrite the theme file if it already exists



## `stencila themes remove`

Remove a user theme

**Usage:** `stencila themes remove [OPTIONS] <NAME>`

Examples
  # Remove a user theme
  stencila themes remove my-theme

  # Force remove without confirmation
  stencila themes remove my-theme --force

  # Use the rm alias
  stencila themes rm my-theme


###### **Arguments:**

* `<NAME>` — The name of the theme to remove

###### **Options:**

* `-f`, `--force` — Remove the theme without confirmation



## `stencila secrets`

Manage secrets

**Usage:** `stencila secrets [COMMAND]`

Examples
  # List all configured secrets
  stencila secrets

  # Set a secret interactively (prompts for value)
  stencila secrets set STENCILA_API_TOKEN

  # Set a secret from stdin (pipe the value)
  echo "sk-abc123..." | stencila secrets set OPENAI_API_KEY

  # Delete a secret
  stencila secrets delete ANTHROPIC_API_KEY

  # Use the add/remove aliases instead
  stencila secrets add STENCILA_API_TOKEN
  stencila secrets remove STENCILA_API_TOKEN

Security
  Secrets are stored securely using your system's keyring.
  They are used to authenticate with external services like
  AI model providers and cloud platforms.


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

Examples
  # Set a secret interactively (you'll be prompted)
  stencila secrets set OPENAI_API_KEY

  # Set a secret from stdin
  echo "sk-abc123..." | stencila secrets set OPENAI_API_KEY

  # Set API tokens for different services
  stencila secrets set ANTHROPIC_API_KEY
  stencila secrets set GOOGLE_AI_API_KEY
  stencila secrets set STENCILA_API_TOKEN

  # Use the add alias instead
  stencila secrets add STENCILA_API_TOKEN

Security
  When setting secrets interactively, your input will be
  hidden. When piping from stdin, ensure your shell history
  doesn't record the command with the secret value.


###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila secrets delete`

Delete a secret previously set using Stencila

**Usage:** `stencila secrets delete <NAME>`

Examples
  # Delete a specific secret
  stencila secrets delete OPENAI_API_KEY

  # Delete API tokens
  stencila secrets delete ANTHROPIC_API_KEY
  stencila secrets delete GOOGLE_AI_API_KEY

  # Use the remove alias instead
  stencila secrets remove GOOGLE_AI_API_KEY

Warning
  This permanently removes the secret from your system's
  keyring. You'll need to set it again if you want to use
  it in the future.


###### **Arguments:**

* `<NAME>` — The name of the secret



## `stencila tools`

Manage tools and environments used by Stencila

Provides a unified interface for managing various tools including programming languages, package managers, linters, and converters. It automatically detects and integrates with environment managers like devbox, mise, and uv to provide isolated and reproducible environments.

**Usage:** `stencila tools [COMMAND]`

Examples
  # List all available tools
  stencila tools

  # Show details about a specific tool
  stencila tools show python

  # Install a tool
  stencila tools install mise

  # Install multiple tools
  stencila tools install mise uv ruff

  # Install all dependencies from config files
  stencila tools install

  # Detect environment configuration in current directory
  stencila tools env

  # Run a command with automatic environment detection
  stencila tools run -- python script.py


###### **Subcommands:**

* `list` — List available tools and their installation status
* `show` — Show information about a specific tool
* `install` — Install a tool or setup development environment
* `env` — Detect environment manager configuration for a directory
* `run` — Run a command with automatic environment detection and setup



## `stencila tools list`

List available tools and their installation status

Displays a table of all tools that Stencila can manage, including their type, required version, available version, and installation path. The versions and paths shown reflect the currently active environment managers (devbox, mise, etc.) if configured in the current directory, otherwise system-wide installations.

**Usage:** `stencila tools list [OPTIONS]`

Examples
  # List all tools
  stencila tools list

  # List only installed tools
  stencila tools list --installed

  # List only installable tools
  stencila tools list --installable

  # List only execution tools (programming languages)
  stencila tools list --type execution

  # Export tool list as Model Context Protocol tool specifications
  stencila tools list --as json

  # Display tool list as YAML
  stencila tools list --as yaml


###### **Options:**

* `-t`, `--type <TYPE>` — Only list tools of a particular type

  Possible values: `collaboration`, `conversion`, `environments`, `execution`, `linting`, `packages`

* `--installed` — Only list tools that are installed

   This filters out tools that are not installed or cannot be found in PATH
* `--installable` — Only list tools that can be installed automatically

   This filters to only show tools that have installation scripts available
* `-a`, `--as <FORMAT>` — Output format for tool specifications

   Export tools as Model Context Protocol (MCP) tool specifications. This is useful for integrating with AI assistants and other MCP-compatible systems. See https://modelcontextprotocol.io/docs/concepts/tools for more details.

  Possible values: `json`, `yaml`




## `stencila tools show`

Show information about a specific tool

Displays information about a tool including its name, URL, description, version requirements, installation status, and file path. The version and path shown reflect the currently active environment managers (devbox, mise, etc.) if configured in the current directory, otherwise system-wide installation.

**Usage:** `stencila tools show <TOOL>`

Examples
  # Show details about Pandoc
  stencila tools show pandoc

  # Show details about uv
  stencila tools show uv

Supported tools
  # See which tools are installed
  stencila tools list --installed


###### **Arguments:**

* `<TOOL>` — The name of the tool to show details for



## `stencila tools install`

Install a tool or setup development environment

When provided with one or more tool names as arguments, installs those tools. When run without arguments, automatically detects and installs environment managers, tools, and dependencies based on configuration files found in the project directory.

**Usage:** `stencila tools install [OPTIONS] [TOOL]...`

Tool Installation Examples
  # Install mise (tool version manager)
  stencila tools install mise

  # Install uv (Python package manager)
  stencila tools install uv

  # Install multiple tools at once
  stencila tools install mise uv ruff

  # Force reinstall an already installed tool
  stencila tools install --force ruff

Environment Setup Examples
  # Install all dependencies from config files in current directory
  stencila tools install

  # Install dependencies from config files in specific directory
  stencila tools install -C /path/to/project

  # Show what would be installed without actually installing
  stencila tools install --dry-run

  # Skip Python dependencies during setup
  stencila tools install --skip-python

Setup phases (when no tool specified)
  1. Install environment managers (mise, devbox, etc.) if needed
  2. Install tools from environment manager configs
  3. Setup Python dependencies (pyproject.toml, requirements.txt)
  4. Setup R dependencies (renv.lock, DESCRIPTION)

Supported tools
  # See which tools can be installed
  stencila tools list --installable


###### **Arguments:**

* `<TOOL>` — The name(s) of the tool(s) to install (if not provided, installs all dependencies from config files)

###### **Options:**

* `-C`, `--path <DIR>` — The directory to setup when installing from config files (defaults to current directory)
* `--skip-env` — Skip environment manager tool installation (only when installing from configs)
* `--skip-python` — Skip Python dependency installation (only when installing from configs)
* `--skip-r` — Skip R dependency installation (only when installing from configs)
* `-f`, `--force` — Force installation even if the tool is already installed
* `--dry-run` — Show which tools would be installed without actually installing them



## `stencila tools env`

Detect environment manager configuration for a directory

Searches the specified directory (and parent directories) for configuration files that indicate the presence of environment or package managers. This helps understand what development environment is configured for a project.

Displays both the manager information and the content of the configuration files for inspection.

**Usage:** `stencila tools env [PATH]`

Examples
  # Check current directory for environment configuration
  stencila tools env

  # Check a specific project directory
  stencila tools env /path/to/project

  # Check parent directory
  stencila tools env ..


###### **Arguments:**

* `<PATH>` — The directory to check for environment manager configuration

   Searches this directory and all parent directories for configuration files. Configuration files include devbox.json, mise.toml, pixi.toml, and pyproject.toml.

  Default value: `.`



## `stencila tools run`

Run a command with automatic environment detection and setup

Mainly for testing configurations. Executes a command within the appropriate development environment by automatically detecting and configuring environment managers. This ensures commands run with the correct tool versions and dependencies as specified in the project configuration.

The command automatically detects and chains environment managers: (1) Environment managers (e.g devbox, mise, pixi) - for tool version management (2) Package managers (e.g uv) - for language-specific dependencies

**Usage:** `stencila tools run [OPTIONS] [COMMAND]...`

Note
  Use '--' to separate the run command options from the command to execute.
  This prevents argument parsing conflicts
  
Examples
  # Run Python script with automatic environment detection
  stencila tools run -- python script.py

  # Run Python code
  stencila tools run -- python -c "print('hello')"

  # Run from a different directory
  stencila tools run -C /path/to/project -- npm test

  # Run a complex command with multiple arguments
  stencila tools run -- pandoc input.md -o output.pdf --pdf-engine=xelatex


###### **Arguments:**

* `<COMMAND>` — The command and arguments to run (specify after --)

   All arguments after '--' are passed directly to the command. This allows commands with arguments that start with hyphens.

###### **Options:**

* `-C`, `--cwd <DIR>` — Working directory for the command

   Environment detection will be performed relative to this directory. If not specified, uses the current working directory.



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

* `--cors <CORS>` — CORS policy level

   Controls Cross-Origin Resource Sharing (CORS) headers. Ordered from most to least restrictive: - `none`: No CORS headers (default) - `restrictive`: Allow GET and POST requests from localhost - `local`: Allow any requests from localhost and 127.0.0.1 origins - `permissive`: Allow all origins, methods, and headers

  Default value: `none`

  Possible values:
  - `none`:
    No CORS headers
  - `restrictive`:
    Allow only same-origin requests
  - `local`:
    Allow localhost and 127.0.0.1 origins only
  - `permissive`:
    Allow all origins, methods, and headers




## `stencila snap`

Capture screenshots and measurements of documents served by Stencila

The `snap` command allows programmatic screenshotting and measurement of documents served by Stencila. It can be used to:

- Iterate on themes and styled elements and verify changes - Capture screenshots for documentation or CI - Assert computed CSS properties and layout metrics - Measure page elements for automated testing

**Usage:** `stencila snap [OPTIONS] [PATH] [OUTPUT]`

Examples
  # Start server in background
  stencila serve --sync in &

  # Capture viewport screenshot (default)
  stencila snap snaps/viewport.png

  # Capture full scrollable page
  stencila snap --full snaps/full.png

  # Verify computed padding for title
  stencila snap --assert "css([slot=title]).paddingTop>=24px"

  # Capture mobile viewport of specific element
  stencila snap --device mobile --selector "stencila-article [slot=title]" snaps/mobile.png

  # Capture full mobile page
  stencila snap --device mobile --full snaps/mobile-full.png

  # Force light or dark mode
  stencila snap --light snaps/light.png
  stencila snap --dark snaps/dark.png

  # Preview with PDF/print styles (A4 width)
  stencila snap --print snaps/print-preview.png

  # Multiple assertions without screenshot
  stencila snap \
    --assert "css([slot=title]).fontSize>=28px" \
    --assert "count(section)==5" \
    --measure

  # Use custom viewport and wait conditions
  stencila snap \
    --width 1920 --height 1080 \
    --wait-until networkidle \
    --delay 500 \
    snaps/desktop.png

  # Capture specific document path
  stencila snap docs/guide.md snaps/guide.png


###### **Arguments:**

* `<PATH>` — Path to document or directory

   If not specified, will use the current directory. The path should be within the directory being served by a running Stencila server.
* `<OUTPUT>` — Output screenshot path (.png)

   If specified, a screenshot will be captured. If not specified, only measurements and assertions will be performed (no screenshot).

###### **Options:**

* `--selector <SELECTOR>` — CSS selector to capture or measure

   If specified, screenshots will be cropped to this element and measurements will focus on it.
* `--full` — Capture full scrollable page

   By default, captures only the viewport (first screen). Use this flag to capture the entire scrollable document.
* `--device <DEVICE>` — Device preset

   Use a predefined viewport configuration: laptop, desktop, iphone-15, ipad, ipad-landscape

  Possible values:
  - `laptop`:
    Laptop (1440x900 @2x DPR)
  - `desktop`:
    Desktop (1920x1080 @1x DPR)
  - `mobile`:
    Mobile (390x844 @3x DPR)
  - `tablet`:
    Tablet (768x1024 @2x DPR)
  - `tablet-landscape`:
    Tablet Landscape (1024x768 @2x DPR)

* `--width <WIDTH>` — Viewport width in pixels

   Overrides device preset width if both are specified
* `--height <HEIGHT>` — Viewport height in pixels

   Overrides device preset height if both are specified
* `--dpr <DPR>` — Device pixel ratio

   Overrides device preset DPR if both are specified
* `--light` — Use light color scheme
* `--dark` — Use dark color scheme
* `--print` — Preview with print media styles (A4 width, for PDF preview)

   Sets viewport to A4 dimensions (794x1123px), emulates print media type, and applies simulated page margins (75px, based on @page margins from web/src/themes/base/pages.css). This provides a preview of PDF output but is not identical - theme-defined @page margin boxes, custom page sizes, and other advanced @page features will not be rendered. Conflicts with --light, --dark, and --device options.
* `--wait-until <WAIT_UNTIL>` — When to capture: load, domcontentloaded, networkidle

  Default value: `network-idle`

  Possible values:
  - `load`:
    Wait for 'load' event
  - `dom-content-loaded`:
    Wait for 'DOMContentLoaded' event
  - `network-idle`:
    Wait for network idle (default)

* `--wait-for <WAIT_FOR>` — Wait for CSS selector to exist before capturing
* `--delay <DELAY>` — Additional delay in milliseconds after page is ready
* `--measure` — Collect computed CSS and layout metrics
* `--assert <ASSERTIONS>` — Assert measurement conditions

   Can be specified multiple times. Each assertion is a condition like: - css(.title).paddingTop >= 24px - count(section) == 5 - box(.header).height < 100
* `--url <URL>` — Override URL (instead of discovering server)

   Useful when connecting to a specific server or non-standard configuration



## `stencila lsp`

Run the Language Server Protocol server

**Usage:** `stencila lsp`



## `stencila cloud`

Manage Stencila Cloud account

**Usage:** `stencila cloud [COMMAND]`

Examples
  // TODO: complete as for other module's CLI_AFTER_LONG_HELP


###### **Subcommands:**

* `status` — Display Stencila Cloud authentication status
* `signin` — Sign in to Stencila Cloud
* `signout` — Sign out from Stencila Cloud
* `logs` — Display logs from Stencila Cloud workspace sessions



## `stencila cloud status`

Display Stencila Cloud authentication status

**Usage:** `stencila cloud status`

Examples
  # See your current authentication status
  stencila cloud status




## `stencila cloud signin`

Sign in to Stencila Cloud

**Usage:** `stencila cloud signin [OPTIONS]`

Examples
  # Sign in to Stencila Cloud
  stencila cloud signin

  # Sign in manually
  stencila cloud signin --manual

  # Use one of the command aliases
  stencila signin
  stencila login


###### **Options:**

* `-m`, `--manual` — Signin by manually entering a Stencila access token



## `stencila cloud signout`

Sign out from Stencila Cloud

**Usage:** `stencila cloud signout`

Examples
  # Sign out from Stencila Cloud
  stencila cloud signout

  # Use one of the command aliases
  stencila signout
  stencila logout




## `stencila cloud logs`

Display logs from Stencila Cloud workspace sessions

**Usage:** `stencila cloud logs [OPTIONS] --session <SESSION>`

Examples
  # View logs for a session
  stencila cloud logs --session SESSION_ID

  # View last 50 logs
  stencila cloud logs --session SESSION_ID --limit 50

  # Follow logs (poll every 5 seconds by default)
  stencila cloud logs --session SESSION_ID --follow

  # Follow logs with custom polling interval
  stencila cloud logs --session SESSION_ID --follow 10

  # Filter logs by level
  stencila cloud logs --session SESSION_ID --level error


###### **Options:**

* `-s`, `--session <SESSION>` — The session ID to retrieve logs for
* `-l`, `--limit <LIMIT>` — Maximum number of recent logs to display
* `-f`, `--follow <FOLLOW>` — Continuously poll for new logs every N seconds (press Ctrl+C to stop)

   If provided without a value, defaults to 5 seconds. Minimum value is 1 second.
* `--level <LEVEL>` — Filter logs by level (error, warn, info, debug, trace)



## `stencila signin`

Sign in to Stencila Cloud

**Usage:** `stencila signin [OPTIONS]`

Examples
  # Sign in to Stencila Cloud
  stencila cloud signin

  # Sign in manually
  stencila cloud signin --manual

  # Use one of the command aliases
  stencila signin
  stencila login


###### **Options:**

* `-m`, `--manual` — Signin by manually entering a Stencila access token



## `stencila signout`

Sign out from Stencila Cloud

**Usage:** `stencila signout`

Examples
  # Sign out from Stencila Cloud
  stencila cloud signout

  # Use one of the command aliases
  stencila signout
  stencila logout




## `stencila logs`

Display logs from Stencila Cloud workspace sessions

**Usage:** `stencila logs [OPTIONS] --session <SESSION>`

Examples
  # View logs for a session
  stencila cloud logs --session SESSION_ID

  # View last 50 logs
  stencila cloud logs --session SESSION_ID --limit 50

  # Follow logs (poll every 5 seconds by default)
  stencila cloud logs --session SESSION_ID --follow

  # Follow logs with custom polling interval
  stencila cloud logs --session SESSION_ID --follow 10

  # Filter logs by level
  stencila cloud logs --session SESSION_ID --level error


###### **Options:**

* `-s`, `--session <SESSION>` — The session ID to retrieve logs for
* `-l`, `--limit <LIMIT>` — Maximum number of recent logs to display
* `-f`, `--follow <FOLLOW>` — Continuously poll for new logs every N seconds (press Ctrl+C to stop)

   If provided without a value, defaults to 5 seconds. Minimum value is 1 second.
* `--level <LEVEL>` — Filter logs by level (error, warn, info, debug, trace)



## `stencila upgrade`

Upgrade to the latest version

**Usage:** `stencila upgrade [OPTIONS]`

Examples
  # Upgrade to the latest version
  stencila upgrade

  # Check if an upgrade is available without installing
  stencila upgrade --check

  # Force upgrade even if current version is latest
  stencila upgrade --force

Note
  Upgrade downloads the latest release from GitHub and replaces
  the current binary.


###### **Options:**

* `-f`, `--force` — Perform upgrade even if the current version is the latest
* `-c`, `--check` — Check for an available upgrade but do not install it



## `stencila uninstall`

Uninstall this command line tool

**Usage:** `stencila uninstall`

Examples
  # Uninstall Stencila CLI (with confirmation prompt)
  stencila uninstall

Note
  This will permanently remove the Stencila CLI binary from your system.
  Your documents and data will not be affected, only the CLI tool itself.
  You can reinstall Stencila at any time from https://stencila.io or GitHub.




<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

