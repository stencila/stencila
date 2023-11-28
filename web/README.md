# Stencila Web

**Components, editors and clients for using Stencila from a web browser**

## 🤝 Clients

This module has several classes of clients that have different capabilities and talk to the Stencila server using different WebSocket subprotocols.

Each client instance is associated with a single document. They communicate with the server over a WebSocket connection at `/~ws/<DOCUMENT_ID>`. The server routes incoming and outgoing messages between the client and the document. A browser window may have instances of more than one class of client.

Some clients are read-only: they can not send any changes to the document, only receive them. Other clients are read-write but will only send changes related to certain node types. Having different JavaScript classes for clients with different capabilities adds an additional layer of security because only minimum the necessary code is running in the browser.

### WebSocket subprotocols

The capabilities of the client correspond to the [WebSocket subprotocol](https://http.dev/ws#sec-websocket-protocol) it uses to communicate with the server. The following Stencila subprotocols are currently defined:

| Subprotocol[^1]  | Capabilities                                                                                                                        |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `read.<FORMAT>`  | Read the entire document in the specified format                                                                                    |
| `comment.nodes`  | Create, and update and delete created, `Comment` nodes in the document                                                              |
| `suggest.nodes`  | Create, and update and delete created, `Suggestion` and `Comment` nodes in the document                                             |
| `input.nodes`    | Update the `value` property of `Parameter` nodes in the document                                                                    |
| `code.nodes`     | Update the `code` property of `CodeExecutable` nodes in the document                                                                |
| `edit.nodes`    | Create, update and delete [prose nodes](https://github.com/stencila/stencila/tree/main/docs/reference/schema/prose) in the document |
| `write.nodes`    | Create, update and delete all nodes in the document with the exception of those related to permissions                              |
| `write.<FORMAT>` | Write the entire document in the specified format with the exception of nodes related to permissions                                |
| `admin.nodes`    | Create, update and delete all nodes in the document including those related to permissions                                          |
| `admin.<FORMAT>` | Write the entire document in the specified format including nodes related to permissions                                            |

[^1]: The naming of subprotocols follows the domain name like convention [commonly used](https://www.iana.org/assignments/websocket/websocket.xml#subprotocol-name), e.g `write.nodes.stencila.org`. But for brevity, the `.stencila.org` suffix is omitted in this document.

On a WebSocket upgrade request the server will only allow connections using subprotocols corresponding to the permissions that the user has for the document. For example, one of the `maintainers` of a `CreativeWork` would be permitted to connect using either the `admin.nodes.stencila.org` subprotocol (for a visual editor) or one of the `admin.<FORMAT>.stencila.org` subprotocols (for a code editor), where `<FORMAT>` is one of the document formats supported by Stencila.

> [!TODO]
> Think about how the "mode" of the document affects the allowed protocols for anon users

| Role[^2]       | Allowed protocols                 |
| -------------- | --------------------------------- |
| Anon           | `read.<FORMAT>`                   |
| `contributors` | `comment.nodes`, `suggest.nodes`  |
| `editors`      | + `edit.nodes`                   |
| `authors`      | + `write.nodes`, `write.<FORMAT>` |
| `maintainers`  | + `admin.nodes`, `admin.<FORMAT>` |

[^2]: A user has a role if they are a member of one of the following properties of a [`CreativeWork`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative-work.md). A user has the "anon" role if they are not in any of those properties.

### Client classes

| Client class                                      | Subprotocol             | Description                                                                                                                                                  |
| ------------------------------------------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | --- |
| [`Client`](src/clients/client.ts)                 | Any                     | Abstract base class for all client classes; maintains a WebSocket connection to the server including reconnecting after disconnections. |
| [`FormatClient`](src/clients/format.ts)           | `<CAPABILITY>.<FORMAT>` | Abstract base class for clients of a document (represented in a particular format) which can send and receive `FormatPatch`s.                                |     |
| [`NodesClient`](src/clients/nodes.ts)             | `<CAPABILITY>.nodes`    | Abstract base class for clients of a document which can send and receive `NodePatch`s; forwards in-browser `NodeEvent`s to the server as `NodePatch`s.                                                                       |
| [`DomClient`](src/clients/dom.ts)                 | `read.html`             | Read-only client of a document's HTML which updates the browser DOM when it receive `FormatPatch`s from the server.                                          |
| [`CodeMirrorClient`](src/clients/codemirror.ts)   | `write.<FORMAT>`        | Read-write client of a document (represented in a particular format) which synchronizes content with a CodeMirror editor.                                                                   |
| [`ProseMirrorClient`](src/clients/prosemirror.ts) | `prose.nodes`           | Read-write client of a document which synchronizes content with a ProseMirror editor.                                                                   |

## 🛠️ Develop

### Testing

There are some HTML files for "manual" testing of components and clients. During development, you can launch these on localhost using Parcel e.g.

```console
npx parcel src/clients/test.html
```

> [!NOTE]
> The Parcel config currently uses `parcel/transformer-typescript-tsc` because of this [issue](https://github.com/parcel-bundler/parcel/issues/7425) related to decorators.
