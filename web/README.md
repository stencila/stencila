# 🌍 Stencila Web Client

**Use Stencila from a web browser**

## 👋 Introduction

This is the Stencila web client, a small TypeScript library for interacting with a Stencila document server. This client is used, in conjunction with Stencila [web components](https://github.com/stencila/designa#-designa), to enable readers of published executable documents to be able to start a document session and interact with it. It is also used within the [desktop](https://github.com/stencila/stencila/tree/master/desktop#readme) and [CLI](https://github.com/stencila/stencila/tree/master/clie#readme) clients for previewing executable documents locally.

## 🛠️ Develop

### Getting started

Get started by cloning this repository and installing the necessary dependencies

```sh
git clone git@github.com:stencila/stencila
cd stencila/web
npm install
```

### Building JavaScript

During development, re-build the `dist` folder on source code changes using,

```
npm run dev
```

### Running a document server

To test this package against a Stencila document server, [install necessary Rust build tools](https://rustup.rs/) and run the CLI's `serve` command at the _top level_ of this repo,

```sh
cargo run --bin stencila -- server start --debug
```

Then visit the login URL that is printed in the console. This will set a cookie in your browser that will authorize subsequent requests, including WebSocket handshakes. You can then open documents in this repository using their relative paths. For example, this file is available at http://127.0.0.1:9000/web/README.md; or you might want to develop against a document with this fixture which has some web components in it: http://127.0.0.1:9000/fixtures/articles/code.md.

If you are also developing the Rust server, you might want to use `cargo watch` for automatic recompiling and running of that code. In that case, you should provide the `--key` option so that you do not need to re-login on each reload,

```sh
cargo watch --ignore web -x "run --bin stencila -- server start --debug --key my-temporary-key"
```

Alternatively, if you don't want to install Rust tooling, you can [install the pre-built CLI](https://github.com/stencila/stencila/tree/master/cli#-install) binary and run it directly,

```sh
stencila server start --debug
```

Regardless of the method used, all of the above default to listening on `http://127.0.0.1:9000` and `ws://127.0.0.1:9000` with JSON Web Token based authorization.

> 💁 To turn off authorization (if you don't want to have to worry about logging in, keys, etc) use the `--insecure` flag. See `stencila server --help` for more options, including changing port numbers and alternative log levels.

The Rust document server serves static assets such as JavaScript and CSS from the [`../rust/static`](../rust/static) folder. During development, these assets are served from disk. When the binary is built the assets are embedded in it and served directly from there. This is faster than fetching from a CDN and allows for offline use.

To enable serving JavaScript developed in this package, the `../rust/static` folder has a symlink, named `web` which points to the `dist` subfolder in this folder. Anything in that folder is available at the `/~static/web` and the `/~static/web/index.js` file is included in the `<head>` of the served pages.

### Running tests

Once you have a document server running you can run the end-to-end tests in this package using,

```sh
npm test
# or
npm run test:watch
```

> 📢 Currently the tests do not have a mechanism authenticating using the token so you'll have to run the server with the `---insecure` option. A PR to fix this (using the Jest `beforeAll` hook?) would be welcomed!

### Production build

Bundles are compressed using Brotli. Given that [>96% of browsers](https://caniuse.com/brotli) can use Brotli, we do not use Gzip compression since that would needlessly increase the size of the files embedded in Stencila binaries.
