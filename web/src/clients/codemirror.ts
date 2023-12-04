import { Extension, TransactionSpec } from "@codemirror/state";
import { EditorView, ViewUpdate } from "@codemirror/view";

import { type DocumentAccess, type DocumentId } from "../types";

import { FormatOperation, FormatPatch, FormatClient } from "./format";

/// The number milliseconds to debounce sending updates
const SEND_DEBOUNCE = 300;

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
  private editor?: EditorView;

  /**
   * Whether updates from the editor should be ignored
   *
   * Used to temporarily ignore updates while applying patches from
   * the server.
   */
  private ignoreUpdates = false;

  /**
   * A cache of `FormatOperation`s used to debounce sending patches to the server
   */
  private cachedOperations: FormatOperation[] = [];

  /**
   * Construct a new `CodeMirrorClient`
   *
   * @param id The id of the document
   * @param access The access level of the client
   * @param format The format of the editor content (e.g. "markdown")
   */
  constructor(id: DocumentId, access: DocumentAccess, format: string) {
    super(id, access, format);
  }

  /**
   * Send patches to the server by listening to updates from the code editor
   *
   * @returns A CodeMirror `Extension` to use when creating a new editor
   */
  public sendPatches(): Extension {
    let timer: string | number | NodeJS.Timeout;
    return EditorView.updateListener.of((update: ViewUpdate) => {
      if (this.ignoreUpdates || !update.docChanged) {
        return;
      }

      update.changes.iterChanges((from, to, fromB, toB, inserted) => {
        const insert = inserted.toJSON().join("\n");
        const op: FormatOperation = { from, to };
        if (insert) op.insert = insert;
        this.cachedOperations.push(op);
      });

      clearTimeout(timer);

      timer = setTimeout(() => {
        // If the last operation is only inserting whitespace, do not send.
        //
        // TODO: This needs to be more refined: it needs to allow for spaces to be
        // inserted in paragraphs and sent immediately, but not spaces at end of
        // paragraphs.
        // https://github.com/stencila/stencila/issues/1788
        const op = this.cachedOperations[this.cachedOperations.length - 1];
        if (op.insert && op.insert.trim().length === 0) {
          return;
        }

        // TODO: Coalesce operations as much as possible to reduce the number sent
        // https://github.com/stencila/stencila/issues/1787

        // Send the patch
        this.sendMessage({
          version: this.version,
          ops: this.cachedOperations,
        });

        // Increment version and clear cache of ops
        this.version += 1;
        this.cachedOperations = [];
      }, SEND_DEBOUNCE);
    });
  }

  /**
   * Receive patches from the server and apply them to the content of the code editor
   *
   * @param editor The CodeMirror editor that will receive patches from the server
   */
  public receivePatches(editor: EditorView) {
    this.editor = editor;

    // Set the initial content of the code editor to the current state
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: this.state },
    });
  }

  /**
   * Override to forward patches directly to the CodeMirror editor instead
   * of updating `this.state`
   */
  override receiveMessage(message: Record<string, unknown>) {
    const { version, ops } = message as unknown as FormatPatch;

    // Is the patch a reset patch?
    const isReset = ops.length === 1 && ops[0].from === 0 && ops[0].to === 0;

    // Check for non-sequential patch and request a reset patch if necessary
    if (!isReset && version != this.version + 1) {
      this.sendMessage({ version: 0 });
      return;
    }

    // Create a transaction for the patch
    let transaction: TransactionSpec;
    if (isReset) {
      transaction = this.editor.state.update({
        changes: {
          from: 0,
          to: this.editor.state.doc.length,
          insert: ops[0].insert,
        },
        selection: this.editor.state.selection,
      });
    } else {
      transaction = { changes: ops };
    }

    // Dispatch the transaction, ignoring any updates while doing so
    this.ignoreUpdates = true;
    this.editor.dispatch(transaction);
    this.ignoreUpdates = false;

    // Update local version number
    this.version = version;
  }
}
