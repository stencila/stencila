# Stencila Web

**Components, clients, and views for using Stencila from a web browser**

- [Stencila Web](#stencila-web)
  - [🤝 Clients](#-clients)
    - [WebSocket subprotocols](#websocket-subprotocols)
    - [Client classes](#client-classes)
  - [🪟 Views](#-views)
  - [💅 Twind usage](#-twind-usage)
    - [1. extend `TWLitElement`](#1-extend-twlitelement)
    - [2. Long class lists](#2-long-class-lists)
    - [3. Custom CSS rules](#3-custom-css-rules)
    - [4. Rendering functions](#4-rendering-functions)
  - [🎨 Themes](#-themes)
  - [🛠️ Develop](#️-develop)
    - [Getting started](#getting-started)
    - [Workspace dependencies](#workspace-dependencies)
    - [Linting \&\& formatting](#linting--formatting)
    - [Testing](#testing)
    - [`Makefile`](#makefile)

## 🤝 Clients

This module has several classes of clients which provide different types of access to documents.

Some clients are read-only: they can not send any changes to the document, only receive them. Other clients are read-write but will only send changes related to certain node types. Having different JavaScript classes for clients with different capabilities adds an additional layer of security because only minimum the necessary code is running in the browser.

Each client instance is associated with a single document and communicates with the server over a WebSocket connection at `/~ws/<DOCUMENT_ID>`. The server routes incoming and outgoing messages between the client and the document. A single browser window may have several instances of more than one class of client.

### WebSocket subprotocols

> [!WARNING]
> These WebSocket subprotocols are in draft and likely to change.

The access level of the client correspond to the [WebSocket subprotocol](https://http.dev/ws#sec-websocket-protocol) it uses to communicate with the server. The following Stencila subprotocols are currently defined:

| Subprotocol[^1]  | Capabilities                                                                                                                        |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `read.<FORMAT>`  | Read the entire document in the specified format                                                                                    |
| `comment.nodes`  | Create, and update and delete created, `Comment` nodes in the document                                                              |
| `suggest.nodes`  | Create, and update and delete created, `Suggestion` and `Comment` nodes in the document                                             |
| `input.nodes`    | Update the `value` property of `Parameter` nodes in the document                                                                    |
| `code.nodes`     | Update the `code` property of `CodeExecutable` nodes in the document                                                                |
| `edit.nodes`     | Create, update and delete [prose nodes](https://github.com/stencila/stencila/tree/main/docs/reference/schema/prose) in the document |
| `write.nodes`    | Create, update and delete all nodes in the document with the exception of those related to permissions                              |
| `write.<FORMAT>` | Write the entire document in the specified format with the exception of nodes related to permissions                                |
| `admin.nodes`    | Create, update and delete all nodes in the document including those related to permissions                                          |
| `admin.<FORMAT>` | Write the entire document in the specified format including nodes related to permissions                                            |

[^1]: The naming of subprotocols follows the domain name like convention [commonly used](https://www.iana.org/assignments/websocket/websocket.xml#subprotocol-name), e.g `write.nodes.stencila.org`. But for brevity, the `.stencila.org` suffix is omitted in this document.

On a WebSocket upgrade request the server will only allow connections using subprotocols corresponding to the permissions that the user has for the document. For example, one of the `maintainers` of a `CreativeWork` would be permitted to connect using either the `admin.nodes.stencila.org` subprotocol (for a visual editor) or one of the `admin.<FORMAT>.stencila.org` subprotocols (for a code editor), where `<FORMAT>` is one of the document formats supported by Stencila.

| Role[^2]       | Allowed protocols                                 |
| -------------- | ------------------------------------------------- |
| Anon           | `read.<FORMAT>`, `input.nodes`, `code.nodes` [^3] |
| `contributors` | `comment.nodes`, `suggest.nodes`                  |
| `editors`      | + `edit.nodes`                                    |
| `authors`      | + `write.nodes`, `write.<FORMAT>`                 |
| `maintainers`  | + `admin.nodes`, `admin.<FORMAT>`                 |

[^2]: A user has a role if they are a member of one of the following properties of a [`CreativeWork`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative-work.md). A user has the "anon" role if they are not in any of those properties.
[^3]: Which of these modes is available to anonymous users may be restricted.

### Client classes

Several client classes are implemented, each using one or more of the WebSocket subprotocols, and interacting with the browser DOM or JavaScript editors.

| Client class                                      | Subprotocol         | Description                                                                                                                             |
| ------------------------------------------------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| [`Client`](src/clients/client.ts)                 | Any                 | Abstract base class for all client classes; maintains a WebSocket connection to the server including reconnecting after disconnections. |
| [`FormatClient`](src/clients/format.ts)           | `<ACCESS>.<FORMAT>` | Abstract base class for clients of a document (represented in a particular format) which can send and receive `FormatPatch`s.           |
| [`NodesClient`](src/clients/nodes.ts)             | `<ACCESS>.nodes`    | A client which forwards in-browser `stencila-node-patch` events (emitted by Web Components) to the server as `NodePatch`s.              |
| [`DomClient`](src/clients/dom.ts)                 | `read.html`         | Read-only client of a document's HTML which updates the browser DOM when it receive `FormatPatch`s from the server.                     |
| [`CodeMirrorClient`](src/clients/codemirror.ts)   | `<ACCESS>.<FORMAT>` | Read-write client of a document (represented in a particular format) which synchronizes content with a CodeMirror editor.               |
| [`ProseMirrorClient`](src/clients/prosemirror.ts) | `<ACCESS>.nodes`    | Read-write client of a document which synchronizes content with a ProseMirror editor.                                                   |

## 🪟 Views

There are several views in the [`src/views`](src/views) folder which make use of various combinations of clients. Each view is a Web Component custom element and is bundled and served in a separate JavaScript bundles.

| Custom element            | Description                                                                                                                        | Clients, or other views, used                         |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `<stencila-static-view>`  | A static view of a document which does not update as the document changes, nor allows changes to it                                | None                                                  |
| `<stencila-live-view>`    | A live view of a document which updates the browser DOM when the document changes but which does not allow changes to the document | `DomClient`                                           |
| `<stencila-dynamic-view>` | A live view of the document which also allows the user to make changes to the document via Web Components for nodes                | `DomClient`and `NodesClient`                          |
| `<stencila-source-view>`  | A source code editor for a document                                                                                                | `CodeMirrorClient`                                    |
| `<stencila-split-view>`   | A split pane view with a source code editor and a dynamic view                                                                     | `<stencila-split-view>` and `<stencila-dynamic-view>` |
| `<stencila-visual-view>`  | A visual (WYSIWYG) editor for a document including Web Components for nodes                                                        | `ProseMirrorClient` and `DomClient`                   |

In addition there is a `print.ts` file, powered by [Paged.js](https://pagedjs.org/), which provides a preview of how the document will look when saved as PDF.

## 💅 Twind usage

We use [twind](https://twind.style/) so that we can apply [tailwind](https://tailwindcss.com/)
to our components. In order to do so, there are a few simple patterns we should
follow:

### 1. extend `TWLitElement`

This component applies the `@withTailwind()` decorator to the class & correctly
types the protected `tw` class property (used to access `css`, `theme` functions
etc).

**Note:**

The remaining points below are recommendations for _all_ functions of a
component that **render html**.

### 2. Long class lists

If the component (or parts of a component) have long lists of tailwind classes,
it's best to break this up into a single `apply` function to add to the html -
e.g:

```ts
const styles = apply([
  'text-base font-bold',
  'leading-none',
  'select-none',
  'appearance-none ',
  'min-w-fit',
  'py-2 px-4',
  'bg-white',
  'border-b-4 border-b-transparent',
  'transition-all ease-in-out',
  'flex',
  'items-center',
  'group-hover:text-brand-blue group-hover:border-b-brand-blue',
])
```

**Notes:**

- Each entry in the array of classes should roughly correspond to a css area. In
  the example above, the first entry `'text-base font-bold'` corresponds to font
  rendering, whilst in a later entry `'border-b-4 border-b-transparent'` is
  concerned with border rendering.
- Variants **_must_** be at the end of the class list. This includes breakpoints,
  states (hover, active etc.), et al.

### 3. Custom CSS rules

The vast majority of css can be applied directly with the `tw` classes. When
this isn't possible (something missing in tailwind), use the `css` function to
create the desired classes.

```ts
const hideMarker = css`
  &::marker {
    display: none;
    font-size: 0;
  }
`
```

In this case, the `::marker` selector (for use in `details > summary`) isn't
supported in tailwind.

### 4. Rendering functions

**Tip:**

When adding a function to your class that breaks up the render function, you
should create a private function with a prefix of `render` e.g. `renderHeader`,
`renderBody` etc.

Wrapping all of the above, you will have a function that looks similar to the
following:

```ts
private renderSummary() {
  const styles = apply([
    'text-base font-bold',
    'leading-none',
    ...,
    'flex',
    'items-center',
    'group-hover:text-brand-blue group-hover:border-b-brand-blue',
  ])

  const hideMarker = css`
    &::marker {
      display: none;
      font-size: 0;
    }
  `

  return html`<summary
    aria-haspopup="listbox"
    role="button"
    class="${styles} ${hideMarker}"
  >
    ...
  </summary>`
}
```

## 🎨 Themes

The [`src/themes`](src/themes) folder contains builtin themes. Builtin themes (and the fonts and other static assets that they use) are embedded into distributed binaries and can be referred to by name.

| Name    | Description                                                                                                          |
| ------- | -------------------------------------------------------------------------------------------------------------------- |
| Default | The default theme used when none other is specified                                                                  |
| LaTeX   | A LaTeX-like theme based on [LaTeX.css](https://latex.now.sh/)                                                       |
| Tufte   | A theme inspired by Edward Tufte’s books and handouts based on [Tufte CSS](https://edwardtufte.github.io/tufte-css/) |

## 🛠️ Develop

### Getting started

In a console run Parcel (the bundler we use) in watch mode (so that JavaScript modules are hot loaded when their TypeScript source code changes) from this directory:

```console
npm start
```

In a separate console run the Stencila CLI's `serve` command in debug mode using `cargo` from the directory that you want to be the home for the server paths e.g.

```console
cd ../examples/nodes
cargo run --bin stencila -- serve
```

When contributing code please run the following linting, formatting and testing scripts. Linting checks are run on CI, so for faster iteration, fewer failed runs and less noise, it's generally a good idea to run them locally before pushing code.

### Workspace dependencies

This module uses types from `@stencila/types`, a package that is defined in the sibling [`ts`](../ts) directory, and which is also part of the NPM [workspace](https://docs.npmjs.com/cli/v7/using-npm/workspaces) defined in the root [package.json](../package.json).

You may find that ESLint will complain that `@stencila/types` does not exist because [../ts/dist](../ts/dist) does not yet exist. To fix this build that package:

```console
cd ../ts && npm run build
```

or

```console
make -C ../ts build
```

### Linting && formatting

We use [ESLint](https://eslint.org/) and [Prettier](https://prettier.io/) for code linting and formatting respectively. To apply linting and formatting fixes:

```console
npm run fix
```

To just check linting and formatting:

```console
npm run lint
```

### Testing

At present we don't have comprehensive tests for this package ([coming soon!](https://github.com/stencila/stencila/issues/1781)) so, for now, the `npm test` script just checks the code using Typescript.

```console
npm test
```

### `Makefile`

As with most modules in this repo, there is a `Makefile` which you may prefer to use for common development tasks. For example to easily run multiple NPM scripts at once:

```console
make fix test
```

A recommended combination of recipes to run before committing code is:

```console
make audit lint test
```

> [!NOTE]
> The Parcel config currently uses `parcel/transformer-typescript-tsc` because of this [issue](https://github.com/parcel-bundler/parcel/issues/7425) related to decorators.

> [!NOTE]
> There is a `.postcssrc` config file even though PostCSS is not a direct dependency of this package. It is used to integrate Tailwind with Parcel (see https://parceljs.org/languages/css/#postcss) which in turn allows us to use Tailwind's @apply directive and utility classes in `./src/themes/*.css` files.

> [!NOTE]
> Themes are applied to the Shadow DOM of HTML-based document views using [Constructed Stylesheets](https://web.dev/articles/constructable-stylesheets). These do not allow the use of `@import`. Therefore the `postcss-import` plugin is used to inline these statements. This requires some oddities in the file paths used for font file in imported files. See the notes in the CSS file for existing fonts.
