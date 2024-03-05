import type { NodeType } from '@stencila/types'
import { diffApply, jsonPatchPathConverter } from 'just-diff-apply'

import type { NodeId, DocumentAccess, DocumentId } from '../types'

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
   * The type of operation for operations on the string content
   */
  type?: 'reset' | 'insert' | 'replace' | 'delete' | 'viewport' | 'selection'

  /**
   * The position in the string from which the operation is applied
   */
  from?: number | string

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

  /**
   * The type of operation for operations on the mapping object
   */
  op?: 'add' | 'remove' | 'replace'

  /**
   * The JSON path to be added to, removed, replaced etc in the mapping object
   */
  path?: string

  /**
   * The value to be added or is the replacement in the mapping object
   */
  value?: unknown
}

/**
 * An entry in the mapping between character positions and nodes and their properties
 *
 * Uses offsets for the start and end positions (rather than absolute values) to reduce
 * the size of patches sent by the server to update the mapping.
 */
export interface MappingEntry {
  /**
   * The offset of the start this entry from the start of the previous entry
   */
  start: number

  /**
   * The offset of the end this entry from the end of the previous entry
   */
  end: number

  /**
   * The type of the node
   */
  nodeType: NodeType

  /**
   * The id of the node
   */
  nodeId: NodeId

  /**
   * The name of the node property, if applicable
   */
  property?: string
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
   * The mapping between character ranges and nodes and their properties
   */
  protected mapping: MappingEntry[] = []

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
   * the version if any of the operations modify content
   */
  protected sendPatch(ops: FormatOperation[]) {
    const modified = ops.find((op) =>
      ['reset', 'insert', 'replace', 'delete'].includes(op.type)
    )

    this.sendMessage({
      version: this.version,
      ops,
    })

    if (modified) {
      this.version += 1
    }
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
    const isReset = ops.length >= 1 && ops[0].type === 'reset'

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version > this.version + 1) {
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
        typeof from === 'number' &&
        insert !== undefined
      ) {
        this.state = this.state.slice(0, from) + insert + this.state.slice(from)
        updated = true
      } else if (
        type === 'delete' &&
        typeof from === 'number' &&
        typeof to === 'number'
      ) {
        this.state = this.state.slice(0, from) + this.state.slice(to)
        updated = true
      } else if (
        type === 'replace' &&
        typeof from === 'number' &&
        typeof to === 'number' &&
        insert !== undefined
      ) {
        this.state = this.state.slice(0, from) + insert + this.state.slice(to)
        updated = true
      } else if (op.op !== undefined) {
        if (op.op == 'replace' && op.path === '') {
          this.mapping = op.value as MappingEntry[]
        } else {
          // @ts-expect-error because the `diffApply` typings for the path of ops is wrong
          // when using a path converter
          diffApply(this.mapping, [op], jsonPatchPathConverter)
        }
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

  /**
   * Get the type and id of the node (and any property name) corresponding to character position
   *
   * Returns the first entry in the mapping which spans the position (i.e. the most
   * leafiest node in the node tree)
   */
  public nodeAt(position: number): MappingEntry | undefined {
    let start = 0
    let end = 0
    for (const entry of this.mapping) {
      start += entry.start
      end += entry.end
      if (position >= start && position < end) {
        return {
          nodeType: entry.nodeType,
          nodeId: entry.nodeId,
          property: entry.property,
          start,
          end,
        }
      }
    }

    return undefined
  }

  /**
   * Get the id of all nodes corresponding to character position
   *
   * Returns a list of nodes that have a range that spans the position.
   * The first node in the list is the leafiest node, the last is the
   * root node (an `Article`, or possibly some other `CreativeWork` in the future).
   */
  public nodesAt(position: number): MappingEntry[] {
    let start = 0
    let end = 0
    const nodes: MappingEntry[] = []
    for (const entry of this.mapping) {
      start += entry.start
      end += entry.end
      if (position >= start && position < end) {
        if (
          !nodes.find((existing) => existing.nodeId === entry.nodeId) &&
          !entry.property
        ) {
          nodes.push({ ...entry, start, end })
        }
      }
    }

    return nodes
  }

  /**
   * Return a list of nodes within the given range
   *
   * Will ONLY include nodes that start and end within the range,
   * unless the `allowPartailNodes`
   *
   * If no nodes are found or the range start is greater than the range end,
   * will return an empty array.
   */
  public nodesInRange(
    from: number,
    to: number,
    allowPartialNodes: boolean = false
  ): MappingEntry[] {
    let start = 0
    let end = 0

    const nodes: MappingEntry[] = []
    if (from > to) {
      return nodes
    }
    for (const entry of this.mapping) {
      start += entry.start
      end += entry.end
      // If condition set, get nodes within or partialy within the range,
      // else only include complete nodes within the range
      if (allowPartialNodes) {
        if ((from >= start && from < end) || (to >= start && to < end)) {
          nodes.push({ ...entry, start, end })
        }
      } else if (start >= from && start < to && end > from && end <= to) {
        nodes.push({ ...entry, start, end })
      }
    }
    console.log(nodes)
    return nodes
  }
}
