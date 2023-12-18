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
   * Construct a new `CodeMirrorClient`
   *
   * @param id The id of the document
   * @param access The access level of the client
   * @param format The format of the editor content (e.g. "markdown")
   */
  constructor(id: DocumentId, access: DocumentAccess, format: string) {
    super(id, access, format)
  }

  /**
   * Send any buffered operations and increment the version number
   */
  private sendBufferedOperations() {
    // If the last operation is only inserting whitespace, do not send.
    //
    // TODO: This needs to be more refined: it needs to allow for spaces to be
    // inserted in paragraphs and sent immediately, but not spaces at end of
    // paragraphs.
    // https://github.com/stencila/stencila/issues/1788
    const op = this.bufferedOperations[this.bufferedOperations.length - 1]
    // For some reason (maybe because this runs async and cached operations
    // get filtered above?) `op` can occasionally be undefined so catch that here
    if (op && op.insert && op.insert.trim().length === 0) {
      return
    }

    // TODO: Coalesce operations as much as possible to reduce the number sent
    // https://github.com/stencila/stencila/issues/1787

    // Don't send empty patches
    if (this.bufferedOperations.length === 0) {
      return
    }

    // Send the patch and clear buffered operations
    this.sendPatch(this.bufferedOperations)
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
      update.changes.iterChanges((from, to, fromB, toB, inserted) => {
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
    const isReset = ops.length === 1 && ops[0].type === 'reset'

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version != this.version + 1) {
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
      transaction = { changes: ops as ({ from: number } & FormatOperation)[] }
    }

    // Dispatch the transaction, ignoring any updates while doing so
    this.ignoreUpdates = true
    this.editor.dispatch(transaction)
    this.ignoreUpdates = false

    // Update local version number
    this.version = version
  }
}
