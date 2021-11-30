import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import {
  closeDocument,
  setActiveDocument,
} from '../../../../store/documentPane/documentPaneActions'
import { selectDoc } from '../../../../store/documentPane/documentPaneSelectors'
import { userOS } from '../../../../utils/env'
import { showAndCaptureError } from '../../../../utils/errors'

@Component({
  tag: 'app-document-pane-tab',
  styleUrl: 'app-document-pane-tab.css',
  scoped: true,
})
export class AppDocumentPaneTab {
  /** Indicates whether the tab is active, if so it is styled differently */
  @Prop() isActive: boolean

  /** ID of the view/document being represented by this tab */
  @Prop() viewId: EntityId

  /** ID of the editor pane this tab is attached to */
  @Prop() paneId: EntityId

  private activateDoc = (e: MouseEvent) => {
    e.preventDefault()
    setActiveDocument(this.paneId, this.viewId)
  }

  private closeDoc = (e: MouseEvent) => {
    e.stopPropagation()
    e.preventDefault()
    closeDocument(this.paneId, this.viewId).catch((err) =>
      showAndCaptureError(err)
    )
  }

  render() {
    const doc = selectDoc(state)(this.viewId)
    return (
      <Host
        class={{
          isActive: this.isActive,
          [`userOS-${userOS ?? 'unknown'}`]: true,
          [doc?.status ?? '']: true,
        }}
        onClick={this.activateDoc}
      >
        <li>
          <stencila-icon
            class="closeTabIcon"
            icon="close"
            onClick={this.closeDoc}
          ></stencila-icon>
          <stencila-icon
            icon="pencil"
            iconStyle="fill"
            class="documentStatusIcon"
          ></stencila-icon>
          <a href="#">{doc?.name}</a>
        </li>
      </Host>
    )
  }
}
