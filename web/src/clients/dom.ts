import morphdom from "morphdom";

import { DocumentId } from "../ids";

import { FormatClient } from "./format";

/**
 * A client that keeps a DOM element synchronized with a
 * HTML buffer on the server
 */
export class DomClient extends FormatClient {
  /**
   * Construct a new `DomClient`
   * 
   * @param docId        The id of the document
   * @param elem         The DOM element that will be updated
   */
  constructor(docId: DocumentId, elem: HTMLElement) {
    super(docId, "read", "html");

    this.subscribe((html) => morphdom(elem, html));
  }
}
