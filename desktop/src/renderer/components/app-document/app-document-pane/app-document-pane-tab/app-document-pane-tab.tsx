import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import { selectProjectFile } from '../../../../store/project/projectSelectors'

@Component({
  tag: 'app-document-pane-tab',
  styleUrl: 'app-document-pane-tab.css',
  scoped: true,
})
export class AppDocumentPaneTab {
  @Prop() documentPath: string

  render() {
    return (
      <Host>
        <li class="documentPaneTab">
          {selectProjectFile(state)(this.documentPath)?.name}
        </li>
      </Host>
    )
  }
}
