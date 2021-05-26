# Viewer

## 👀 Introduction

This is a viewer for Stencila documents. It is not an editor, it is optimized for reading. It does however support readers interacting with documents, for example running executable documents, or changing document parameters and seeing how dependent output change etc.

The `viewer` is also optimized for changing documents, for example documents that are being actively edited by humans or whose contents is being updated by an algorithm. It does this by only rendering the parts that have changed.

## 🕓 Status and roadmap

The `viewer` is early development and initially it will be only deployed in the Stencila Desktop (embedded in the document preview pane) and Stencila CLI (embedded for use whe previewing a document using `stencila open ..`). It is envisioned that it will eventually replace the HTML codec in Encoda for generating HTML.

Currently the `viewer` is built using [Solid](https://solidjs.com/) a framework for reactive, declarative user interfaces. Solid has a several characteristics that make it well suited for this purpose including having fined grained reactivity, being [very fast](https://krausest.github.io/js-framework-benchmark/2021/table_chrome_90.0.4430.72.html) and having a [similar syntax to React](https://dev.to/alexmercedcoder/solidjs-react-meets-svelte-4fmm).

Currently, the `DocumentFetcher` simply fetches a JSON document and renders it in the browser. It does not allow for changes to the document to be made incrementally.

Eventually `DocumentSubscriber` should be able to subscribe to two types of document events:

- `changed`: When a description of how the document changed is available e.g. as a [Prosemirror transaction](https://prosemirror.net/docs/guide/#state.transactions) or a [Automerge change](https://github.com/automerge/automerge/blob/main/INTERNALS.md#change-structure-and-operation-types). Should use Solid's [`setState(changes)`](https://github.com/solidjs/solid/blob/main/documentation/state.md#setstatechanges).

- `refreshed`: When the document has changed but only the entire JSON is available. Should use Solid's [`setState(reconcile(state))`](https://github.com/solidjs/solid/blob/main/documentation/state.md#reconcilevalue-options) to do a deep diff of the document state and update as few DOM elements as possible

The `ThemeSwitcher` allows for switching of document themes.

## 🛠️ Develop

### Getting started

```sh
git clone git@github.com:stencila/stencila
cd stencila/viewer
npm install
npm start
```

That should give you a (mostly) blank page. To view a document, you need something to serve it...

### Serving documents

The `viewer` needs to be able to get documents to be able to render them in the browser. You can serve those documents using the `stencila` CLI tool (or equivalently `cargo run` if in development). In the top level of this repo run,

```sh
stencila serve --insecure
```

The `--insecure` flag is necessary to turn off the requirement to pass authorization tokens to the server.

Alternatively you could use a specialized static file server e.g. the `serve` NPM package:

```sh
npx serve -l 9000 -C
```

Then you will be able to open fixtures using the `url` query parameter e.g.

http://localhost:3000/?url=http://127.0.0.1:9000/fixtures/articles/elife-small.json&theme=wilmore
