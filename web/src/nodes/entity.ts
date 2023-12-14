import { LitElement } from 'lit'

import { nodePatchEvent, NodePatch } from '../clients/nodes'
import { DocumentAccess } from '../types'

/**
 * Abstract base class for web components representing Stencila Schema `Entity` node types
 *
 * Given that the `Entity` node type is the ancestor of all other node types in the Stencila
 * Schema (other than "primitive" types), this is the ancestor class of all web components
 * for node types. As such it implements and few core methods commonly used.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md
 *
 * Note that this does not need to have an `id` property (as in the `Entity` schema)
 * because `id` is already a property of `HTMLElement` from which this is derived.
 */
export abstract class Entity extends LitElement {
  /**
   * Get the name of the view that this web component is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentView(): string {
    return this.closest('[view]').getAttribute('view')
  }

  /**
   * Get the document access level of the view that this web component
   * is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentAccess(): DocumentAccess {
    return this.closest('[view]').getAttribute('access') as DocumentAccess
  }

  /**
   * Patch the node that this web component represents
   *
   * Emits a `CustomEvent` containing a `NodePatch` which is forwarded by
   * the `NodesClient` to the document on the server.
   */
  protected patchNode(patch: NodePatch) {
    this.dispatchEvent(nodePatchEvent(patch))
  }
}
