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
    const activeDocument = selectPane(state)?.activeDocument

    return (
      <div class="documentPane">
        {activeDocument ? (
          [
            <app-document-pane-tabs
              documents={[activeDocument]}
            ></app-document-pane-tabs>,
            <div class="documentPaneContents">
              <app-document-editor
                filePath={activeDocument}
              ></app-document-editor>
              <app-document-preview
                filePath={activeDocument}
              ></app-document-preview>
            </div>,
          ]
        ) : (
          <app-document-pane-empty></app-document-pane-empty>
        )}
      </div>
    )
  }
}
