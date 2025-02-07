# Change Log

## 0.0.23 2025-02-05

- Based on Stencila CLI v2.0.0-beta.25 which includes new document tracking functionality.

## 0.0.22 2025-01-31

- Fixes and improvements to "Chat to create..." context menu.

- Improved node type syntax highlighting for `/create` and `::: create` Markdown directives.

## 0.0.21 2025-01-31

- Load kernel, prompt, and model lists early to reduce waits when user interfaces first loaded. 

- Based on Stencila CLI v2.0.0-beta.24 which includes new code linting and formatting functionality, and quality, cost, and speed scores for models.

## 0.0.20 2025-01-23

- Improvements to chat interfaces, particularly in multi-model scenarios.

- Improvements to interfaces for executable nodes giving more feedback on the execution status and whether execution is required.

- Based on Stencila CLI v2.0.0-beta.23.

## 0.0.19 2025-01-22

- Adds interfaces for standalone chats, chats about documents, and chats to insert specific types of content.

- Improvements to autocomplete, including file path completions for include blocks and images.

- Adds preliminary support for LaTeX documents.

- Based on Stencila CLI v2.0.0-beta.22.


## 0.0.18 2024-11-27

- Remove more advanced syntax from snippets to de-clutter autocomplete dropdown for Stencila Markdown and MyST.

- Improved syntax highlighting for MyST Markdown based on https://github.com/jupyter-book/vscode-mystmd.

- Based on Stencila CLI v2.0.0-beta.20 which adds initial support for Pandoc based codecs.

## 0.0.17 2024-11-24

- Adds gutter markers for document nodes and settings for enabling/disabling and appearance.

- Adds initial support for Quarto Markdown.

- Based on Stencila CLI v2.0.0-beta.19 which adds initial support for Quarto Markdown, and language server support for gutter markers and fixes for document synchronization state.

## 0.0.16 2024-11-21

- Adds a "Setup View" for guiding users through setting up a Stencila CLoud account, adding API keys, and setting user details.

- Adds a "Stencila Shell" terminal to allow you to use the Stencila CLI that ships with the VSCode extension.

- Adds support for Cloudflare generative AI models.

- Based on Stencila CLI v2.0.0-beta.18 which includes a fix for `@media` rules in CSS, and enables opening and saving of prompts.

## 0.0.15 2024-11-17

- Adds bidirectional scroll syncing between source editor and preview panel.

- Open preview panel when opening walkthroughs.

- Adds actions to walkthrough previews for expanding the next, and all, walkthrough steps.

- Based on Stencila CLI v2.0.0-beta.17 which includes a fix for which kernel is used for executable nodes when no language is specified.

## 0.0.14 2024-11-14

- Fixes to AI commands walkthrough

- Adds demo screen casts to README

## 0.0.13 2024-11-12

- Reduces the diagnostic level of Markdown pre-decoding checks and increases the debounce delay.

- Filtering of commands and prompts based on description.

- Based on Stencila CLI v2.0.0-beta.16 which includes fixes related to commands within walkthroughs and display of the prompts provided in commands.

## 0.0.12 2024-11-11

- Fixes related to having multiple preview tabs opens at once

- Based on Stencila CLI v2.0.0-beta.15 which includes improvements to incremental updates of previews, adds a check for unbalanced dollar delimited math in Markdown, and fixes publishing of images within raw HTML blocks.

## 0.0.11 2024-11-08

- Based on Stencila CLI v2.0.0-beta.14 which changes the syntax for "self-closing" and "next-block" commands in Stencila Markdown, has fixes to the default theme, and switches to using `idiomorph` for incremental updates of the preview panel,

## 0.0.10 2024-11-06

- Fix to sidebar and model icons.

- Improvements to AI Commands walkthrough.

- Based on Stencila CLI v2.0.0-beta.13 which includes support for configurable themes, and fixes to styled blocks.

## 0.0.9 2024-11-05

- Allow user to select a format for walkthroughs.

- Fixed issue with node chips not appearing within walkthrough steps.

- Further fixes suggestion scrolling in previews, including handling scrolling to original.

## 0.0.8 2024-11-05

- Added LLM Commands and Mermaid walkthroughs.

- Fixed suggestion scrolling in previews.

## 0.0.7 2024-11-05

- New approach to walkthroughs based on Stencila documents.

- Adds a prompt filter and use command.

- Uses `Ctrl/Cmd+F1` for the Stencila command menu (instead of `F2`).

- Based on Stencila CLI v2.0.0-beta.12 which includes support for walkthroughs and bug fixes.

## 0.0.6 2024-10-30

- Adds `errorlens` as a package dependency.

- Fixes hover preview for Mermaid and other non-standard image code chunk outputs.

- Based on Stencila CLI v2.0.0-beta.11 which includes fixes for re-initializing and updating builtin prompts, improved error handling for Mermaid code chunks.

## 0.0.5 2024-10-30

- Added Stencila side bar with tree views for kernels, prompts, and models.

- Updating of the document preview is done using patches, as with the CLI server, rather than replacing the entire content. This maintains the UI state between updates and should improve performance, particularly for large documents.

- Reverted to using VSCode's secrets store for setting `STENCILA_API_TOKEN` and other secrets.

- Based on Stencila CLI v2.0.0-beta.10 which includes a new carousel interaction for command suggestions in Markdown-based documents and in document preview.

## 0.0.4 2024-10-14

- Based on Stencila CLI v2.0.0-beta.9 which improves the determination of whether code chunks require re-execution (by assigning unique ids to each kernel instance) and exposes information about the binary used by microkernels.

## 0.0.3 2024-10-11

- Added commands and settings for viewing logs from Language Server

- Fixed Language Server so user name and affiliation settings are only applied if missing in user object setting.

- Based on Stencila CLI v2.0.0-beta.8

## 0.0.2 2024-10-09

- Initial release based on Stencila CLI v2.0.0-beta.7

- Fixed sign out from Stencila Cloud

- Fixed percent encoding of file paths in LSP server

- Improvements to menu labels

## 0.0.1 2024-10-08

- Initial pre-release release
