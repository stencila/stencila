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

### Testing against a server

To test this package against a Stencila document server, [install necessary Rust build tools](https://rustup.rs/) and run the CLI's `serve` command at the top level of this repo,

```sh
cargo run serve --debug
```

Then visit the login URL that is printed in the console. This will set a cookie in your browser that will authorize subsequent requests, including a WebSocket handshake.

You can test that the authorization and WebSocket handshake were successful in the browser console (check the WebSocket messages sent/received in the `Network` tab of your browser developer tools),

```js
var ws = new WebSocket("ws://127.0.0.1:9000/~ws")
ws.send(JSON.stringify({
    method: "documents:start",
    params: {"id": ".."}
}))
```

You can also open documents in this repository using their relative paths. For example, this file is available at http://127.0.0.1:9000/web/README.md; or you might want to develop against a document with this fixture which has some web components in it: http://127.0.0.1:9000/fixtures/articles/code.md.

If you are also developing the Rust server, you might want to use `cargo watch` for automatic recompiling and running of that code. In that case, you should provide the `--key` option so that you do not need to re-login on each reload,

```sh
cargo watch -x "run serve --debug --key my-temporary-key"
```

Alternatively, if you don't want to install Rust tooling, you can [install the pre-built CLI](https://github.com/stencila/stencila/tree/master/cli#-install) binary and run it directly,

```sh
stencila serve --debug
```

Regardless of the method used, all of the above default to listening on `http://127.0.0.1:9000` and `ws://127.0.0.1:9000` with JSON Web Token based authorization. To turn off authorization (if you don't want to have to worry about logging in, keys, etc) use the `--insecure` flag. See `stencila serve --help` for more options, including changing port numbers and alternative log levels.

### Serving built JavaScript

The Rust document server can serve static assets such as JavaScript and CSS from the [`../rust/static`](../rust/static) folder. During development, these assets are served "live" from disk. When the binary is built the assets are embedded in it and served directly from there. This is faster than fetching from a CDN and allows for offline use.

During development, you can either build assets into the `../rust/static` folder or create a symlink from it to a build sub-folder in this folder. This has the advantage over running a separate development server of more closely mimicking the production setup, including serving from the same domain.
