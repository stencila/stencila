/**
 * WebSocket client for synchronizing a Tiptap editor with a document encoded as
 * Tiptap JSON.
 *
 * The server treats the JSON document as a string format. Local edits are
 * debounced and sent as whole-document replacements, while incoming server
 * patches replace the editor content when the canonical JSON changes.
 */
import type { Editor } from '@tiptap/core'

import type { DocumentId } from '../types'

import {
  FormatClient,
  FormatPatch,
  codePointLength,
} from './format'

const SEND_DEBOUNCE = 300

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
   * Register a status callback and immediately replay the last known status.
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
   * Attach the editor that will send and receive Tiptap JSON patches.
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
   */
  public destroy() {
    this.clearBufferedEdit()
    this.editor?.off('update', this.editorUpdateHandler)
    this.editor = undefined
    this.close()
  }

  protected override handleConnected() {
    this.updateStatus('connected')
  }

  protected override handleDisconnected() {
    this.updateStatus('disconnected')
  }

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

  private clearBufferedEdit() {
    clearTimeout(this.sendTimer)
    this.sendTimer = undefined
    this.bufferedJson = undefined
  }

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
      this.editor.commands.setContent(json, { emitUpdate: false })
    } finally {
      this.ignoreUpdates = false
    }
    if (status) {
      this.updateStatus(status)
    }
  }

  private updateStatus(status: TiptapClientStatus) {
    this.lastStatus = status
    this.statusHandler?.(status)
  }
}
