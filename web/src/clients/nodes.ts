import { type Node } from '@stencila/types'

import { type NodeId, type DocumentAccess, type DocumentId } from '../types'

import { Client } from './client'

/**
 * A patch to apply to a node within a document
 *
 * See the `document` Rust crate for the server-side structure of patches
 * (which this should be consistent with, if not exactly the same).
 */
export interface NodePatch {
  /**
   * The id of the document node this is the target of this patch
   */
  id: NodeId

  /**
   * The patch operation
   */
  op: 'add' | 'remove' | 'replace' | 'copy' | 'move'

  /**
   * The path to the property or item from which to `move` or `copy`
   */
  path: number | string

  /**
   * The value of the property or item to `add` or `replace`
   */
  from?: number | string
  value?: Node
}

/**
 * The name of the `CustomEvent` for node patches emitted by
 * custom elements a views in the browser DOM
 */
const NODE_PATCH_EVENT = 'stencila-node-patch'

/**
 * Create a `CustomEvent` containing a `NodePatch`
 */
export function nodePatchEvent(patch: NodePatch): CustomEvent {
  return new CustomEvent(NODE_PATCH_EVENT, { detail: patch, bubbles: true })
}

/**
 * A write-only client which listens for `stencila-node-patch` events and
 * sends the `NodePatch` on that event a WebSocket message to the server.
 */
export class NodesClient extends Client {
  /**
   * Construct a new `NodesClient`
   *
   * @param id  The id of the document
   * @param access The access level of the client
   * @param elem The element to which an event listener will be attached
   */
  constructor(
    id: DocumentId,
    access: DocumentAccess,
    elem: HTMLElement,
    clientType?: string
  ) {
    super(id, `${access}.nodes`, clientType ?? 'node')

    elem.addEventListener(NODE_PATCH_EVENT, (event: CustomEvent) => {
      this.sendMessage(event.detail)
    })
  }
}
