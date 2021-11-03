import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop, State, Watch } from '@stencil/core'
import { File } from 'stencila'
import { state } from '../../../../renderer/store'
import { selectDoc } from '../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  shadow: true,
})
export class AppDocumentPreview {
  /**
   * ID of the document to be previewed
   */
  @Prop() documentId: EntityId

  private updateDoc = (id: EntityId) => {
    const doc = selectDoc(state)(id)
    if (doc) {
      this.doc = doc
    }
  }

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.updateDoc(newValue)
    }
  }

  @State() doc: File

  componentWillLoad() {
    this.updateDoc(this.documentId)
  }

  render() {
    return (
      <Host>
        <iframe
          title="document-preview"
          src={`http://127.0.0.1:9000/${this.doc.path}`}
          sandbox="allow-scripts"
        />
      </Host>
    )
  }
}
