import { type DocumentAccess, type DocumentId } from '../types'

import { Client } from './client'

/**
 * A patch to apply to a string representing a document in a particular format
 *
 * See the `document` Rust crate for the server-side structure of patches
 * (which this should be consistent with, if not exactly the same).
 */
export interface FormatPatch {
  /**
   * The version of the patch
   */
  version: number

  /**
   * The operations in the patch
   */
  ops?: FormatOperation[]
}

/**
 * An operation on a format string
 */
export interface FormatOperation {
  /**
   * The type of operation
   */
  type: 'reset' | 'insert' | 'replace' | 'delete' | 'execute' | 'selection'

  /**
   * The position in the string from which the operation is applied
   */
  from?: number

  /**
   * The position in the string to which the operation is applied
   *
   * May be omitted for additions.
   */
  to?: number

  /**
   * The string to insert between `from` and `to`.
   *
   * For additions and replacements; may be omitted for deletions.
   */
  insert?: string
}

/**
 * A read-write client for a string representation of a document in a particular format
 */
export abstract class FormatClient extends Client {
  /**
   * The local state of the string
   */
  protected state: string = ''

  /**
   * The local version of the string
   *
   * Used to check for missed patches and request a
   * reset patch if necessary.
   */
  protected version: number = 0

  /**
   * A subscriber to the string
   *
   * A function that is called whenever a patch is applied to the
   * string `state`.
   */
  protected subscriber?: (value: string) => void

  /**
   * Construct a new `FormatClient`
   *
   * @param id The id of the document
   * @param access The access level of the client
   * @param format The format of the string (e.g. "html", "markdown")
   */
  constructor(id: DocumentId, access: DocumentAccess, format: string) {
    super(id, `${access}.${format}`)
  }

  /**
   * Send patch operations to the server with current version and increment
   * the version
   */
  protected sendPatch(ops: FormatOperation[]) {
    this.sendMessage({
      version: this.version,
      ops,
    })

    this.version += 1
  }

  /**
   * Receive a message from the server
   *
   * An override to apply the incoming `FormatPatch` message to the
   * local, in-browser, version of the string.
   */
  override receiveMessage(message: Record<string, unknown>) {
    const { version, ops } = message as unknown as FormatPatch

    // Is the patch a reset patch?
    const isReset = ops.length === 1 && ops[0].type === 'reset'

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version != this.version + 1) {
      this.sendMessage({ version: 0 })
      return
    }

    // Apply each operation in the patch
    let updated = false
    for (const op of ops) {
      const { type, from, to, insert } = op

      if (type === 'reset' && insert !== undefined) {
        this.state = insert
        updated = true
      } else if (
        type === 'insert' &&
        from !== undefined &&
        insert !== undefined
      ) {
        this.state = this.state.slice(0, from) + insert + this.state.slice(from)
        updated = true
      } else if (type === 'delete' && from !== undefined && to !== undefined) {
        this.state = this.state.slice(0, from) + this.state.slice(to)
        updated = true
      } else if (
        type === 'replace' &&
        from !== undefined &&
        to !== undefined &&
        insert !== undefined
      ) {
        this.state = this.state.slice(0, from) + insert + this.state.slice(to)
        updated = true
      } else {
        console.error('Operation from server was not handled', op)
      }
    }

    if (updated) {
      // Update local version number
      this.version = version

      // Notify the subscriber (if any)
      if (this.subscriber) {
        this.subscriber(this.state)
      }
    }
  }

  /**
   * Subscribe to changes in the string from within the browser
   *
   * @param subscriber The subscriber function which will be called
   *                   with the string, each time it changes
   */
  public subscribe(subscriber: (value: string) => void) {
    this.subscriber = subscriber
  }
}
