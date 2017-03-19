<div align="center">
  <a href="https://stenci.la/about">
    <img src="https://raw.githubusercontent.com/stencila/stencila/master/images/logo-name-beta.svg" alt="Stencila">
  </a>
  <br>
  <a href="#status">Status</a>&nbsp;|&nbsp;
  <a href="#features">Features</a>&nbsp;|&nbsp;
  <a href="#discuss">Discuss</a>&nbsp;|&nbsp;
  <a href="#develop">Develop</a>
  <br>
</div>

### Status

[![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila)
[![Build status](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila)
[![Code coverage](https://codecov.io/gh/stencila/stencila/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/stencila)
[![Dependency status](https://david-dm.org/stencila/stencila.svg)](https://david-dm.org/stencila/stencila)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

### Features

Stencila is a platform for creating, collaborating on, and sharing data driven content. Content that is **transparent** and **reproducible**, like [RMarkdown](https://github.com/rstudio/rmarkdown) and [Jupyter Notebooks](http://jupyter.org/). Content that can be **versioned** and **composed** just like we do with open source software using tools like [CRAN](https://cran.r-project.org/web/packages/available_packages_by_name.html) and [NPM](https://www.npmjs.com/). And above all, content that is **accessible** to non-coders, like [Google Docs](https://en.wikipedia.org/wiki/Google_Docs,_Sheets_and_Slides) and [Microsoft Office](https://en.wikipedia.org/wiki/Microsoft_Office).

![](images/screenshot.png)

Features                                                                    | State
--------------------------------------------------------------------------- | :------------:
**Documents**                                                               | ✓
**Sheets**                                                                  | 
**Slides**                                                                  | 
                                                                            |
**Static content**                                                          |
Paragraph                                                                   | ✓
Headings                                                                    | ✓
Blockquote                                                                  | ✓
Image                                                                       | 
List                                                                        | ✓
Table                                                                       | ✓
Strong & emphasis                                                           | ✓
Link                                                                        | ✓
Subscript & superscript                                                     | ✓
Code block                                                                  | ✓
Math (AsciiMath and Tex)                                                    | ✓
Discussions                                                                 | 
**Reproducible content**                                                    |
Number input (range slider)                                                 | ✓
Select input (name value pairs)                                             | ✓
File input (CSV etc)                                                        | 
Code cell                                                                   | ✓
Output (value display)                                                      | ✓
**Execution contexts**                                                      |
JavaScript                                                                  | ✓
Node.js                                                                     | ✓
R                                                                           | 0.26
Python                                                                      | 0.26
Julia                                                                       | 
Jupyter kernels                                                             | 0.27
**Functions**                                                               |
Statistics (`sum`, `mean`, `variance`, ...)                                 | 0.26
Data manipulation (`filter`, `sort`, `aggregate`, ...)                      | 0.26
Data visualization (`plot`, `title`, `theme`, ...)                          | ✓
Contribute more...                                                          | ✓
**Formats**                                                                 |
HTML                                                                        | ✓
JATS                                                                        | 
Markdown                                                                    | 0.26
RMarkdown                                                                   | 0.26
Jupyter Notebook                                                            | 
Microsoft Office                                                            | 
Open/Libre Office                                                           | 
**Apps and packages**                                                       |
[Stencila Desktop](https://github.com/stencila/desktop) (native apps for Windows, Mac OSX, Linux)               | ✓
[Stencila for Python](https://github.com/stencila/python) (`pip install stencila`)                              | 0.26
[Stencila for R](https://github.com/stencila/r) (`install.packages('stencila')`)                                | 0.26
[Stencila for Node.js](https://github.com/stencila/node) (`npm install stencila-node`)                          | ✓

Ticks indicate a feature is in the latest release. Numbers (e.g. `0.27`) indicate the release that a feature is planned for. We generally only plan one or two releases ahead. We aim to release every 1 - 2 months.

### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/stencila/issues/new), add to [existing issues](https://github.com/stencila/stencila/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.

### Develop

```
git clone https://github.com/stencila/stencila.git
cd stencila
npm install
npm run start
```

Now you can access the different interfaces in the browser:

- [http://localhost:4000/examples/dashboard](http://localhost:4000/examples/dashboard)
- [http://localhost:4000/examples/document](http://localhost:4000/examples/document)

Most development tasks can be run directly using JavaScript tooling (`npm`) or via `make`.

Task                                                    |`npm`                  | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Run the development server                              | `npm start`           | `make run`
Check code for lint                                     | `npm run lint`        | `make lint`
Run tests                                               | `npm test`            | `make test`
Run tests in the browser                                | `npm run test-bundle` | `make test-bundle`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build documentation                                     | `npm run docs`        | `make docs`
Serve and watch docs for updates                        | `npm run docs-serve`  | `make docs-serve`
Clean                                                   |                       | `make clean`

To help out, [get in touch](https://gitter.im/stencila/stencila), checkout the [platform-wide, cross-repository kanban board](https://github.com/orgs/stencila/projects/1), or just send us a pull request! Please read our contributor [code of conduct](CONDUCT.md).

Docs are published using Github Pages, so to update them after making changes run make docs, commit the updated docs and do a git push.