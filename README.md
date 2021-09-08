<div align="center">
	<img src="https://stenci.la/img/stencila/stencilaLogo.svg" alt="Stencila" style="max-width:300px">
</div>
<br>

## 👋 Welcome

This is the main repository of [Stencila](https://stenci.la), a platform for authoring, collaborating on, and publishing executable documents.

Stencila is comprised of several open source packages, written in a variety of programming languages. This repo acts as an entry point to these other packages as well as hosting code for our desktop and CLI tools.

We 💕 contributions! All types of contributions: ideas 🤔, examples 💡, bug reports 🐛, documentation 📖, code 💻, questions 💬. If you are unsure of where to make a contribution feel free to open a new [issue](https://github.com/stencila/stencila/issues/new) or [discussion](https://github.com/stencila/stencila/discussions/new) in this repository (we can always move them elsewhere if need be).

<br>

## 📜 Help

For documentation, including demos and reference guides, please go to our Help site https://help.stenci.la/. That site is developed in the [`help`](help#readme) folder of this repository and contributions are always welcome.

<br>

## 🎁 Hub

If you don't want to install anything, or just want to try out Stencila, https://hub.stenci.la is the best place to start. It's a web application that makes all our software available via intuitive browser-based interfaces. You can contribute to Stencila Hub at [`stencila/hub`](https://github.com/stencila/hub).

<br>

## 🖥️ Desktop

If you'd prefer to use Stencila on your own computer, the Stencila Desktop is a great place to start. It is still in the early stages of (re)development but please see the [`desktop`](desktop#readme) folder for its current status and how you can help out!

<br>

## ⌨️ Command line tool

Prefer to work on the command line? The `stencila` command line tool (CLI) is for you! Please see the [`cli`](cli#readme) folder for installation and usage instructions.

<br>

## 🔌 Plugins

The `stencila` Hub, Desktop and CLI all rely on _plugins_ to provide much of their functionality. You can install plugins using the `stencila` Desktop or CLI tool using it's name or an alias,

```sh
stencila plugins install <name or alias>
```

The following table lists the main plugins. These plugins are in various stages of development and not all of them are compatible with the Desktop and CLI. Generally, it won't be worth installing them prior to `v1` and coverage of at least 90%.

> 🚨 We are the process of deprecating the "executor" plugins `rasta`, `pyla` and `jesta` and instead focussing on a tighter integration with Jupyter kernels by way of porting the functionality in `jupita` into the core Rust library.

| Plugin   | Aliases      | Version     | Coverage    | Primary functionality                                    |
| -------- | ------------ | ----------- | ----------- | -------------------------------------------------------- |
| [encoda] | `converter`  | ![encoda-v] | ![encoda-c] | Convert documents between file formats                   |
| [jesta]  | `javascript` | ![jesta-v]  | ![jesta-c]  | Compile, build and execute documents that use JavaScript |
| [rasta]  | `r`          | ![rasta-v]  | ![rasta-c]  | Compile, build and execute documents that use R          |
| [pyla]   | `python`     | ![pyla-v]   | ![pyla-c]   | Compile, build and execute documents that use Python     |
| [jupita] | `jupyter`    | ![jupita-v] | ![jupita-c] | Execute documents that use Jupyter kernels               |
| [dockta] | `docker`     | ![dockta-v] | ![dockta-c] | Build Docker images for executable documents             |
| [nixta]  | `nix`        | ![nixta-v]  | ![nixta-c]  | Build Nix environments for executable documents          |

<br>

## 🐳 Docker images

You can use Stencila as a Docker image. We provide several images of varying sizes and capabilities. All include the `stencila` CLI as the image `ENTRYPOINT` but add varying numbers of plugins and packages.

At present the number of images listed below is limited. We plan to move the generic images e.g. [`stencila/executa-midi`](https://hub.docker.com/r/stencila/executa-midi) (which are currently built in the `dockta` repository), to this repository as we reach plugin compatibility for the relevant language packages.

| Image               | Size                   | Description                          |
| ------------------- | ---------------------- | ------------------------------------ |
| [stencila/stencila] | ![stencila-stencila-s] | Base image containing `stencila` CLI |
| [stencila/node]     | ![stencila-node-s]     | Adds Node.js and `jesta`             |

<br>

## 👩‍💻 Language bindings

If you prefer, you can use Stencila from within your favorite programming language. Some of these language bindings are in an early, proof-of-concept state and are likely to be developed further only based on demand. If your favorite language is missing, or you would like to help us develop the bindings, [let us know!](https://github.com/stencila/stencila/discussions/new)

| Language | Bindings                | Status                            |
| -------- | ----------------------- | --------------------------------- |
| Node     | [node](node#readme)     | In-development (used for Desktop) |
| Python   | [python](python#readme) | Experimental                      |
| R        | [r](r#readme)           | Experimental                      |

[encoda]: https://github.com/stencila/encoda#readme
[jesta]: https://github.com/stencila/jesta#readme
[pyla]: https://github.com/stencila/pyla#readme
[rasta]: https://github.com/stencila/rasta#readme
[jupita]: https://github.com/stencila/jupita#readme
[dockta]: https://github.com/stencila/dockta#readme
[nixta]: https://github.com/stencila/nixta#readme
[encoda-v]: https://img.shields.io/github/v/release/stencila/encoda
[jesta-v]: https://img.shields.io/github/v/release/stencila/jesta
[rasta-v]: https://img.shields.io/github/v/release/stencila/rasta
[pyla-v]: https://img.shields.io/github/v/release/stencila/pyla
[dockta-v]: https://img.shields.io/github/v/release/stencila/dockta
[nixta-v]: https://img.shields.io/github/v/release/stencila/nixta
[jupita-v]: https://img.shields.io/github/v/release/stencila/jupita
[encoda-c]: https://img.shields.io/codecov/c/github/stencila/encoda
[jesta-c]: https://img.shields.io/codecov/c/github/stencila/jesta
[rasta-c]: https://img.shields.io/codecov/c/github/stencila/rasta
[pyla-c]: https://img.shields.io/codecov/c/github/stencila/pyla
[dockta-c]: https://img.shields.io/codecov/c/github/stencila/dockta
[nixta-c]: https://img.shields.io/codecov/c/github/stencila/nixta
[jupita-c]: https://img.shields.io/codecov/c/github/stencila/jupita
[stencila/stencila]: https://hub.docker.com/r/stencila/stencila
[stencila/node]: https://hub.docker.com/r/stencila/node
[stencila-stencila-s]: https://img.shields.io/docker/image-size/stencila/stencila?label=size&sort=semver
[stencila-node-s]: https://img.shields.io/docker/image-size/stencila/node?label=size&sort=semver
