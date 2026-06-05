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

  private editor?: Editor

  private client?: TiptapClient

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

    this.client = new TiptapClient(this.doc)
    this.client.status = (status) => {
      this.syncStatus = SYNC_STATUS_LABELS[status]
      if (status === 'synced' || status === 'reset') {
        this.editor?.setEditable(true)
      }
    }
    this.client.receivePatches(this.editor)
  }

  override disconnectedCallback() {
    this.client?.destroy()
    this.client = undefined
    this.editor?.destroy()
    this.editor = undefined
    super.disconnectedCallback()
  }

  override render() {
    return html`
      <div class="stencila-edit-view" data-root-type=${this.type}>
        <div class="stencila-edit-status">
          <span>${this.syncStatus}</span>
        </div>
        <div class="stencila-tiptap-editor"></div>
      </div>
    `
  }
}
