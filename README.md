<div align="center">
	<img src="docs/images/stencila.png" alt="Stencila" width=300>
</div>
<br>

## 👋 Introduction

Stencila is a platform for collaborating on, and publishing, dynamic, data-driven content. Our aim is to lower the barriers for creating programmable documents and make it easier to create beautiful, interactive, and semantically rich, articles, web pages and applications from them.

## 📈 Status

This is `v2` of Stencila, a rewrite in Rust aimed at leveraging two relatively recent and impactful innovations:

- [Conflict-free replicated data types (CRDTs)](https://crdt.tech/), and specifically the production ready, Rust-based [Automerge](https://github.com/automerge/automerge), for de-centralized collaboration and version control.

- [Large language models (LLMs)](https://en.wikipedia.org/wiki/Large_language_model) for assisting in writing and editing, prose and code.

We are embarking on a rewrite because CRDTs will now be the foundational synchronization and storage layer for Stencila documents. This requires fundamental changes to most other parts of the platform (e.g. how changes are applied to dynamic documents). Furthermore, a rewrite allow us to bake in, rather than bolt on, mechanisms to mitigate the risks associated with using LLM assistants for authoring documents (e.g. recording the actor, human or LLM, that made the change to a document).

Having said that, much of the code in the [`v1` branch](https://github.com/stencila/stencila/tree/v1) will be reused (probably after some tidy-ups and refactoring), so `v2` is not a _complete_ rewrite.

## 📥 Install

Although `v2` is in early stages of development, and functionality may be limited or buggy, we are taking a "release early, release often" approach and releasing binary builds of alpha versions of the Stencila CLI tool and language packages. Doing so allows us to get early feedback and monitor what impact the addition of features has on build times and distribution sizes.

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

## ⚡ Usage

See `stencila --help` or the reference documentation for the CLI [here](docs/reference/cli.md).

## 🚴 Roadmap

We'll be releasing early and often across all products: initial versions will have limited functionality and be buggy but will establish a deployment pipeline that can be rapidly iterated on.

We're aiming for a `v2` release at the end of Q3 2024. Development priorities for later quarters can't be know until user testing of initial version has been done.

### Q3 2023: First alpha releases

Focus on re-establishing continuous deployments of CLI, Python and Node packages, and JSON Schema and JSON-LD contexts.

Feature additions:

- [ ] Code editor interface, based on [CodeMirror](https://codemirror.net/) and served by CLI, with live sync between alternative formats in editor and on disk. Doing this early ensures the basic architecture is in place for format-agnostic reactivity of documents.

- [ ] HTML codec for live preview in page served by CLI

- [ ] Markdown codec based on [`v1`](https://github.com/stencila/stencila/tree/v1/rust/codec-md) implementation.

- [ ] JATS codec on par with [`encoda`](https://github.com/stencila/encoda)'s implementation and using its test suite.

Release first `v2.0.0-alpha.x` versions of:

- [x] CLI
- [ ] Python package
- [ ] Node package
- [ ] JSON Schema & JSON-LD `@context`

### Q4 2023: Final alpha release

Focus on an initial deployment of desktop application.

Feature additions:

- [ ] Desktop application built with [Tauri](https://tauri.app/), bundling web interface and the core functionality in Rust crates

Release last `v2.0.0-alpha.x` versions of:

- [ ] Desktop
- [ ] CLI
- [ ] Python package
- [ ] Node package
- [ ] JSON Schema & JSON-LD `@context`


### Q1 2024: First beta release

User testing of `v2.0.0-alpha.x`. Development priorities to be determined based on user feedback.

Release `v2.0.0-beta.1` versions of:

- [ ] Desktop
- [ ] CLI
- [ ] Python package
- [ ] Node package
- [ ] JSON Schema & JSON-LD `@context`


### Q2 2024: Second beta release

User testing of `v2.0.0-beta.1`. Development priorities to be determined based on user feedback.

Release `v2.0.0-beta.2` versions of:

- [ ] Desktop
- [ ] CLI
- [ ] Python package
- [ ] Node package
- [ ] JSON Schema & JSON-LD `@context`

### Q3 2024: Version 2.0.0

User testing of `v2.0.0-beta.2`. Development priorities to be determined based on user feedback.

Release `v2.0.0` versions of:

- [ ] Desktop
- [ ] CLI
- [ ] Python package
- [ ] Node package
- [ ] JSON Schema & JSON-LD `@context`


## 🛠️ Develop

This repository is organized into the following modules. Please see their respective READMEs for guides to contributing.

- [`schema`](schema): YAML files which define the Stencila Schema, an implementation of, and extensions to, [schema.org](https://schema.org), for programmable documents.

- [`json`](json) [`🏗️ In progress`]: A [JSON Schema](https://json-schema.org/) and [JSON LD](https://json-ld.org/) `@context`, generated from Stencila Schema, which can be used to validate Stencila documents and transform them to other vocabularies

- [`rust`](rust): Several Rust crates implementing core functionality and a CLI for working with Stencila documents.

- [`python`](python) [`🏗️ In progress`](https://github.com/stencila/stencila/issues/1624): A Python package, with [Pydantic](https://docs.pydantic.dev/latest/) classes generated from Stencila Schema and bindings to Rust functions, so you can work with Stencila documents from within Python.

- [`typescript`](typescript) [`🏗️ In progress`](https://github.com/stencila/stencila/issues/1625): A package of TypeScript types generated from Stencila Schema so you can create type-safe Stencila documents in the browser, Node.js, Deno etc.

- `node` [`🧭 Planned`](https://github.com/stencila/stencila/issues/1626): A Node.js package, using the generated TypeScript types and with runtime validation and bindings to Rust functions, so you can work with Stencila documents from within Node.js.

- [`docs`](docs) `🏗️ In progress`: Documentation, including reference documentation generated from `schema` and the `rust` CLI tool.

- [`examples`](examples) `🏗️ In progress`: Example of documents conforming to Stencila Schema, mostly for testing purposes.


## 💖 Supporters

We wouldn’t be doing this without the support of these generous, forward looking organizations.

<p align="center"><a href="https://sloan.org/"><img src="docs/images/sloan.png" height="70"></img></a><p>
<p align="center"><a href="https://elifesciences.org/"><img src="docs/images/elife.svg" height="70"></img></a><p>
<p align="center"><a href="https://www.mbie.govt.nz"><img src="docs/images/mbie.jpeg" height="70"></img></a><p>
<p align="center"><a href="https://coko.foundation/"><img src="docs/images/coko.png" height="70"></img></a><p>
<p align="center"><a href="https://www.codeforsociety.org/"><img src="docs/images/css.png" height="70"></img></a><p>
<p align="center"><a href="https://www.callaghaninnovation.govt.nz/"><img src="docs/images/callaghan.png" height="70"></img></a><p>

## 🙏 Acknowledgements

Stencila is built on the shoulders of many open source projects. Our sincere thanks to all the maintainers and contributors of those projects for their vision, enthusiasm and dedication, but most of all for all their hard work! The following open source projects in particular have an important role in the current version of Stencila.

|                                                  | Link                                  | Summary                                                                                                                                 |
| ------------------------------------------------ | ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| <img src="docs/images/automerge.png" width="80"> | [Automerge](https://automerge.org/)   | A Rust library of data structures for building collaborative applications.                                                              |
| <img src="docs/images/clap.png" width="80">      | [Clap](https://crates.io/crates/clap) | A Command Line Argument Parser for Rust.                                                                                                |
| <img src="docs/images/rust.png" width="80">      | [Rust](https://www.rust-lang.org/)    | A multi-paradigm, high-level, general-purpose programming language which emphasizes performance, type safety, and concurrency.          |
| <img src="docs/images/ferris.png" width="80">    | [Serde](https://serde.rs/)            | A framework for **ser**ializing and **de**serializing Rust data structures efficiently and generically.                                 |
| <img src="docs/images/similar.png" width="80">   | [Similar](https://insta.rs/similar/)  | A Rust library of diffing algorithms including Patience and Hunt–McIlroy / Hunt–Szymanski LCS.                                          |
| <img src="docs/images/tokio.png" width="80">     | [Tokio](https://tokio.rs/)            | An asynchronous runtime for Rust which provides the building blocks needed for writing network applications without compromising speed. |
