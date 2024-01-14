import { diffApply, jsonPatchPathConverter } from 'just-diff-apply'

import { type DocumentId } from '../types'

import { Client } from './client'

/**
 * A patch to apply to a JavaScript object representing a document
 *
 * This is a JSONPatch (https://jsonpatch.com/) but with a version
 * number to be able to detect lost patches and request a reset patch
 * if necessary.
 *
 * A reset patch had a single 'replace' operation and the root path ('/')
 * specified.
 */
export interface ObjectPatch {
  /**
   * The version of the patch
   */
  version: number

  /**
   * The operations in the patch
   */
  ops: ObjectOperation[]
}

/**
 * An operation within an `ObjectPatch`
 */
export interface ObjectOperation {
  /**
   * The type of operation
   */
  op: 'add' | 'remove' | 'replace'

  /**
   * The JSON path to be added to, removed, replaced etc
   */
  path: string

  /**
   * The JSON path to be moved or copied from
   */
  from?: string

  /**
   * The value to be added or is the replacement
   */
  value?: unknown
}

/**
 * A read-only client for a JavaScript object representing a document
 */
export class ObjectClient extends Client {
  /**
   * The JavaScript object representing the state of the document
   */
  protected state: object = {}

  /**
   * The local version of the object
   *
   * Used to check for missed patches and request a
   * reset patch if necessary.
   */
  protected version: number = 0

  /**
   * A subscriber to the string
   *
   * A function that is called whenever a patch is applied to the
   * object `state`.
   */
  protected subscriber?: (patch: ObjectPatch, object: object) => void

  /**
   * Construct a new `ObjectClient`
   *
   * @param id The id of the document
   */
  constructor(id: DocumentId) {
    super(id, 'read.object')
  }

  /**
   * Receive a message from the server
   *
   * An override to apply an incoming `ObjectPatch` message to the
   * local, in-browser, version of the object.
   */
  override receiveMessage(message: Record<string, unknown>) {
    const patch = message as unknown as ObjectPatch
    const { version, ops } = patch

    // Is the patch a reset patch?
    const isReset =
      ops.length === 1 && ops[0].op === 'replace' && ops[0].path == ''

    if (isReset) {
      // Apply the new value
      this.state = patch.ops[0].value as object
    } else {
      // Check for non-sequential patch and request a reset patch if necessary
      if (version != this.version + 1) {
        this.sendMessage({ version: 0 })
        return
      }

      // Apply the patch. If any errors doing so, request a reset patch
      try {
        // @ts-expect-error because the `diffApply` typings for the path of ops is wrong
        // when using a path converter
        diffApply(this.state, ops, jsonPatchPathConverter)
      } catch (error) {
        console.error('Error applying object patch', error)
        this.sendMessage({ version: 0 })
        return
      }
    }

    // Update local version number
    this.version = version

    // Notify the subscriber (if any)
    if (this.subscriber) {
      this.subscriber(patch, this.state)
    }
  }

  /**
   * Subscribe to changes in the string from within the browser
   *
   * @param subscriber The subscriber function which will be called
   *                   with the patch and the updated object each time it changes
   */
  public subscribe(subscriber: (patch: ObjectPatch, object: object) => void) {
    this.subscriber = subscriber
  }
}