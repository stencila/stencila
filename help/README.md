# Help

**Stencila help and documentation**

## ✨ Introduction

This folder contains content and code used to build https://help.stenci.la.

We broadly follow the [Diátaxis](https://diataxis.fr/) framework for structuring (and guiding) documentation. The `docs` folder is divided into four subfolders, three of which represent quadrants of the [Diátaxis](https://diataxis.fr/) system (`tutorials`, `guides`, `reference`), as well as `demos` for demonstration screencasts.

1. [Tutorials](https://diataxis.fr/tutorials/) are _lessons_ that take the reader by the hand through a series of steps to complete a project of some kind. Tutorials are **learning-oriented**. They are aimed at users who are new to Stencila.

2. [Guides](https://diataxis.fr/how-to-guides/) are _directions_ or _recipes_ that take the reader through the steps required to solve a real-world problem or use-case. Guides are **goal-oriented**. They are more advanced than tutorials and assume some knowledge of how Stencila works.

3. [References](https://diataxis.fr/reference/) are _technical descriptions_ of the machinery and how to operate it. References are **information-oriented**.

4. Demos are audio / visual demonstrations of functionality and user experience. Demos are primarily aimed at engaging and enthusing users.

## 🛠️ Development

This website is built using [Docusaurus 2](https://docusaurus.io/), a modern static website generator.

### Getting started

```console
npm install
npm start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

### References

Much of the content in the references section [docs/references/](docs/references) is obtained from other repos. To fetch and unzip those run,

```console
make resources
```

### Demos

To generate the demos in [docs/demos/cli](docs/demos/cli) you need the following prerequisites installed:

- [Asciinema](https://asciinema.org/docs/installation)
- [`pv` Pipe Viewer](http://www.ivarch.com/programs/pv.shtml)

Then run,

```console
make demos
```

## 🏗️ Build

To build the entire site,

```console
make resources demos build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## 🚀 Deployment

```console
GIT_USER=<Your GitHub username> USE_SSH=true npm deploy
```

If you are using GitHub pages for hosting, this command is a convenient way to build the website and push to the `gh-pages` branch.
