---
title: What is an rPNG?
description: A Reproducible PNG is an image in a PNG format
authors:
  - 'Colette Doughty <colette@stenci.la> (https://stenci.la)'
---

rPNG are Reproducible PNG's. In essence **rPNG** is an image in a PNG format that has the source code/text embedded into it => ReproduciblePNG.
This is Stencila’s approach to bridging the gap between reproducible documents and legacy formats such as Word documents.

![](https://i.imgur.com/kb1u4Eg.png)

- They are static images and figures that contain the underlying codes that are used to generate them.

- Our file converters namely **encoda** create rPNG's during the conversion process for non-executable file formats (Google Docs, Word files, PDFs) and this enables collaboration between researches who use workflows of various techincal levels. So if one researcher prefers working in Jupyter Notebook or RMarkdown and another prefers working in Word and Excel that is ok.
  They can still alter documents in their preferred formats and Stencila will preserve the source code between conversions.

- It enables back-and-forth conversion between executable and non-executable formats.
