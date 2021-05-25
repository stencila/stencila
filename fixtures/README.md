# Fixtures

This folder contains some examples of content that can be opened using Stencila including [`articles`](articles) and [`projects`](projects). They are used in automated and manual testing.

Be careful when changing files since that will probably break tests that rely upon them.

## 📜 Articles

The [`articles`](articles) folder contains [`Article`](https://schema.stenci.la/Article) JSON documents having varying characteristics e.g.

- [`elife-small.json`](projects/elife-small.json): A smallish eLife article useful for things like visual regression tests
- [`elife-mid.json`](projects/elife-mid.json): A mid-sized eLife article with several figures and tables
- [`era-plotly.json`](projects/era-plotly.json): An executable research article, written as a Jupyter Notebook, with Plotly code chunk outputs

Run `make -C articles` to update the article fixtures.

## 📂 Projects

The [`projects`](projects) folder contains projects with a variety of structures e.g.

- [`empty`](projects/empty): nothing in it (except a `.gitignore`)
- [`readme`](projects/readme): has a single `README.md` (its "main" file)
- [`manifest`](projects/manifest): has a `project.json` manifest file
- [`mid`](projects/mid): several sub-folders with varying numbers and types of files
- [`shallow`](projects/shallow): no sub-folders, just several files
- [`deep`](projects/deep): has a deeply nested sub-folder structure
