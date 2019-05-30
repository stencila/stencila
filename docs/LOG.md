# 2018-02-27 Dev call

Tasks for next release in March/April:

- Engine Refactor (Oliver)
  - Paused cells
  - Cell errors delay #534

- Sheet Improvements (Daniel, Michael, in progress)
  - Address all bugs except for those blocked by deficiencies of sheet rendering engine (we need to evaluate Handsontable before continuing there)
  - Daniel: Remove overflowing graphs in Sheets (they cause ux issues and can’t be solved based on the current rendering strategy)
  - Daniel: Remove Source Mode and Preview Mode
  - Daniel: Remove typed cells (it should go away from the interface until we enable issues features)

- Finalise Dar Sheet Spec and update schema (Michael)
  - Article Improvements (Michael, Daniel)
  - Remove all publisher specific stuff
  - Make empty documents good looking (but keeping all functionality, add references etc.)

- Stabilize Dar storage and improve error handling (Michael, close to done)

- Stencila Desktop (Michael, close to ready)
  - How can we bundle the execution engine with the Desktop (remove manual docker setup steps)

- Static Site generator (Michael)
  - Create a static page from a Dar
  - Reduced one column view (ready for mobile)
  - Prominent “Reproduce” Button brings up full Stencila editor (without save functionality) connected to Backend (R sessions etc.)
  - “Download” Button to download a zip file of the Dar (then you can extract it and edit with Stencila Desktop)

- Stencila Website (Nokome)
  - Nice landing page describing the features
  - Kiosk page with introductory examples
  -  Documentation

- Stencila Hub (Nokome)
  - Ability to upload a dar folder (or provide a Github repo)
  - beta.stenci.la/michael/my-document will show a static view first with ability to click on “Reproduce” to bring up the Stencila editor

- Stencila App Icon (Nokome)

- Stencila Language Packages for R, Python & Node:
  - Normalise / finalise Context APIs
  - Release to package archives (ie. CRAN, PyPI, NPM)
  - Registration of functions

- Function libraries: libcore, libtemplate


# 2018-02-06 Dev call

Priorities on stencila/stencila repo:

- saving/reading from (virtual) file systems - available this week?

- focus on removing usability bugs over next week

- cell execution - do not execute if op still pending; allow for 'pausing'/'breakpoints'

- cell error reporting - delay syntax errors (from all contexts); more readable errors from Mini

- creating new documents within project (plus sign tab down the bottom)

- allow for blank Articles:
    - remove 'Article starts here' & 'Article ends here'
    - remove references and footnotes titles?

- do some general usability testing and create issues


# 2018-01-31 Dev call

## Roadmap

Consolidate and document towards a 1.0 release at end of March. No new features, and some prototype features may need to be deferred.

Proposed:

- 1.0.0-preview.1 end Feb 2018

- 1.0.0-preview.2 end Mar 2018 -> enroll interested users in a beta-testing programme

- set up https://beta.stenci.la as permanent deployment channels (in addition to https://builds.stenci.la) to point people to latest version (instead of long, changing builds URLS)


### stencila/stencila

To do:

* update `README.md` :  screenshot, roadmap and feature table - NB

* update `/docs` with clear 'in progress' on every page (using a docsify hook) and merge to master so available at https://stencila.github.io/stencila - NB

* continue with documentation - as much as possible one PR per topic - NB & AP

- saving/reading from (virtual) file systems - MA & OB working on this now; maybe by end of week

- importers/exporters handling cell values

- cell error reporting - delay syntax errors (from all contexts); more readable errors from Mini

- cell execution - allow for 'pausing'/'breakpoints' especially for long running external cells; possibly smart detection; compute on idle; not executing while execution still pending

- issues (StEP 0002) - reinstate issue panel

- comments (StEP 0004) - comments in documents is an often requested feature, can this be combined?

- tests - tests which can be applied across cells (sheets and documents) making assertions about their value

- help panel - search function

- styling: style guide with SASS implementation is in separate repo https://github.com/stencila/style (apply here using SASS `@extend` on `sc-` classes then work on another revision with Salted Herring) 

- code tidy up - remove dead unused code, address TODOs and HACKs

