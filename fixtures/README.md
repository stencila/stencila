# Fixtures

Author: Noo

Date: 2 July 2021 00:00:00 +00:00

This folder contains some examples of content that can be opened using Stencila including [`projects`](projects), [`articles`](articles), and [`media`](media) files. They are used in automated and manual testing. Be careful when changing files since that will probably break tests that rely upon them.

Note the Stencila converter plugin, [`encoda`](https://github.com/stencila/encoda), has a more complete set of fixtures. The fixtures here are not intended for testing conversion and do not necessarily encompass the entire range of document node types. Rather, they are to test the interfaces developed in this repository handle alternative files types as expected.

## 📂 Projects

The [`projects`](projects) folder contains projects with a variety of structures e.g.

- [`empty`](projects/empty): nothing in it (except a `.gitignore`)
- [`readme`](projects/readme): has a single `README.md` (its "main" file)
- [`manifest`](projects/manifest): has a `project.json` manifest file
- [`themed`](projects/themed): has a `theme` set in `project.json`
- [`mid`](projects/mid): several sub-folders with varying numbers and types of files
- [`shallow`](projects/shallow): no sub-folders, just several files
- [`deep`](projects/deep): has a deeply nested sub-folder structure

## 📜 Articles

The [`articles`](articles) folder contains [`Article`](https://schema.stenci.la/Article) documents having varying characteristics and formats e.g.

- [`elife-small.json`](articles/elife-small.json): A smallish eLife article useful for things like visual regression tests
- [`era-plotly.json`](articles/era-plotly.json): An executable research article, written as a Jupyter Notebook, with Plotly code chunk outputs
- [`simple.tex`](articles/simple.tex): A simple LaTeX article

Run `make -C articles` to update the article fixtures.

## 📷 Media

The [`media`](media) folder contains documents that are derived from [`MediaObject`](https://schema.stenci.la/MediaObject) including [`ImageObject`](https://schema.stenci.la/ImageObject), [`AudioObject`](https://schema.stenci.la/AudioObject), and [`VideoObject`](https://schema.stenci.la/VideoObject).

- [`grapefruit.jpg`](media/grapefruit.jpg)
- [`trex.mp3`](media/trex.mp3)
- [`flower.mp4`](media/flower.mp4)

All sample media files are from https://developer.mozilla.org/ and are in the public domain (Creative Commons CC-0).

## 🍕 Fragments

The [`fragments`](fragments) folder contains many small examples of parts of documents in a variety of formats. These are used in snapshot-based unit tests of decoding and encoding functions. Generally, each file should be focussed on one node type in one format.
