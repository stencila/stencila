import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'
import { option as O } from 'fp-ts'
import { constFalse, pipe } from 'fp-ts/function'
import { state } from '../../../../../renderer/store'
import { selectPaneViews } from '../../../../../renderer/store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-pane-tabs',
  styleUrl: 'app-document-pane-tabs.css',
  scoped: true,
})
export class AppDocumentPaneTabs {
  @Prop() activeDocument: O.Option<EntityId>

  @Prop() paneId: EntityId

  private isActive = (id: EntityId): boolean => {
    return pipe(
      this.activeDocument,
      O.map((activeId) => activeId === id),
      O.getOrElse(constFalse)
    )
  }

  render() {
    return (
      <Host>
        <ul class="documentPaneTabs">
          {selectPaneViews(state)(this.paneId).map((docId) => (
            <app-document-pane-tab
              isActive={this.isActive(docId)}
              viewId={docId}
              paneId={this.paneId}
              key={docId}
            ></app-document-pane-tab>
          ))}
        </ul>
      </Host>
    )
  }
}
