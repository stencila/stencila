import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop, State, Watch } from '@stencil/core'
import { DocumentEvent } from 'stencila'
import { CHANNEL } from '../../../../preload'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  scoped: true,
})
export class AppDocumentPreview {
  @Prop() documentId: EntityId

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.unsubscribeFromDocument(prevValue).then(() => {
        this.subscribeToDocument(newValue)
      })
    }
  }

  @State() previewContents: string

  private subscribeToDocument = (documentId = this.documentId) => {
    window.api.invoke(CHANNEL.DOCUMENT_GET_PREVIEW, documentId).then((html) => {
      this.previewContents = html as string
    })

    window.api.receive(CHANNEL.DOCUMENT_GET_PREVIEW, (event) => {
      const e = event as DocumentEvent
      if (
        e.document.id === documentId &&
        e.type === 'encoded' &&
        e.format == 'html' &&
        e.content !== undefined
      ) {
        this.previewContents = e.content
      }
    })
  }

  private unsubscribeFromDocument = (documentId = this.documentId) =>
    window.api.invoke(CHANNEL.UNSUBSCRIBE_DOCUMENT, {
      documentId,
      topics: ['encoded:html'],
    })

  componentWillLoad() {
    this.subscribeToDocument()
  }

  disconnectedCallback() {
    this.unsubscribeFromDocument()
  }

  render() {
    return (
      <Host>
        <div
          class="app-document-preview"
          innerHTML={this.previewContents}
        ></div>
      </Host>
    )
  }
}
