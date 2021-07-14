import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Listen, Prop } from '@stencil/core'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { IResizeEvent } from 'split-me/dist/types/components/split-me/interfaces'
import { state } from '../../../store'
import {
  selectActiveView,
  selectDoc,
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
          O.map(selectDoc(state)),
          O.map((activeDocument) => {
            const isEditable = !activeDocument?.format.binary
            const isPreviewable = activeDocument?.previewable
            const paneCount = isEditable && isPreviewable ? 2 : 1

            return (
              <div class="documentPaneContents">
                <split-me
                  n={paneCount}
                  sizes={
                    this.splitSizes ?? A.makeBy(paneCount, () => 1 / paneCount)
                  }
                  minSizes={A.makeBy(paneCount, () => 0.05)}
                  d="horizontal"
                >
                  {isEditable ? (
                    <app-document-editor
                      documentId={activeDocument?.id}
                      slot="0"
                    ></app-document-editor>
                  ) : null}
                  {isPreviewable ? (
                    <app-document-preview
                      documentId={activeDocument?.id}
                      slot={isEditable ? `1` : `0`}
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
