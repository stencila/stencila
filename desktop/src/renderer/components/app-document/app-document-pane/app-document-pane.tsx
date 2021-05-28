import { Component, h } from '@stencil/core'
import { state } from '../../../store'
import {
  selectActiveDoc,
  selectPaneDocs,
  selectPaneId,
} from '../../../store/documentPane/documentPaneSelectors'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'

@Component({
  tag: 'app-document-pane',
  styleUrl: 'app-document-pane.css',
  scoped: true,
})
export class AppDocumentPane {
  render() {
    const activeDocument = selectActiveDoc(state)

    return (
      <div class="documentPane">
        <app-document-pane-tabs
          activeDocument={activeDocument}
          paneId={selectPaneId(state)}
          documents={selectPaneDocs(state)(selectPaneId(state))}
        ></app-document-pane-tabs>

        {pipe(
          activeDocument,
          O.map((activeFilePath) => (
            <div class="documentPaneContents">
              <app-document-editor
                filePath={activeFilePath}
              ></app-document-editor>
              <app-document-preview
                filePath={activeFilePath}
              ></app-document-preview>
            </div>
          )),
          O.getOrElse(() => <app-document-pane-empty></app-document-pane-empty>)
        )}
      </div>
    )
  }
}
