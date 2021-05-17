import { Component, h } from '@stencil/core'
import { state } from '../../../store'
import { selectPane } from '../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-pane',
  styleUrl: 'app-document-pane.css',
  scoped: true,
})
export class AppDocumentPane {
  render() {
    return (
      <div class="documentPane">
        <app-document-pane-tabs
          documents={[selectPane(state)?.activeDocument ?? '']}
        ></app-document-pane-tabs>

        <app-document-preview
          filePath={selectPane(state)?.activeDocument}
        ></app-document-preview>
      </div>
    )
  }
}
