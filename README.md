<div align="center">
  <a href="https://stenci.la">
    <img src="https://raw.githubusercontent.com/stencila/stencila/master/images/logo-name.png" alt="Stencila">
  </a>
</div>

Stencila is a platform for creating, collaborating on, and sharing data driven content. Content that is **transparent** and **reproducible**, like [RMarkdown](https://github.com/rstudio/rmarkdown) and [Jupyter Notebooks](http://jupyter.org/). Content that can be **versioned** and **composed** just like we do with open source software using tools like [CRAN](https://cran.r-project.org/web/packages/available_packages_by_name.html) and [NPM](https://www.npmjs.com/). And above all, content that is **accessible** to non-coders, like [Google Docs](https://en.wikipedia.org/wiki/Google_Docs,_Sheets_and_Slides) and [Microsoft Office](https://en.wikipedia.org/wiki/Microsoft_Office).

![](https://raw.githubusercontent.com/stencila/stencila/master/images/screenshot.png)

### Roadmap

Stencila is still at an early beta stage: there are likely to be missing features, bugs and API changes. But we would :heart: to get your suggestions and :bug: reports. Get help from the [community](https://community.stenci.la), create a [new issue](https://github.com/stencila/stencila/issues/new), or join the [chat](https://gitter.im/stencila/stencila).

- ![prod](https://img.shields.io/badge/status-prod-green.svg) = ready for production use
- ![beta](https://img.shields.io/badge/status-beta-yellow.svg) = ready for beta user testing
- ![alpha](https://img.shields.io/badge/status-alpha-red.svg) = ready for alpha testing; use with caution
- numbers (e.g. `0.31`) = planned release

We generally only plan one or two releases ahead. We aim to release every 1-2 months, towards a 1.0 release in early 2018. Checkout the current release [milestones](https://github.com/stencila/stencila/milestones).

Feature                                | Ready
:------------------------------------- | :------------:
Documents                              | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Datatables                             | 0.28
Sheets                                 | 0.29
**Static content**                     |
Paragraph                              | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Headings                               | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Blockquote                             | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Image                                  | 0.30
List                                   | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Table                                  | 0.30
Strong & emphasis                      | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Link                                   | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Subscript & superscript                | ![prod](https://img.shields.io/badge/status-prod-green.svg)
Code block                             | 0.30
Math (AsciiMath and Tex)               | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Discussions                            | 0.31
**Reproducible content**               |
Number input (range slider)            | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Select input (name value pairs)        | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
Tabular data input                     | 0.30
Code cell                              | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Output (value display)                 | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
**Embedded functions**                                   |
Statistics (`sum`, `mean`, `variance`, ...)              |
Data manipulation (`filter`, `sort`, `aggregate`, ...)   | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
Data visualization (`plot`, `title`, `theme`, ...)       | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
Contribute more...                                       | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
**Execution contexts**                 |
Bash                                   |
JavaScript                             | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Julia                                  |
Jupyter kernels                        | 0.29
Node.js                                | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
Python                                 | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
R                                      | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
SQLite                                 | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
**Supported formats**                  |
HTML                                   | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
JATS                                   | 0.30
Markdown `.md`                         | ![beta](https://img.shields.io/badge/status-beta-yellow.svg)
RMarkdown `.Rmd`                       | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
Jupyter Notebook `.ipynb`              | ![alpha](https://img.shields.io/badge/status-alpha-red.svg)
Microsoft Office `.docx`               |
Open/Libre Office `.odt`               |

### Download

See the [Getting Started](https://github.com/stencila/stencila/wiki/Getting-started) page on the wiki.

Application or package                                                                                          | Ready
:-------------------------------------------------------------------------------------------------------------- | :------------:
[Stencila Desktop](https://github.com/stencila/desktop/releases)                                                | ✓
[Stencila for Python](https://github.com/stencila/py#readme)                                                    | ✓
[Stencila for R](https://github.com/stencila/r#readme)                                                          | ✓
[Stencila for Node.js](https://github.com/stencila/node#readme)                                                 | ✓

### Develop

[![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila)
[![Build status](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila)
[![Code coverage](https://codecov.io/gh/stencila/stencila/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/stencila)
[![Dependency status](https://david-dm.org/stencila/stencila.svg)](https://david-dm.org/stencila/stencila)

Quick start:

```bash
git clone https://github.com/stencila/stencila.git
cd stencila
npm install
npm run start
```

Now you can access the examples in the browser at [http://localhost:4000/](http://localhost:4000/).


Use external contexts during development:

Run the docker image first.

```bash
docker run -p 2100:2000 stencila/alpha
```

Now start the development environment and point `STENCILA_PEERS` to the new host.

```bash
STENCILA_PEERS=http://localhost:2100 npm start
```

Most development tasks can be run  via `npm` or `make` shortcuts:

Task                                                    |`npm`                  | `make`          |
------------------------------------------------------- |-----------------------|-----------------|    
Install and setup dependencies                          | `npm install`         | `make setup`
Run the development server                              | `npm start`           | `make run`
Check code for lint                                     | `npm run lint`        | `make lint`
Run tests                                               | `npm test`            | `make test`
Run tests in the browser                                | `npm run test-browser`| `make test-browser`
Run tests with coverage                                 | `npm run cover`       | `make cover`
Build bundles                                           | `npm build`           | `make build`
Build documentation                                     | `npm run docs`        | `make docs`
Run documentation [server](http://localhost:4001/)      | `npm run docs-serve`  | `make docs-serve`
Clean                                                   |                       | `make clean`

To contribute, [get in touch](https://gitter.im/stencila/stencila), checkout the [platform-wide, cross-repository kanban board](https://github.com/orgs/stencila/projects/1), or just send us a pull request! Please read our contributor [code of conduct](CONDUCT.md).

API documentation is at http://stencila.github.io/stencila/. These are published using Github Pages, so to update them after making changes: run `make docs`, commit the updates and do a `git push`.

Builds done on [Travis CI](https://travis-ci.org/stencila/stencila) are archived at http://builds.stenci.la/stencila/. That site can be useful for user acceptance testing without the need to download Stencila Desktop. Provide test users with a link to an work-in-progress user interface e.g http://builds.stenci.la/stencila/test-deploy-2017-08-13-54a67a6/examples/document/index.html?documentId=01-welcome-to-stencila.
