---
title: Files vs Sources
published: true
description: Difference between project files and sources
id: 4278920
collectionId: 2470979
relatedArticles:
  - ../hub/sources/upload.md
  - ../hub/sources/elife.md
authors:
  - 'Alexander Ketchakmadze <alexander@stenci.la> (https://stenci.la)'
---

An important concept to be aware of is that of linked (file) sources.

In addition to uploading files to a project, you can link to sources that are hosted elsewhere, such as public websites and GitHub. These sources remain on the external service, but a versioned copy is downloaded and stored with your project to maintain reproducibility. You can go to the sources tab of your project to update the local copies of your source files by pulling them periodically. When you start a compute session to run your project, we will mount the pulled files into the session.

Files on the other hand live and are managed on Stencila servers and do not need to be pulled.
