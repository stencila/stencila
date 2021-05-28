import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'

@Component({
  tag: 'app-document-pane-tabs',
  styleUrl: 'app-document-pane-tabs.css',
  scoped: true,
})
export class AppDocumentPaneTabs {
  @Prop() activeDocument: string

  @Prop() documents: string[] = []

  @Prop() paneId: EntityId

  render() {
    return (
      <Host>
        <ul class="documentPaneTabs">
          {this.documents.map((docPath) => (
            <app-document-pane-tab
              isActive={this.activeDocument === docPath}
              documentPath={docPath}
              paneId={this.paneId}
            ></app-document-pane-tab>
          ))}
        </ul>
      </Host>
    )
  }
}
