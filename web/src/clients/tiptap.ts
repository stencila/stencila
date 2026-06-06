/**
 * WebSocket client for synchronizing a Tiptap editor with a document encoded as
 * Tiptap JSON.
 *
 * The server treats the JSON document as a string format. Local edits are
 * debounced and sent as whole-document replacements, while incoming server
 * patches replace the editor content when the canonical JSON changes.
 */
import { type Content, type Editor, createDocument } from '@tiptap/core'
import type { Node as ProseMirrorNode } from '@tiptap/pm/model'
import { Selection, TextSelection } from '@tiptap/pm/state'

import { isHistoryPlugin } from '../tiptap/history'
import type { DocumentId } from '../types'

import {
  FormatClient,
  FormatPatch,
  codePointLength,
} from './format'

const SEND_DEBOUNCE = 300

interface CapturedSelection {
  anchor: number
  head: number
  json: unknown
}

function clampPosition(position: number, max: number): number {
  return Math.max(0, Math.min(position, max))
}

/**
 * Build a ProseMirror document from Tiptap JSON using the editor schema.
 *
 * Centralizes the `createDocument` options so every server-driven comparison and
 * replacement honors the editor's `enableContentCheck` setting.
 */
function createEditorDocument(editor: Editor, json: unknown): ProseMirrorNode {
  return createDocument(
    json as Content,
    editor.schema,
    {},
    { errorOnInvalidContent: editor.options.enableContentCheck }
  )
}

/**
 * Compare raw JSON values when a real ProseMirror schema is unavailable.
 *
 * Production Tiptap editors use schema-aware document equality, but lightweight
 * editor doubles in sync tests only expose `getJSON`. This fallback lets those
 * schema-less editors skip server replacements that are structurally identical
 * apart from object key order or omitted `undefined` values.
 */
function jsonValuesEqual(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) {
    return true
  }

  if (typeof left !== typeof right || left === null || right === null) {
    return false
  }

  if (Array.isArray(left) || Array.isArray(right)) {
    return (
      Array.isArray(left) &&
      Array.isArray(right) &&
      left.length === right.length &&
      left.every((value, index) => jsonValuesEqual(value, right[index]))
    )
  }

  if (typeof left !== 'object') {
    return false
  }

  const leftObject = left as Record<string, unknown>
  const rightObject = right as Record<string, unknown>
  const leftKeys = Object.keys(leftObject)
    .filter((key) => leftObject[key] !== undefined)
    .sort()
  const rightKeys = Object.keys(rightObject)
    .filter((key) => rightObject[key] !== undefined)
    .sort()

  return (
    leftKeys.length === rightKeys.length &&
    leftKeys.every(
      (key, index) =>
        key === rightKeys[index] &&
        jsonValuesEqual(leftObject[key], rightObject[key])
    )
  )
}

/**
 * User-visible synchronization state for the Tiptap editor.
 */
export type TiptapClientStatus =
  | 'connected'
  | 'disconnected'
  | 'synced'
  | 'reset'

/**
 * A read-write client that keeps a Tiptap editor synchronized with Tiptap JSON.
 */
export class TiptapClient extends FormatClient {
  private editor?: Editor

  private ignoreUpdates = false

  private sendTimer?: ReturnType<typeof setTimeout>

  private bufferedJson?: string

  private lastStatus?: TiptapClientStatus

  private statusHandler?: (status: TiptapClientStatus) => void

  private editorUpdateHandler = () => this.bufferEditorUpdate()

  /**
   * Register a status callback and immediately replay the last status.
   *
   * The edit view can attach this callback after connection state has already
   * changed, and replaying prevents the visible sync badge from missing the
   * current state.
   */
  public set status(handler: (status: TiptapClientStatus) => void) {
    this.statusHandler = handler
    if (this.lastStatus) {
      handler(this.lastStatus)
    }
  }

  constructor(id: DocumentId) {
    super(id, 'write', 'tiptap')
  }