- `Notebook` document type for less strict document schema (?) / more minimal document interface (like in Stencila Desktop v0.27) and elements not supported by JATS4M (e.g. inline value output, value inputs sliders etc); how tied is Stencila to Texture, can we easily develop extensions on top of Texture?

### stencila/mini

- [PR #19](https://github.com/stencila/mini/pull/19) adds support for "lambda expressions" and functions (needed for better syntax and avoid `eval` for higher order functions e.g. `filter`, `map`, `plot`)

### stencila/libcore

- recent changes (incl in stencila/stencila) to make it easier to develop functions, allows for variadic args, named variadic args

- [PR #18](https://github.com/stencila/libcore/pull/18) also adds an `execute` function for executing "operations" (e.g. from Mini lambda expressions)


### stencila/libtemplate : a template repository for creating function libraries

- want to provide an easy way for people to create domain-specific libraries

- hoping to trial a `libdh` with digital humanities folk from UCLA, and perhaps a `libgenomics` or `libneuro`


### stencila/convert

- Implements various converter classes with `import` and `export` e.g. `DocumentMarkdownConverter`, `SheetExcelConverter`. 

- Current focus on `import`

- Most of the "Document" (Articles & Notebooks) converters are wrappers around Pandoc. @hamishmack wrote a JATS reader for Pandoc to make `export` possible: https://github.com/jgm/pandoc/pull/4177 as well as refinements to the existing JATS writer: https://github.com/jgm/pandoc/pull/4178. These changes were released in Pandoc 2.0.6. This package downloads Pandoc if the right version is not available.

- The Sheet converters use SheetJS package which acts like Pandoc with a format-agnostic in-memory representation of spreadsheets. Initial implementation of CSV and Excel converters including parsing of Excel formulas to convert them to Mini expressions (or perhaps just Stencila operations).

To do:

* converters based on `DocumentPandocConverter` : put appropriate JATS wrappers around the body content that Pandoc creates; ensure compatibility with [JATS4M](https://github.com/substance/dar/blob/master/specs/JATS4M.md); do a whole lot of testing (import, export and roundtrip)

* re-implement `DocumentRMarkdownConverter` to target JATS4M/Dar (currently targets HTML) and based on `DocumentMarkdownConverter`

* re-implement `DocumentJupyterConverter` to target JATS4M/Dar (currently targets HTML); once initial version of that is done, contact Jupyter team, perhaps port to Python and contribute to https://github.com/jupyter/nbconvert; test on a variety of notebooks


### stencila/node : Stencila Host in Node.js
### stencila/r : Stencila Host in R
### stencila/py : Stencila Host in Python

To do:

- various changes required for new(ish) `analyseCode`/`executeCode` API
- improve ease of connection
- finish implementing `JupyterContext`

### stencila/images : environments and container images with one or more Stencila Hosts

- Recently refactored to use Nix instead of Dockerfiles providing better reproducibility, no package dependency conflicts and better composability e.g. use `stencila/base/r` if you only want R, use `stencila/base` (which combines `stencila/base/r`, `stencila/base/py`, `stencila/base/node`) if you want all language contexts in the image.

To do:

* publish manifest of the content's of each image i.e. package versions

* write documentation

* #13 needs CONTRIBUTING.md including PRs for adding new packages to existing images and PRs for new images

- fix test (expecting outdated package version)

- CI for all images (currently just `base`) - will require paid Travis CI account and/or Nix cache due to long build times

- establish protocol for updating, versioning, channels (possibly following NixPkg builds ie. every six months, 17.09, 18.03 etc)

- refactor and integrate `shrink-docker.sh` script so that user can connect to running container, execute a document against it, and then obtain minimal image as a `tar.gz`

- #11 create a Nix cache for faster build times


### stencila/cli : command line interface
### stencila/desktop : desktop application

- In hiatus, little development in last 6 months, awaiting a usable release

To do:

- signed builds for Windows and Mac OS

- setup auto updating


### stencila/hub : web application (https://stenci.la)

- In hiatus, little development for over a year

To do:

* update the landing page removing outdated instructions, providing more description and linking to documentation etc

- planning a sprint for end Feb to reinvigorate with login for beta users, ability to share documents etc.
