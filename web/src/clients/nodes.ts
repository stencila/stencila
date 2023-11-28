import { Capability } from "../capability";
import { DocumentId } from "../ids";

import { Client } from "./client";

export type NodePatch = PutMap

export interface PutMap {
  key: string
  value: unknown
}

export class NodesClient extends Client {
  /**
   * Construct a new `NodesClient`
   *
   * Listens for `node-patch` events and sends them as a WebSocket message
   * to the server.
   *
   * @param docId        The id of the document
   * @param capability   The capability of the client (e.g. "input", "admin")
   * @param elem         The element to which an event listener will be attached
   */
  constructor(docId: DocumentId, capability: Capability, elem: HTMLElement) {
    super(docId, `${capability}.nodes`);

    elem.addEventListener("node-patch", (event: CustomEvent) => {
      this.sendMessage(event.detail);
    });
  }
}
