import { Component, h, Host, Prop } from '@stencil/core'

@Component({
  tag: 'app-document-pane-tabs',
  styleUrl: 'app-document-pane-tabs.css',
  scoped: true,
})
export class AppDocumentPaneTabs {
  @Prop() documents: string[] = []

  render() {
    return (
      <Host>
        <ul class="documentPaneTabs">
          {this.documents.map((docPath) => (
            <app-document-pane-tab
              documentPath={docPath}
            ></app-document-pane-tab>
          ))}
        </ul>
      </Host>
    )
  }
}
