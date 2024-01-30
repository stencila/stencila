import { type Node } from '@stencila/types'

import { type DocumentId } from '../types'

import { Client } from './client'

/**
 * An action to apply to a path within a directory
 *
 * This should have the same structure as the Rust `DirectoryAction`
 * struct in `rust/document/sync_directory.rs`.
 */
export interface DirectoryAction {
  /**
   * The type of action
   */
  type: 'create-file' | 'create-directory' | 'delete' | 'rename'

  /**
   * The path to the file or subdirectory to which the action should be applied
   */
  path: string

  /**
   * The new path for renaming actions
   */
  to?: Node
}

/**
 * The name of the `CustomEvent` for node patches emitted by
 * custom elements a views in the browser DOM
 */
const DIRECTORY_ACTION_EVENT = 'stencila-directory-action'

/**
 * Create a `CustomEvent` containing a `DirectoryAction`
 */
export function nodePatchEvent(patch: DirectoryAction): CustomEvent {
  return new CustomEvent(DIRECTORY_ACTION_EVENT, {
    detail: patch,
    bubbles: true,
  })
}

/**
 * A write-only client which listens for `stencila-directory-action` events and
 * sends the `DirectoryAction` on that event a WebSocket message to the server.
 */
export class DirectoryClient extends Client {
  /**
   * Construct a new `DirectoryClient`
   *
   * @param id  The id of the directory document
   * @param elem The element to which an event listener will be attached
   */
  constructor(id: DocumentId, elem: HTMLElement) {
    super(id, `write.directory`)

    elem.addEventListener(DIRECTORY_ACTION_EVENT, (event: CustomEvent) => {
      this.sendMessage(event.detail)
    })
  }
}
