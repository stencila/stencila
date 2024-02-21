import { Extension, SelectionRange, TransactionSpec } from '@codemirror/state'
import { EditorView, ViewUpdate } from '@codemirror/view'

import { type DocumentAccess, type DocumentId } from '../types'

import { FormatOperation, FormatPatch, FormatClient } from './format'

/// The number milliseconds to debounce sending updates
const SEND_DEBOUNCE = 300

/**
 * A read-write client that keeps a CodeMirror editor synchronized with a
 * string representation of a document in a particular format.
 *
 * To send patches it is necessary to create the client first
 * and add the return value of `sendPatches` to the `extensions` of the
 * editor constructor.
 *
 * To receive patches call the `receivePatches` method with the editor
 * instance.
 *
 * e.g.
 *
 * ```ts
 * const client = new CodeMirrorClient(...);
 *
 * const editor = new EditorView({
 *   extensions: [client.sendPatches()]
 *   ...
 * })
 *
 * client.receivePatches(editor)
 * ```
 */
export class CodeMirrorClient extends FormatClient {
  /**
   * The CodeMirror view to update with patches from the server
   */
  private editor?: EditorView

  /**
   * Whether updates from the editor should be ignored
   *
   * Used to temporarily ignore updates while applying patches from
   * the server.
   */
  private ignoreUpdates = false

  /**
   * A buffer of `FormatOperation`s used to debounce sending patches to the server
   */
  private bufferedOperations: FormatOperation[] = []

  /**
   * The current selection in the editor
   *
   * Used to detect changes in the user's selection and send presence updates
   * accordingly.
   */
  private currentSelection?: SelectionRange

  /**
   * If true the client will not send messages from the server
   * back to the `editor`
   */
  private _writeOnly: boolean

  set writeOnly(value: boolean) {
    this._writeOnly = value
  }

  /**
   * Construct a new `CodeMirrorClient`
   *
   * @param id The id of the document
   * @param access The access level of the client
   * @param format The format of the editor content (e.g. "markdown")
   */
  constructor(
    id: DocumentId,
    access: DocumentAccess,
    format: string,
    writeOnly: boolean = false
  ) {
    super(id, access, format)
    this._writeOnly = writeOnly
  }

  /**
   * Send any buffered operations and increment the version number
   */
  private sendBufferedOperations() {
    // Don't send empty patches
    if (this.bufferedOperations.length === 0) {
      return
    }

    // To reduce the number of operations, coalesce consecutive 'insert'
    // operations (as happens with successive key presses) into a single op
    let selection: FormatOperation
    const coalesced = this.bufferedOperations.reduce<FormatOperation[]>(
      (ops, current, index) => {
        if (index === 0) {
          ops.push(current)
        } else {
          const prev = ops[ops.length - 1]
          if (
            prev.type === 'insert' &&
            current.type === 'insert' &&
            +prev.from + prev.insert.length === current.from
          ) {
            ops[ops.length - 1] = {
              ...prev,
              insert: prev.insert + current.insert,
              to: current.to,
            }
          } else if (
            selection === undefined &&
            current.type == 'selection' &&
            index == this.bufferedOperations.length - 2
          ) {
            selection = current
          } else {
            ops.push(current)
          }
        }
        return ops
      },
      []
    )
    if (selection) {
      coalesced.push(selection)
    }

    // Send the patch and clear buffered operations
    this.sendPatch(coalesced)
    this.bufferedOperations = []
  }

  /**
   * Send patches to the server by listening to updates from the code editor
   *
   * @returns A CodeMirror `Extension` to use when creating a new editor
   */
  public sendPatches(): Extension {
    let timer: string | number | NodeJS.Timeout
    return EditorView.updateListener.of((update: ViewUpdate) => {
      if (this.ignoreUpdates) {
        return
      }

      let newOperations = false

      // Update the selection if necessary
      const selection = update.view.state.selection.main
      if (
        this.currentSelection?.from !== selection.from ||
        this.currentSelection?.to !== selection.to
      ) {
        this.currentSelection = selection

        // Only want to send the last presence so remove any existing
        // presence update patches
        this.bufferedOperations = this.bufferedOperations.filter(
          (op) => op.type !== 'selection'
        )

        // Add this one
        const { from, to } = selection
        this.bufferedOperations.push({ type: 'selection', from, to })
        newOperations = true
      }

      // Send changes
      update.changes.iterChanges((from, to, _fromB, _toB, inserted) => {
        //console.log(from, to, fromB, toB, inserted)

        const insert = inserted.toJSON().join('\n')

        let op: FormatOperation
        if (from === to && insert) {
          op = { type: 'insert', from, insert }
        } else if (from !== to && !insert) {
          op = { type: 'delete', from, to }
        } else if (from !== to && insert) {
          op = { type: 'replace', from, to, insert }
        } else {
          return
        }

        this.bufferedOperations.push(op)
        newOperations = true
      })

      if (newOperations) {
        clearTimeout(timer)
        timer = setTimeout(() => this.sendBufferedOperations(), SEND_DEBOUNCE)
      }
    })
  }

  /**
   * Send a special operation to the server
   *
   * The `from` and `to` character positions are resolved into document node(s)
   * on the server and the operation is applied there.
   *
   * @param type The type of the operation
   */
  public sendSpecial(type: 'execute', selection: SelectionRange) {
    const { from, to } = selection
    if (from === undefined) {
      console.error('Current selection is undefined')
    }

    // Ensure any buffered operations are sent first
    this.sendBufferedOperations()

    // Send the special operation itself
    const patch = {
      version: this.version,
      ops: [
        {
          type,
          from,
          to,
        },
      ],
    }
    this.sendMessage(patch)
  }

  /**
   * Receive patches from the server and apply them to the content of the code editor
   *
   * @param editor The CodeMirror editor that will receive patches from the server
   */
  public receivePatches(editor: EditorView) {
    this.editor = editor
    // Set the initial content of the code editor to the current state
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: this.state },
    })
  }

  /**
   * Override to forward patches from the server directly to the CodeMirror
   * view instead of updating `this.state`
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

    // Create a transaction for the patch
    let transaction: TransactionSpec
    if (isReset) {
      transaction = this.editor.state.update({
        changes: {
          from: 0,
          to: this.editor.state.doc.length,
          insert: ops[0].insert,
        },
        selection: this.editor.state.selection,
      })
    } else {
      const changes = ops.filter((op) => op.type !== undefined) as ({
        from: number
      } & FormatOperation)[]
      transaction = { changes }
    }

    /*
      If not in `writeOnly` mode Dispatch the transaction, 
      ignoring any updates while doing so.
    */
    if (!this._writeOnly) {
      this.ignoreUpdates = true
      this.editor.dispatch(transaction)
      this.ignoreUpdates = false
    }

    // Call `FormatClient.receiveMessage` to update mapping and version
    super.receiveMessage(message)
  }
}
