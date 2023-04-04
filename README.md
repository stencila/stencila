<div align="center">
	<img src="https://stenci.la/img/stencila/stencilaLogo.svg" alt="Stencila" style="max-width:300px">
</div>
<br>

## 👋 Introduction

Stencila is a platform for authoring, sharing and publishing dynamic, data-driven documents. Our aim is to lower the barriers for creating and collaborating on data-driven documents and make it easier to create beautiful, interactive, and semantically rich, articles, web pages and applications from them.

## 📈 Status

This is `v2` of Stencila, a rewrite in Rust aimed at leveraging two relatively recent and impactful innovations:

- [Conflict-free replicated data types (CRDTs)](https://crdt.tech/), and specifically the production ready, Rust-based [Automerge `v2`](https://github.com/automerge/automerge), for de-centralized collaboration and version control.

- [Generative pre-trained transformers (GPTs)](https://en.wikipedia.org/wiki/Generative_pre-trained_transformer), and large language models (LLMs) in general, for enhancing the authoring and coding productivity.

We are embarking on a rewrite because CRDTs will now be the foundational synchronization and storage layer for Stencila documents which necessitates fundamental changes to other parts of the platform. Furthermore, a rewrite will allow us to leverage CRDTs to bake in mechanisms to mitigate the risks associated with using LLM assistants for authoring documents.

We're in the early stages of this rewrite, and this document will be updated with a roadmap and other details soon.

We are currently tagging releases using a `2.0.0-alpha.X` pattern (where we increment X on each release). The `v1` branch can be browsed [here](https://github.com/stencila/stencila/tree/v1).

## 📥 Install

Although `v2` is in early stages of development, and functionality may be limited or buggy, we are taking a continuous delivery approach and releasing binary builds of alpha versions of the Stencila CLI tool and language packages. Doing so allows us to get early feedback and monitor what impact the addition of features has on build times and distribution sizes.

### CLI tool

#### Windows

To install the latest release download `stencila-<version>-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/stencila/stencila/releases/latest) and place it somewhere on your `PATH`.

#### MacOS

To install the latest release in `/usr/local/bin`,

```console
curl -L https://raw.githubusercontent.com/stencila/stencila/main/install.sh | bash
```

To install a specific version, append `-s vX.X.X`. Or, if you'd prefer to do it manually, download `stencila-<version>-x86_64-apple-darwin.tar.xz` from the one of the [releases](https://github.com/stencila/stencila/releases) and then,

```console
tar xvf stencila-*.tar.xz
sudo mv -f stencila /usr/local/bin # or wherever you prefer
```

#### Linux

To install the latest release in `~/.local/bin/`,

```console
curl -L https://raw.githubusercontent.com/stencila/stencila/main/install.sh | bash
```

To install a specific version, append `-s vX.X.X`. Or, if you'd prefer to do it manually, download `stencila-<version>-x86_64-unknown-linux-gnu.tar.xz` from the one of the [releases](https://github.com/stencila/stencila/releases) and then,

```console
tar xvf stencila-*.tar.xz
mv -f stencila ~/.local/bin/ # or wherever you prefer
```

#### Docker

The CLI is also available in a Docker image you can pull from the Github Container Registry,

```console
docker pull ghcr.io/stencila/stencila
```

and use locally like this for example,

```console
docker run -it --rm -v "$PWD":/work -w /work --network host ghcr.io/stencila/stencila --help
```

## 🛠️ Develop

This repository is organized into the following modules. Please see their respective READMEs for guides to contributing.

- `schema`: YAML files which define the Stencila Schema for dynamic documents.

- `json-ld` `🏗️ In progress`: A [JSON LD](https://json-ld.org/) `@context` for Stencila Schema generated from the files in `schema`

- `json-schema` `🏗️ In progress`: A [JSON Schema](https://json-schema.org/) for Stencila Schema generated from the files in `schema`

- `rust`: Several Rust crates implementing core functionality including generating language bindings for the schema (including for Rust itself), and for working with documents that adhere to that schema.

- `docs` `🏗️ In progress`: Documentation, including reference documentation generated from `schema` and the `rust` CLI interface.

- `node` `🧭 Planned`: A Node.js package built on top of the Rust crates which provides interfaces to use Stencila from within Node.js.

- `python` `🧭 Planned`: A Python package built on top of the Rust crates which provides interfaces to use Stencila from within Python.

- `r` `🧭 Planned`: An R package built on top of the Rust crates which provides interfaces to use Stencila from within R.