  /**
   * Attach the editor that sends and receives Tiptap JSON patches.
   *
   * Only one editor should be the active sync source at a time; detaching any
   * previous update listener avoids duplicate sends and stale editor writes.
   */
  public receivePatches(editor: Editor) {
    this.editor?.off('update', this.editorUpdateHandler)
    this.editor = editor
    editor.on('update', this.editorUpdateHandler)

    if (this.state) {
      this.replaceEditorContent(this.state, 'synced')
    }
  }

  /**
   * Stop synchronizing and close the underlying WebSocket connection.
   *
   * Clearing pending local edits and removing editor listeners prevents delayed
   * updates from being sent after the view has been disconnected.
   */
  public destroy() {
    this.clearBufferedEdit()
    this.editor?.off('update', this.editorUpdateHandler)
    this.editor = undefined
    this.close()
  }

  /**
   * Mark the client as connected.
   *
   * The base WebSocket client owns connection lifecycle events, while this class
   * translates them into Tiptap-specific user-visible sync status.
   */
  protected override handleConnected() {
    this.updateStatus('connected')
  }

  /**
   * Mark the client as disconnected.
   *
   * The editor can remain mounted while the socket reconnects, so the view needs
   * a status update that reflects transport availability separately from editor
   * content.
   */
  protected override handleDisconnected() {
    this.updateStatus('disconnected')
  }

  /**
   * Apply a server patch and mirror changed state into the editor.
   *
   * `FormatClient` maintains canonical string state, but Tiptap needs parsed
   * JSON pushed back into ProseMirror whenever that string changes. Buffered
   * local edits are dropped when the server wins to keep the editor consistent.
   */
  override receiveMessage(message: Record<string, unknown>) {
    const { ops = [] } = message as unknown as FormatPatch
    const previousState = this.state
    const hadBufferedEdit = this.bufferedJson !== undefined

    super.receiveMessage(message)

    if (this.state !== previousState) {
      if (
        hadBufferedEdit &&
        this.serverStateMatchesPreviousDocument(previousState)
      ) {
        this.updateStatus('synced')
        return
      }

      const status = hadBufferedEdit ? 'reset' : 'synced'
      if (hadBufferedEdit) {
        this.clearBufferedEdit()
      }

      this.replaceEditorContent(this.state, status, {
        clearHistory: hadBufferedEdit,
      })
    } else if (ops.length > 0) {
      this.updateStatus('synced')
    }
  }

  /**
   * Buffer the editor's current JSON after a local Tiptap update.
   *
   * Serializing and sending the whole document on every keystroke is noisy;
   * buffering lets quick edits coalesce while ignoring updates caused by server
   * replacements.
   */
  private bufferEditorUpdate() {
    if (this.ignoreUpdates || !this.editor) {
      return
    }

    const nextJson = JSON.stringify(this.editor.getJSON())
    if (nextJson === this.state) {
      this.clearBufferedEdit()
      this.updateStatus('synced')
      return
    }

    this.bufferedJson = nextJson
    clearTimeout(this.sendTimer)
    this.sendTimer = setTimeout(() => this.sendBufferedJson(), SEND_DEBOUNCE)
  }

  /**
   * Send the buffered editor JSON as a whole-document replacement.
   *
   * The server currently treats Tiptap JSON as a string format, so local edits
   * are represented as one replace operation over the previous canonical string
   * rather than as ProseMirror steps.
   */
  private sendBufferedJson() {
    const nextJson = this.bufferedJson
    if (nextJson === undefined || nextJson === this.state) {
      this.clearBufferedEdit()
      return
    }

    const previousLength = codePointLength(this.state)
    this.sendPatch([
      {
        type: 'replace',
        from: 0,
        to: previousLength,
        insert: nextJson,
      },
    ])

    this.state = nextJson
    this.clearBufferedEdit()
    this.updateStatus('synced')
  }

