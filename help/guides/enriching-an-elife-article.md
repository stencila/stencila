---
id: 4184684
collectionId: 2422090
title: Enriching an eLife article
description: A user guide for enriching a published eLife article
relatedArticles:
  - ../projects/publish-a-project.md
authors:
  - Colette Doughty<colette@stenci.la> (https://stenci.la)
  - Alexander Ketchakmadze<alexander@stenci.la> (https://stenci.la)
published: true
---

👋 _This guide is under active development. Please feel free to contribute using the link at the bottom of the page!_

## Introduction

Stemming from a longstanding legacy of print, the primary outputs of today’s research publishing workflows are based
around PDF and plain HTML. These formats strip out much of the underlying richness from research findings that
increasingly rely on the use of code and data.

Stencila has partnered with eLife, a leading publisher that aims to improve research communication through open science
and open technology innovation.

![](https://elifesciences.org/assets/patterns/img/patterns/organisms/elife-logo-xs@1x.24c98c55.png)

The new, open-source, and web-native Executable Research Article (ERA) publication format allows you to embed code and
data natively, in a fully reproducible document that is designed to help transparency, collaboration, training, and
reproducibility.

[Learn more about eLife’s ERA initiative.](https://elifesciences.org/)

## Overview

The enrichment process will consist of:

- Using Stencila Hub to fetch the published article XML
- Using Stencila Hub to convert the article XML back to the format you used to author the article for submission.
- Replacing the static figures in the article with Code Chunks
- Uploading the enriched article and supporting assets to a Stencila project

[![](https://i.imgur.com/i9EbbmA.png)](https://i.imgur.com/i9EbbmA.png)

## Getting started

This guide assumes that you are already signed in to Stencila, and have set up a new project for your eLife article. If
you haven't signed up yet, see the [creating an account](../hub/getting-started.md) and [creating a project](../hub/projects/create-a-project.md) guides to get started.

_There is an interactive version of this guide if you'd rather follow along on the Hub._

[Switch to Interactive Tutorial](https://hub.stenci.la/projects/?userflow=fede9de4-9bd4-4521-9c8e-6c15345cc157)

This guide uses the following eLife article as an example: Lewis, L. Michelle, et al. ["Replication Study:
Transcriptional amplification in tumor cells with elevated c-Myc." eLife 2018;7:e30274 DOI:
10.7554/eLife.30274.](https://elifesciences.org/articles/30274)

![](https://i.imgur.com/pqexnWj.png)

## Files vs Sources: Pulling in the content of the original article from the eLife website

An important concept to be aware of is that of linked (file) sources.
In addition to uploading files to a project, you can link to sources that are hosted elsewhere, such as public websites and GitHub.
These sources remain on the external service, but a versioned copy is downloaded and stored with your project to maintain reproducibility.
You can go to the sources tab and update the local copies of your source files by pulling them periodically.

For this guide, we will be linking to the published version of the article. This will allow you to save the article in
one of several formats so that you can replace the static tables and figures with reproducible code chunks.

[Add an eLife source to your project](../hub/sources/elife.md)

## Converting article to another format for editing

Now that you have a link to the published version of your article, you can decide which format you will use to transform
it into a reproducible article. See the available formats, under the **Actions** menu for the linked article.

There are two primary pathways for enriching an article:

1. Save the article as a Google Doc and use the Stencila plugin for Google Docs to replace static tables and figures with reproducible versions
2. Save the article as a Jupyter Notebook or R Markdown document, download it, and edit it locally.

![](https://i.imgur.com/Kq0iDix.png)

Click on your desired format to begin the conversion process.
**The conversion process can take some time.** In the background, Stencila Encoda needs to do several things:

1. Search the [eLife article Github repository](https://github.com/elifesciences/elife-article-xml) to get the most recent version number
2. Download the XML of the article from that repository
3. Download images for each of the figures in the article
4. Convert the XML and images to Google Docs format and upload the file to Google.

Don’t worry if the **Save** button spins for some time after you press it. When the conversion is complete, you’ll have
a new entry in the project’s list of files:

![](https://i.imgur.com/y8pmshl.png)

## Enriching the article

Now that you've converted the article into an easily editable format, you can download it to your computer to edit using
your tool of choice such as R Studio or Jupyter Notebooks.

The enrichment process consists of finding static figures in the article and replacing them with Code Chunks.
Please refer to the help documentation of [R Studio](https://bookdown.org/yihui/rmarkdown/r-code.html), [Jupyter
Notebooks](https://jupyter-notebook.readthedocs.io/en/stable/notebook.html), or your editor for specific instructions.

If you have questions please don't hesitate to reach out to us via the chat widget on our website or emailing us at hello@stenci.la
