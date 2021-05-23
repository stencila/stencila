import { Component, h, Host, Prop, State, Watch } from '@stencil/core'
import { CHANNEL } from '../../../../preload'
import { DocumentEvent } from 'stencila'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  scoped: true,
})
export class AppDocumentPreview {
  @Prop() filePath: string

  @Watch('filePath')
  filePathWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.closeDoc(prevValue).then(() => {
        this.subscribeToUpdates(newValue)
      })
    }
  }

  @State() previewContents: string

  private subscribeToUpdates = (filePath = this.filePath) => {
    window.api.invoke(CHANNEL.DOCUMENT_GET_PREVIEW, filePath)
    window.api.receive(CHANNEL.DOCUMENT_GET_PREVIEW, (event) => {
      const e = event as DocumentEvent
      if (e.type === 'converted' && e.path === filePath) {
        this.previewContents = e.content
      }
    })
  }

  private closeDoc = (filePath = this.filePath) =>
    window.api.invoke(CHANNEL.CLOSE_DOCUMENT, filePath)

  componentWillLoad() {
    this.subscribeToUpdates()
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
