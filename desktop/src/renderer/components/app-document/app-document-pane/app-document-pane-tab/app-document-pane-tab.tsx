import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import {
  closeDocument,
  setActiveDocument,
} from '../../../../store/documentPane/documentPaneActions'
import { selectDoc } from '../../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-pane-tab',
  styleUrl: 'app-document-pane-tab.css',
  scoped: true,
})
export class AppDocumentPaneTab {
  @Prop() isActive: boolean

  @Prop() documentId: string

  @Prop() paneId: EntityId

  private activateDoc = (e: MouseEvent) => {
    e.preventDefault()
    setActiveDocument(this.paneId, this.documentId)
  }

  private closeDoc = (e: MouseEvent) => {
    e.stopPropagation()
    e.preventDefault()
    closeDocument(this.paneId, this.documentId)
  }

  render() {
    return (
      <Host class={{ isActive: this.isActive }} onClick={this.activateDoc}>
        <li>
          <stencila-icon icon="close" onClick={this.closeDoc}></stencila-icon>
          <a href="#">{selectDoc(state)(this.documentId)?.name}</a>
        </li>
      </Host>
    )
  }
}