  /**
   * Cancel any delayed local edit send and clear buffered JSON.
   *
   * Buffered edits become invalid when the editor is destroyed or when a newer
   * server state replaces the local draft before the debounce fires.
   */
  private clearBufferedEdit() {
    clearTimeout(this.sendTimer)
    this.sendTimer = undefined
    this.bufferedJson = undefined
  }

  /**
   * Replace the ProseMirror document with server-provided Tiptap JSON.
   *
   * Equivalent documents are skipped so server echoes do not disturb the editor.
   * When the content does change, replacement resets ProseMirror selection, so
   * the surrounding selection handling keeps the browser selection stable and
   * avoids the initial all-document selection left by loading content into an
   * empty editor. `clearHistory` additionally drops undo events that point back
   * into a discarded local draft after a server reset.
   */
  private replaceEditorContent(
    value: string,
    status?: TiptapClientStatus,
    options: { clearHistory?: boolean } = {}
  ) {
    if (!this.editor || !value) {
      return
    }

    let json: unknown
    try {
      json = JSON.parse(value)
    } catch (error) {
      console.error('Unable to parse Tiptap JSON from server', error)
      return
    }

    let document: ProseMirrorNode | undefined
    if (this.editor.schema) {
      try {
        document = createEditorDocument(this.editor, json)
      } catch (error) {
        console.error('Unable to create Tiptap document from server JSON', error)
        return
      }

      if (this.editor.state?.doc?.eq(document)) {
        if (status) {
          this.updateStatus(status)
        }
        return
      }
    } else if (jsonValuesEqual(this.editor.getJSON(), json)) {
      if (status) {
        this.updateStatus(status)
      }
      return
    }

    this.ignoreUpdates = true
    try {
      const wasEmpty = this.editor.state?.doc?.content.size === 0
      const selection = this.captureEditorSelection()

      this.replaceContentWithoutHistory(json, document)

      if (wasEmpty) {
        this.moveEditorSelectionToStart()
      } else if (selection) {
        this.restoreEditorSelection(selection)
      }

      if (options.clearHistory) {
        this.clearEditorHistory()
      }
    } finally {
      this.ignoreUpdates = false
    }
    if (status) {
      this.updateStatus(status)
    }
  }

  /**
   * Record and publish the latest synchronization status.
   *
   * Storing the last status supports late status handler registration, and
   * publishing through one method keeps UI status transitions consistent.
   */
  private updateStatus(status: TiptapClientStatus) {
    this.lastStatus = status
    this.statusHandler?.(status)
  }

  /**
   * Detect server acknowledgements that only normalize the previously sent JSON.
   *
   * A local undo can be buffered while the server is still echoing the edit that
   * is being undone. If that echo has equivalent ProseMirror content, keep the
   * pending local undo instead of treating the acknowledgement as a server reset.
   */
  private serverStateMatchesPreviousDocument(previousState: string): boolean {
    return this.documentsAreEquivalent(previousState, this.state)
  }

  /**
   * Compare two serialized Tiptap documents using the editor schema.
   */
  private documentsAreEquivalent(left: string, right: string): boolean {
    const editor = this.editor
    if (!editor?.schema || !left || !right) {
      return false
    }

    try {
      const leftDocument = createEditorDocument(editor, JSON.parse(left))
      const rightDocument = createEditorDocument(editor, JSON.parse(right))

      return leftDocument.eq(rightDocument)
    } catch {
      return false
    }
  }

  /**
   * Capture the current ProseMirror selection before content replacement.
   *
   * Server replacements can rebuild the document and reset selection to the end.
   * Keeping both JSON and directional anchor/head positions lets the client
   * restore normal forward and backward selections afterward.
   */
  private captureEditorSelection(): CapturedSelection | undefined {
    const selection = this.editor?.state?.selection
    if (!selection) {
      return undefined
    }

    return {
      anchor: selection.anchor,
      head: selection.head,
      json: selection.toJSON(),
    }
  }

