# Roadmap

Welcome to the Stencila Roadmap 👋. You might also be interested in our [discussion](https://github.com/stencila/stencila/discussions) forum and [sprint planning](https://github.com/orgs/stencila/projects/6/views/1).

We've been make some significant changes to Stencila over the past six months including porting a lot of our code to Rust and developing local-first destop and CLI tools. Many of the features below are already partially implemented or documented (🔶). This roadmap is focussed on progressively _fully implementing, testing, documenting and "officially" releasing_ each feature (🟦), after which it will mostly receive mainly maintenance attention (🟢).

The roadmap is aspirational and changeable: not all features listed will completed in the order, or to the schedule currently shown (and some may not be completed at all). Obviously, further away releases are likely to be more uncertain. Please see the roadmap contributing [guidelines](#guidelines) at the end of this document.

|                                                 | [Alfonsino] | [Bonito] | [Candiru] | [Dorado] | [Escolar] | [Flathead] |
| ----------------------------------------------- | ----------: | -------: | --------: | -------: | --------: | ---------: |
|                                                 |    Nov 2021 | Dec 2021 |  Jan 2022 | Feb 2022 |  Mar 2022 |   Apr 2022 |
|                                                 |
| **WRITE ✏️**                                    |
| **Authoring formats** <sup>[1](#1)<sup>         |
| Markdown                                        |          🔶 |       🟦 |        🟢 |       🟢 |        🟢 |         🟢 |
| R Markdown                                      |          🔶 |       🟦 |        🟢 |       🟢 |        🟢 |         🟢 |
| Jupyter Notebook                                |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| LaTeX                                           |          🔶 |       🔶 |        🔶 |       🟦 |        🟢 |         🟢 |
| Microsoft Word                                  |          🔶 |       🔶 |        🟦 |       🟢 |        🟢 |         🟢 |
| Open Document Text                              |             |          |           |          |        🟦 |         🟢 |
| Google Docs                                     |             |          |           |          |           |         🟦 |
| **Other formats** <sup>[1](#1)<sup>             |
| HTML                                            |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| PDF                                             |          🔶 |       🔶 |        🔶 |       🔶 |        🟦 |         🟢 |
| JSON                                            |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| JSON5                                           |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| TOML                                            |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| YAML                                            |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| XML                                             |             |          |        🟦 |       🟢 |        🟢 |         🟢 |
| JATS                                            |             |          |        🟦 |       🟢 |        🟢 |         🟢 |
| **Editors**                                     |
| Jupyter Notebook <sup>[2](#2)<sup>              |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| RStudio <sup>[3](#3)<sup>                       |             |       🟦 |        🟢 |       🟢 |        🟢 |         🟢 |
| Microsoft Word / Libre Office <sup>[4](#4)<sup> |             |          |        🟦 |       🟢 |        🟢 |         🟢 |
| Code editor <sup>[5](#5)<sup>                   |          🔶 |       🟦 |        🟢 |       🟢 |        🟢 |         🟢 |
| WYSIWYG editor                                  |             |          |           |          |        🟦 |         🟢 |
|                                                 |
| **RUN ⚡**                                      |
| **Languages** <sup>[6](#6)<sup>                 |
| Calc                                            |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| Python                                          |          🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| R                                               |          🔶 |       🟦 |        🟢 |       🟢 |        🟢 |         🟢 |
| JavaScript                                      |          🔶 |       🔶 |        🔶 |       🟦 |        🟢 |         🟢 |
| TypeScript                                      |             |          |           |       🟦 |        🟢 |         🟢 |
| Bash                                            |
| Juila                                           |
| SQL                                             |
| Rust                                            |
| **Execution** <sup>[7](#7)<sup>                 |
| Linear                                          |🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| Manual                                          |🟦 |       🟢 |        🟢 |       🟢 |        🟢 |         🟢 |
| Interrupt                                       |
| Reactive                                        |
| Reactive polyglot                               |
| Concurrent                                      |
| **Reproducibility**                             |
| Dockerfile                                      |
| Nix profile                                     |
|                                                 |
| **CONNECT 🔗**                                  |
| **Import**                                      |
| **Enrich**                                      |
|                                                 |
| **COLLABORATE 👩‍🔬**                              |
| **Sync**                                        |
| **Snapshots**                                   |
| **Merging async changes**                       |
| **Realtime collab**                             |
|                                                 |
| **PUBLISH 📢**                                  |
| **Themes**                                      |
| **Performance**                                 |
| **Identifiers**                                 |
|                                                 |
| **EXTEND 🔌**                                   |
| **Plugin API**                                      |

<!-- Named release links (allow for more columns without line wrapping) -->

[alfonsino]: #alfonsino
[bonito]: #bonito
[candiru]: #candiru
[dorado]: #dorado
[escolar]: #escolar
[flathead]: #flathead

## Notes

1. <a name="1"></a>For all formats, _done_ means that bi-directional conversion (i.e. encoding **and** decoding) is implemented and tested (using end-to-end and snapshot tests). Documentation should be available for all formats, but for authoring formats in particular should provide guides for users. Note that most of these formats are already functional in Stencila Encoda but some still need to be ported of plugged-in to Stencila CLI and Desktop.

2. <a name="2"></a> Able to edit a notebook within the Jupyter Notebook frontend, see a live, themed, executable (in the kernel started by the frontend) preview using Stencila CLI or desktop.

3. <a name="3"></a> Able to edit a R Markdown file within R Studio, see a live, themed, executable (in the main RStudio session) preview using Stencila CLI or desktop.

4. <a name="4"></a> Able to edit a `docx` file within Microsoft Word, see a live, themed, executable preview using Stencila CLI or desktop and have executable elements persisted in the `docx` file.

5. <a name="5"></a> CodeMirror based editor.

6. <a name="6"></a>For languages, _done_ means (a) tested and documented use with kernel for that language and (b) semantic analysis of code using Tree Sitter (a Rust `compile-<lang>` feature flag; documenting should wait until the "Reactive" execution feature is done).

7. <a name="7"></a>Execution features described a little more:
   - Linear: Can press a run button, or do `stencila execute <file>`, and execute all executable nodes (e.g. `CodeChunk`) in the order that they appear in the document.
   - Manual: Can press a run button on an individual node (e.g. `CodeExpression`) and have it run
   - Interrupt: Can interrupt execution of individual nodes, or of the entire document
   - Reactive: Automatic dependency analysis, with manual override, with reactive updates on changes
   - Reactive polyglot: Ability to pass values between languages as needed based on dependency analysis
   - Concurrent: Dependency analysis to identify which nodes can be executed concurrently

## Named releases

### Alfonsino

### Bonito

### Candiru

### Dorado

### Escolar

### Flathead

## Guidelines

### Named releases

This repository uses a continuous deployment approach with [Semantic Versioning](https://semver.org/), [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), and [`semantic-release`](https://github.com/semantic-release/semantic-release). The result is a large number of frequent semantically numbered [releases](https://github.com/stencila/stencila/releases).

This roadmap introduces the notion of special "named releases". An automatic, semantic release will be "named" to mark the completion of a bundle of features which enable a particular user story.

We use the convention of naming releases after fish; in alphabetical order. Because fish are cool 🐟. Choose a name from [Wikipedia's list of fish common names](https://en.wikipedia.org/wiki/List_of_fish_common_names) or elsewhere. For interest and aesthetics, prefer single-word over multi-word, unusual (e.g. Arowana) over well-known (e.g. Anchovy), and those that do not contain "fish" (e.g. Flagfin) over those that do (e.g. Flagfish).

### User stories

Each named release should have a brief user story(ies) describing it motivations and goals in terms of outcomes for users.
