import { Component, h, Host, Prop, State, Watch } from '@stencil/core'
import { state } from '../../../store'
import { DocumentEvent } from 'stencila'
import { CHANNEL } from '../../../../preload'
import { selectDocSubscriptionTopics } from '../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  scoped: true,
})
export class AppDocumentPreview {
  @Prop() documentId: string

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.closeDoc(prevValue).then(() => {
        this.subscribeToUpdates(newValue)
      })
    }
  }

  @State() previewContents: string

  private subscribeToUpdates = (documentId = this.documentId) => {
    window.api.invoke(CHANNEL.DOCUMENT_GET_PREVIEW, documentId)
    window.api.receive(CHANNEL.DOCUMENT_GET_PREVIEW, (event) => {
      const e = event as DocumentEvent
      if (
        e.type === 'encoded' &&
        e.document.id === documentId &&
        e.content !== undefined &&
        e.format == 'html'
      ) {
        this.previewContents = e.content
      }
    })
  }

  private closeDoc = (documentId = this.documentId) =>
    window.api.invoke(CHANNEL.UNSUBSCRIBE_DOCUMENT, {
      documentId,
      topics: ['encoded:html'],
    })

  componentWillLoad() {
    this.subscribeToUpdates()
  }

  render() {
    return (
      <Host>
        <div class="app-document-preview">
          <p>Temporary: JSON preview of document content</p>
          <pre innerHTML={this.previewContents}></pre>
        </div>
      </Host>
    )
  }
}
