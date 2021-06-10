import { Component, h, Listen } from '@stencil/core'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { IResizeEvent } from 'split-me/dist/types/components/split-me/interfaces'
import { state } from '../../../store'
import {
  selectActiveView,
  selectPaneId,
  selectPaneViews
} from '../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-pane',
  styleUrl: 'app-document-pane.css',
  scoped: true
})
export class AppDocumentPane {
  private splitSizes: number[] | undefined

  @Listen('slotResized')
  resizeHandler(e: CustomEvent<IResizeEvent>) {
    const { sizes } = e.detail
    this.splitSizes = sizes
  }

  render() {
    const activeDocument = selectActiveView(state)

    return (
      <div class="documentPane">
        <app-document-pane-tabs
          activeDocument={activeDocument}
          paneId={selectPaneId(state)}
          viewIds={selectPaneViews(state)(selectPaneId(state))}
        ></app-document-pane-tabs>

        {pipe(
          activeDocument,
          O.map(activeDocumentId => (
            <div class="documentPaneContents">
              <split-me
                n={2}
                sizes={this.splitSizes ?? [0.5, 0.5]}
                minSizes={[0.05, 0.05]}
                d="horizontal"
              >
                <app-document-editor
                  documentId={activeDocumentId}
                  slot="0"
                ></app-document-editor>
                <app-document-preview
                  documentId={activeDocumentId}
                  slot="1"
                ></app-document-preview>
              </split-me>
            </div>
          )),
          O.getOrElse(() => <app-document-pane-empty></app-document-pane-empty>)
        )}
      </div>
    )
  }
}
