/**
 * WebSocket client for synchronizing a Tiptap editor with a document encoded as
 * Tiptap JSON.
 *
 * The server treats the JSON document as a string format. Local edits are
 * debounced and sent as whole-document replacements, while incoming server
 * patches replace the editor content when the canonical JSON changes.
 */
import type { Editor } from '@tiptap/core'
import { Selection, TextSelection } from '@tiptap/pm/state'

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
      const status = hadBufferedEdit ? 'reset' : 'synced'
      if (hadBufferedEdit) {
        this.clearBufferedEdit()
      }

      this.replaceEditorContent(this.state, status)
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
   * `setContent` is necessary after canonical server changes, but it also
   * mutates ProseMirror selection. The surrounding selection handling keeps the
   * browser selection stable and avoids the initial all-document selection left
   * by loading content into an empty editor.
   */
  private replaceEditorContent(value: string, status?: TiptapClientStatus) {
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

    this.ignoreUpdates = true
    try {
      const wasEmpty = this.editor.state?.doc?.content.size === 0
      const selection = this.captureEditorSelection()

      this.editor.commands.setContent(json, { emitUpdate: false })

      if (wasEmpty) {
        this.moveEditorSelectionToStart()
      } else if (selection) {
        this.restoreEditorSelection(selection)
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
}
