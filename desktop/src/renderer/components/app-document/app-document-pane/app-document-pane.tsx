import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Listen, Prop } from '@stencil/core'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { IResizeEvent } from 'split-me/dist/types/components/split-me/interfaces'
import { state } from '../../../store'
import {
  isEditPaneOpen,
  isPreviewPaneOpen,
} from '../../../store/documentPane/documentPaneActions'
import {
  selectActiveView,
  selectLayoutModuleCount,
  selectView,
} from '../../../store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-pane',
  styleUrl: 'app-document-pane.css',
  scoped: true,
})
export class AppDocumentPane {
  private splitSizes: number[] | undefined

  @Prop() paneId!: EntityId

  @Listen('slotResized')
  resizeHandler(e: CustomEvent<IResizeEvent>) {
    const { sizes } = e.detail
    this.splitSizes = sizes
  }

  render() {
    const activeDocumentId = selectActiveView(state)

    return (
      <div class="documentPane">
        <app-document-pane-tabs
          activeDocument={activeDocumentId}
          paneId={this.paneId}
        ></app-document-pane-tabs>

        {pipe(
          activeDocumentId,
          O.map(selectView(state)(this.paneId)),
          O.map(({ view, layout }) => {
            if (!view || !layout) {
              return
            }

            const isEditPaneVisible = isEditPaneOpen(layout)
            const isPreviewPaneVisible = isPreviewPaneOpen(layout)
            const moduleCount = selectLayoutModuleCount(state.panes)(layout.id)

            return (
              <div class="documentPaneContents">
                <app-document-pane-action-bar
                  docId={view.id}
                  paneId={this.paneId}
                ></app-document-pane-action-bar>

                <split-me
                  n={moduleCount}
                  sizes={this.splitSizes ?? layout.sizes}
                  minSizes={A.makeBy(moduleCount, () => 0.05)}
                  d={layout.orientation}
                >
                  {isEditPaneVisible ? (
                    <app-document-editor
                      documentId={view.id}
                      slot="0"
                      key="editor"
                    ></app-document-editor>
                  ) : null}
                  {isPreviewPaneVisible ? (
                    <app-document-preview
                      documentId={view.id}
                      slot={isEditPaneVisible ? `1` : `0`}
                      key="preview"
                    ></app-document-preview>
                  ) : null}
                </split-me>
              </div>
            )
          }),
          O.getOrElse(() => <app-document-pane-empty></app-document-pane-empty>)
        )}
      </div>
    )
  }
}
