/**
 * Light-DOM document edit view backed by a Tiptap editor.
 *
 * The view starts with an empty read-only editor and becomes editable only
 * after the first server synchronization has populated canonical Tiptap JSON.
 */
import { Editor } from '@tiptap/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { TiptapClient, type TiptapClientStatus } from '../clients/tiptap'
import type { DocumentId } from '../types'
import { createStencilaTiptapExtensions } from '../tiptap/extensions'
import { initUno } from '../unocss'

import './edit.css'
import './edit/nodes/toolbar'

initUno()

const SYNC_STATUS_LABELS: Record<TiptapClientStatus, string> = {
  connected: 'Connected',
  disconnected: 'Disconnected',
  synced: 'Synced',
  reset: 'Server reset',
}

/**
 * Tiptap edit view of a document.
 */
@customElement('stencila-edit-view')
export class EditView extends LitElement {
  /**
   * The server-side document identifier used by the WebSocket client.
   */
  @property()
  doc: DocumentId

  /**
   * The root Stencila node type, used for styling hooks.
   */
  @property()
  type = 'Article'

  /**
   * Short synchronization status label shown in the view chrome.
   */
  @state()
  private syncStatus = 'Connecting'

  @state()
  private canUndo = false

  @state()
  private canRedo = false

  private editor?: Editor

  private client?: TiptapClient

  private handleEditorTransaction = () => {
    this.canUndo = this.editor?.can().undo() ?? false
    this.canRedo = this.editor?.can().redo() ?? false
  }

  /**
   * Use Light DOM so document theme styles apply directly to editor content.
   */
  protected override createRenderRoot() {
    this.replaceChildren()
    return this
  }

  protected override firstUpdated() {
    const element = this.querySelector<HTMLElement>('.stencila-tiptap-editor')
    if (!element) {
      return
    }

    this.editor = new Editor({
      element,
      extensions: createStencilaTiptapExtensions(),
      content: {
        type: 'doc',
        content: [],
      },
      editable: false,
      editorProps: {
        attributes: {
          class: 'stencila-tiptap-root',
          root: '',
          spellcheck: 'true',
        },
      },
    })
    this.editor.on('transaction', this.handleEditorTransaction)
    this.handleEditorTransaction()
    // `editor` is a plain (non-reactive) field, so re-render explicitly to pass
    // the now-created editor to the node toolbar's `.editor` binding.
    this.requestUpdate()

    this.client = new TiptapClient(this.doc)
    this.client.status = (status) => {
      this.syncStatus = SYNC_STATUS_LABELS[status]
      if (status === 'synced' || status === 'reset') {
        if (this.editor && !this.editor.isEditable) {
          this.editor.setEditable(true, false)
        }
        this.handleEditorTransaction()
      }
    }
    this.client.receivePatches(this.editor)
  }

  override disconnectedCallback() {
    this.client?.destroy()
    this.client = undefined
    this.editor?.off('transaction', this.handleEditorTransaction)
    this.editor?.destroy()
    this.editor = undefined
    super.disconnectedCallback()
  }

  private keepEditorFocused(event: MouseEvent) {
    event.preventDefault()
  }

  private undo() {
    if (this.editor?.commands.undo()) {
      this.editor.view.focus()
      this.handleEditorTransaction()
    }
  }

  private redo() {
    if (this.editor?.commands.redo()) {
      this.editor.view.focus()
      this.handleEditorTransaction()
    }
  }

  override render() {
    return html`
      <div class="stencila-edit-view" data-root-type=${this.type}>
        <div class="stencila-edit-chrome">
          <div class="stencila-edit-toolbar" role="toolbar" aria-label="Editor history">
            <button
              type="button"
              class="stencila-edit-tool"
              aria-label="Undo"
              title="Undo (Ctrl/Cmd+Z)"
              ?disabled=${!this.canUndo}
              @mousedown=${this.keepEditorFocused}
              @click=${this.undo}
            >
              <span class="i-lucide:undo-2" aria-hidden="true"></span>
            </button>
            <button
              type="button"
              class="stencila-edit-tool"
              aria-label="Redo"
              title="Redo (Ctrl/Cmd+Shift+Z)"
              ?disabled=${!this.canRedo}
              @mousedown=${this.keepEditorFocused}
              @click=${this.redo}
            >
              <span class="i-lucide:redo-2" aria-hidden="true"></span>
            </button>
          </div>
          <div class="stencila-edit-status">
            <span>${this.syncStatus}</span>
          </div>
        </div>
        <div class="stencila-tiptap-editor"></div>
        <stencila-edit-node-toolbar
          .editor=${this.editor}
        ></stencila-edit-node-toolbar>
      </div>
    `
  }
}
