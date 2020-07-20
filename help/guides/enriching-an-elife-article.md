---
id: 4184684
title: Enriching an eLife article
description: A user guide for enriching a published eLife article
published: true
---

👋 _This guide is under active development. Please feel free to contribute using the link at the bottom of the page!_

## Introduction

Stemming from a longstanding legacy of print, the primary outputs of today’s research publishing workflows are based around PDF and plain HTML. These formats strip out much of the underlying richness from research findings that increasingly rely on the use of code and data.

Stencila has partnered with eLife, a leading publisher that aims to improve research communication through open science and open technology innovation.

![](https://elifesciences.org/assets/patterns/img/patterns/organisms/elife-logo-xs@1x.24c98c55.png)

The new, open-source, and web-native Executable Research Article (ERA) publication format allows you to embed code and data natively, in a fully reproducible document that is designed to help transparency, collaboration, training, and reproducibility.

[Learn more about eLife’s ERA initiative.](https://elifesciences.org/)

## Getting started

This guide assumes that you are already signed in to Stencila, and have set up a new project for your eLife article. See these guides for help with getting started:

[Create a account](../hub/getting-started.md)

[Create a project](../hub/projects/create-a-project.md)

This guide uses the following eLife article as an example: Lewis, L. Michelle, et al. ["Replication Study: Transcriptional amplification in tumor cells with elevated c-Myc." eLife 2018;7:e30274 DOI: 10.7554/eLife.30274.](https://elifesciences.org/articles/30274)

![](https://i.imgur.com/pqexnWj.png)

## Pulling in the content of the original article from the eLife website

An important concept on Stencila is that of linked file sources. In addition to uploading files to a project, you can link to files that are hosted elsewhere, such as Dropbox and Github. For this guide, we will be linking to the published version of the article. This will allow you to save the article in one of several formats so that you can replace the static tables and figures with reproducible code chunks.

[Add an eLife source to your project](../hub/sources/elife.md)

## Converting article to another format for editing

Now that you have a link to the published version of your article, you can decide which format you will use to transform it into a reproducible article. See the formats that are available, under the **Actions** menu for the linked article.

There are two primary pathways for this transformation:

1. Save the article as a Google Doc and use the Stencila plugin for Google Docs to replace static tables and figures with reproducible versions
2. Save the article as a Jupyter Notebook or R Markdown document, download it, and edit it locally.

![](https://i.imgur.com/Kq0iDix.png)

The advantage of option A is that it will use a reproducible execution environment, hosted by Stencila, which will be exactly the same as when the article is published. If you choose to use option B, there is likely to be more effort required to ensure consistency between then the computational environment on your local machine and on Stencila Hub. **The rest of this guide assumes option A, i.e. using Google Docs, and the Stencila plugin, to enliven the article.**

Select **Actions > Save as > Google Doc.** Choose a different name to the default, e.g. _elife-article-30274.gdoc,_ if necessary then press the **Save** button.

![](https://i.imgur.com/68vg6fR.png)

**The conversion from the linked eLife article to a Google Doc (or any other format can take some time).** In the background, Stencila Encoda needs to do several things:

1. Search the [eLife article Github repository](https://github.com/elifesciences/elife-article-xml) to get the most recent version number
2. Download the XML of the article from that repository
3. Download images for each of the figures in the article
4. Convert the XML and images to Google Docs format and upload the file to Google.

Don’t worry if the **Save** button spins for some time after you press it. When the conversion is complete, you’ll have a new entry in the project’s list of files:

![](https://i.imgur.com/y8pmshl.png)

Open the article for editing by selecting **Actions > Open in Google Docs.** This will open the article in Google Docs:

![](https://i.imgur.com/1eQrPYS.png)

You may notice that some content from the published article is missing in the Google Doc. This can arise because of incomplete conversion from XML to Google Docs format.
