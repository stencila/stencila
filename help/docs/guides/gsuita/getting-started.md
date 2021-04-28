---
title: Getting started
description: Overview of the workflows for executing code in your documents
authors:
  - 'Alexander Ketchakmadze <alexander@stenci.la> (https://stenci.la)'
relatedArticles:
  - ./installation.md
  - ./interface-overview.md
---

## Overview

1.  Link document to Stencila project
2.  Create a Block
3.  Write code or equation
4.  Run block to preview results
5.  Repeat steps 2 and 3 until satisfied
6.  Insert figure into document

## Linking document to a Stencila project

The add-on allows you to execute code and embed resulting figures in your document by communicating with the Stencila servers.

To ensure that documents use the right library versions and have access to project files, you need to link your Google Doc to an existing Stencila Hub project.

When opened, the add-on will detect if the document is not linked to a project, and ask you to link it to a project if needed.

Once linked to a project, the add-on will start a Docker container on a remote server with all your project assets. You will then be able to create, evaluate, and embed either Code or Equation Blocks.

## Code & Equations Blocks

Blocks are the foundational elements for writing executable Google Docs.

They are PNG images enriched with either source code or LaTeX equations depending on the type of block, as well as other metadata such as resulting output and time of execution.

This allows you to embed generated charts and other figures as images within the flow of your document. The add-on is able to detect these images, allowing you to re-execute and modify them at will.

## Inserting generated figures into the document

Due to limitations with Google Docs add-ons, the process to generate a reproducible figure and embed it into the document can be relative slow.

In order to speed up your workflow, we don’t immediately update the document contents when you run a block. Instead we show you a preview of the results in the sidebar, allowing you to rapidly refine and iterate the code.

Once you are satisfied with the results, click the `Save` button which will insert the figure into the document. If the figure being edited was already present in the document, the add-on will instead update the image with the new contents.
