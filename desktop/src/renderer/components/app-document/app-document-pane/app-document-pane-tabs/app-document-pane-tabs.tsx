import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'
import { option as O, string } from 'fp-ts'

@Component({
  tag: 'app-document-pane-tabs',
  styleUrl: 'app-document-pane-tabs.css',
  scoped: true,
})
export class AppDocumentPaneTabs {
  @Prop() activeDocument: O.Option<string>

  @Prop() documents: string[] = []

  @Prop() paneId: EntityId

  private eq = O.getEq(string.Eq).equals

  private isActive = (path: string): boolean => {
    return this.eq(this.activeDocument, O.some(path))
  }

  render() {
    return (
      <Host>
        <ul class="documentPaneTabs">
          {this.documents.map((docPath) => (
            <app-document-pane-tab
              isActive={this.isActive(docPath)}
              documentPath={docPath}
              paneId={this.paneId}
            ></app-document-pane-tab>
          ))}
        </ul>
      </Host>
    )
  }
}
