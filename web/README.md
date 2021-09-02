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
npm run watch
```

### Testing against a server

To test this package against a Stencila document server, [install necessary Rust build tools](https://rustup.rs/) and run the CLI's `serve` command at the top level of this repo,

```sh
cargo run serve --debug
```

Then visit the login URL that is printed in the console. This will set a cookie in your browser that will authorize subsequent requests, including a WebSocket handshake.

You can test that the authorization and WebSocket handshake were successful in the browser console (check the WebSocket messages sent/received in the `Network` tab of your browser developer tools),

```js
var ws = window.stencilaWebClient('ws://127.0.0.1:9000/~ws')
ws.on('open', async () => {
  const { id: sessionId } = await ws.call(
    "sessions.start",
    {"project": "pr.b4sa95hsg.."}
  )
})
```

You can also open documents in this repository using their relative paths. For example, this file is available at http://127.0.0.1:9000/web/README.md; or you might want to develop against a document with this fixture which has some web components in it: http://127.0.0.1:9000/fixtures/articles/code.md.

If you are also developing the Rust server, you might want to use `cargo watch` for automatic recompiling and running of that code. In that case, you should provide the `--key` option so that you do not need to re-login on each reload,

```sh
cargo watch --ignore web -x "run serve --debug --key my-temporary-key"
```

Alternatively, if you don't want to install Rust tooling, you can [install the pre-built CLI](https://github.com/stencila/stencila/tree/master/cli#-install) binary and run it directly,

```sh
stencila serve --debug
```

Regardless of the method used, all of the above default to listening on `http://127.0.0.1:9000` and `ws://127.0.0.1:9000` with JSON Web Token based authorization. To turn off authorization (if you don't want to have to worry about logging in, keys, etc) use the `--insecure` flag. See `stencila serve --help` for more options, including changing port numbers and alternative log levels.

### Serving built JavaScript

The Rust document server can serve static assets such as JavaScript and CSS from the [`../rust/static`](../rust/static) folder. During development, these assets are served from disk. When the binary is built the assets are embedded in it and served directly from there. This is faster than fetching from a CDN and allows for offline use.

To enable this, the `../rust/static` folder has a symlink, named `web` which points to the `dist` subfolder in this folder. Anything in that folder is available at the `/~static/web` and the `/~static/web/index.js` file is included in the `<head>` of the served pages.