  /**
   * Restore a captured selection after replacing editor content.
   *
   * Preserving selection prevents double-click and drag selections from
   * disappearing after a server echo. If the exact JSON selection no longer fits
   * the new document, the fallback clamps the old direction-aware positions.
   */
  private restoreEditorSelection(captured: CapturedSelection) {
    const editor = this.editor
    const doc = editor?.state?.doc
    const view = editor?.view
    if (!editor || !doc || !view) {
      return
    }

    let selection: Selection | undefined
    try {
      selection = Selection.fromJSON(doc, captured.json)
    } catch {
      try {
        const max = doc.content.size
        selection = TextSelection.between(
          doc.resolve(clampPosition(captured.anchor, max)),
          doc.resolve(clampPosition(captured.head, max)),
          captured.anchor <= captured.head ? 1 : -1
        )
      } catch {
        const position = clampPosition(captured.head, doc.content.size)
        selection = Selection.near(doc.resolve(position))
      }
    }

    if (!selection.eq(editor.state.selection)) {
      view.dispatch(editor.state.tr.setSelection(selection))
    }
  }

  /**
   * Put the cursor at the first selectable position after initial load.
   *
   * Replacing an empty editable document can leave ProseMirror with an
   * all-document selection whose head is at the end, which makes subsequent
   * browser selection behave as if it is anchored to the final character.
   */
  private moveEditorSelectionToStart() {
    const editor = this.editor
    const doc = editor?.state?.doc
    const view = editor?.view
    if (!editor || !doc || !view) {
      return
    }

    const selection = Selection.near(doc.resolve(0), 1)
    if (!selection.eq(editor.state.selection)) {
      view.dispatch(editor.state.tr.setSelection(selection))
    }
  }

  /**
   * Clear stale local undo/redo events after an authoritative server reset.
   *
   * ProseMirror keeps existing history entries and maps them across transactions
   * with `addToHistory: false`. That is useful for collaborative rebasing, but a
   * server reset that discards buffered local JSON should leave no undo event
   * that points back into the discarded local document.
   */
  private clearEditorHistory() {
    const editor = this.editor
    const state = editor?.state
    const view = editor?.view
    if (!state || !view) {
      return
    }

    const historyIndex = state.plugins.findIndex(isHistoryPlugin)
    if (historyIndex === -1) {
      return
    }

    const historyPlugin = state.plugins[historyIndex]
    const pluginsWithoutHistory = state.plugins.filter(
      (_plugin, index) => index !== historyIndex
    )
    const stateWithoutHistory = state.reconfigure({
      plugins: pluginsWithoutHistory,
    })
    const pluginsWithFreshHistory = [
      ...pluginsWithoutHistory.slice(0, historyIndex),
      historyPlugin,
      ...pluginsWithoutHistory.slice(historyIndex),
    ]

    view.updateState(
      stateWithoutHistory.reconfigure({ plugins: pluginsWithFreshHistory })
    )
  }

  /**
   * Replace editor content without making server synchronization undoable.
   *
   * Local edits should be the only entries in the user's undo stack. Server
   * loads and resets update the document state, but should not make Undo jump
   * back to the previous synced document or the initial empty editor.
   */
  private replaceContentWithoutHistory(
    json: unknown,
    document?: ProseMirrorNode
  ) {
    const editor = this.editor
    if (!editor) {
      return
    }

    if (!editor.schema || !editor.view || !editor.state?.tr) {
      editor.commands.setContent(json, { emitUpdate: false })
      return
    }

    document ??= createEditorDocument(editor, json)
    // Insert the document's block children rather than the `doc` node itself: a
    // `doc` cannot legally nest inside a `doc`, so passing the whole node relies
    // on ProseMirror's slice fitting to unwrap it. `document.content` is already
    // a fragment of valid top-level blocks, so the replacement needs no fitting.
    const transaction = editor.state.tr
      .replaceWith(0, editor.state.doc.content.size, document.content)
      .setMeta('preventUpdate', true)
      .setMeta('addToHistory', false)

    editor.view.dispatch(transaction)
  }
}
