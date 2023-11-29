import { LitElement } from "lit";

import { nodePatchEvent, NodePatch } from "../clients/nodes";
import { DocumentAccess } from "../types";

/**
 * The abstract base class for all custom elements for Stencila Schema
 * node types corresponding to the `Entity` node type.
 */
export abstract class Entity extends LitElement {
  /**
   * Get the name of the view that this custom element is contained within
   * 
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  documentView(): string {
    return this.closest("[view]").getAttribute("view")
  }

  /**
   * Get the document access level of the view that this custom element
   * is contained within
   * 
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  documentAccess(): DocumentAccess {
    return this.closest("[view]").getAttribute("access") as DocumentAccess
  }

  /**
   * Patch the node that this custom element represents
   *
   * Emits a `CustomEvent` containing a `NodePatch` which is forwarded by
   * the `NodesClient` to the document on the server.
   */
  patchNode(patch: NodePatch) {
    this.dispatchEvent(nodePatchEvent(patch));
  }
}
