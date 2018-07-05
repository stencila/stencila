<div align="center">
  <a href="https://stenci.la">
    <img src="http://stenci.la/img/logo-name.png" alt="Stencila"
    style="height:80px">
  </a>
</div>


Stencila provides a set of open-source software components enabling reproducible and transparent research within the tool of your choice. Stencila allows you to write reproducible documents containing interactive source code using the interfaces you are most familiar with, such as  [RMarkdown](https://github.com/rstudio/rmarkdown) and [Jupyter Notebooks](http://jupyter.org/) but also MS Word or Excel, Google Docs or
Google Sheets.

This repository contains source code for Stencila GUI (Graphical User Interface) and also guides you through the repositories for other Stencila components.

### Develop

Quick start:

```bash
git clone https://github.com/stencila/stencila.git
cd stencila
npm install
npm run start
```

And navigate to [http://localhost:4000/?archive=kitchen-sink&storage=fs](http://localhost:4000/example.html?archive=kitchen-sink&storage=fs).
You can save your document changes by pressing `CommandOrControl+S`.
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

| Task                                               | `npm`                  | `make`              |
|:---------------------------------------------------|:-----------------------|:--------------------|
| Install and setup dependencies                     | `npm install`          | `make setup`        |
| Run the development server                         | `npm start`            | `make run`          |
| Check code for lint                                | `npm run lint`         | `make lint`         |
| Run tests                                          | `npm test`             | `make test`         |
| Run tests in the browser                           | `npm run test-browser` | `make test-browser` |
| Run tests with coverage                            | `npm run cover`        | `make cover`        |
| Build bundles                                      | `npm build`            | `make build`        |
| Build documentation                                | `npm run docs`         | `make docs`         |
| Run documentation [server](http://localhost:4001/) | `npm run docs-serve`   | `make docs-serve`   |
| Clean                                              |                        | `make clean`        |

### Build

Builds done on [Travis CI](https://travis-ci.org/stencila/stencila) are archived at http://builds.stenci.la/stencila/. That site can be useful for user acceptance testing without requiring users to download Stencila Desktop. Just provide test users with a link to a work-in-progress user interface e.g http://builds.stenci.la/stencila/test-deploy-2017-08-13-54a67a6/examples/document/index.html?documentId=01-welcome-to-stencila.


## Components

| Component repository                                                   |                                               Description                                               |
|:-----------------------------------------------------------------------|:-------------------------------------------------------------------------------------------------------:|
| [Stencila Bindila](https://github.com/stencila/bindila)                |                      Host for running Stencila in [Binder](https://mybinder.org/).                      |
| [Stencila Convert](https://github.com/stencila/convert)                |        Converters for importing and exporting documents in various formats to and from Stencila.        |
| [Stencila CLI](https://github.com/stencila/cli)                        |                                      Command Line Interface tool.                                       |
| [Stencila Cloud](https://github.com/stencila/cloud)                    |                                     A Stencila host for the cloud.                                      |
| [Stencila Desktop](https://github.com/stencila/desktop)                |                                  Desktop application for your machine.                                  |
| [Stencila Engine](https://github.com/stencila/engine)                  |                  Evaluation Engine underpinning the execution and dependency analysis.                  |
| [Stencila Examples](https://github.com/stencila/examples)              |              Examples of Stencla articles and sheets for data analysis and visualization.               |
| [Stencila Hub](https://github.com/stencila/hub)                        |   Interface to Stencila Cloud allowing users to create, share and collaborate on Stencila documents.    |
| [Stencila JS](https://github.com/stencila/js)                          |                                      Stencila JavaScript package.                                       |
| [Stencila Images](https://github.com/stencila/images)                  |     Stencila Docker containers including execution contexts and various packages for data analysis.     |
| [Stencila Node](https://github.com/stencila/node)                      | Stencila Node.js package enabling SQL execution context and enhancing the Javascript execution context. |
| [Stencila Mini](https://github.com/stencila/mini)                      |                 Built-in simple Mini language for basic data manipulation and plotting.                 |
| [nbStencilaHostProxy](https://github.com/stencila/nbstencilahostproxy) |                         Proxy to a Stencila Host from a Jupyter Notebook server                         |
| [Stencila LibDH] (https://github.com/stencila/libdh)                   |                  A library of Stencila-compatible functions for the Digital Humanities                  |
| [Stencila LibTemplate](https://github.com/stencila/libtemplate)        |               A template repository for creating Stencila-compatible function libraries.                |
| [Stencila Libcore](https://github.com/stencila/libcore)                |                     Core function library for the built-in Stencila language Mini.                      |
| [Stencila Py](https://github.com/stencila/py)                          |                     Stencila Python package which enables Python execution context.                     |
| [Stencila R](https://github.com/stencila/r)                            |                          Stencila R package which enables R execution context.                          |
| [Stencila Specs](https://github.com/stencila/specs)                    |                              API and schemas used for Stencila documents.                               |



### Issues

Please report any problems you encounter with Stencila and any of its components:
* by filing in an [issue in this repository](https://github.com/stencila/stencila/issues/new);
* posting on our [Community Forum](https://community.stenci.la);
* chatting to us and other users on our [chat channel](https://gitter.im/stencila/stencila).
