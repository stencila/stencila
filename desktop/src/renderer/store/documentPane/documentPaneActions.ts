import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { Document } from 'stencila'
import { errorToast } from '../../../renderer/utils/errors'
import { client } from '../../client'
import { clearEditorState } from '../editorState/editorStateActions'
import { state, store } from '../index'
import { selectPaneId } from './documentPaneSelectors'
import { documentPaneActions } from './documentPaneStore'
import { PaneLayout } from './documentPaneTypes'

export const isPreviewable = (doc: Document) => doc.previewable

export const isPreviewPaneOpen = (layout: PaneLayout) =>
  layout.modules.includes('preview')

export const isEditable = (doc: Document) => doc.format.binary === false

export const isEditPaneOpen = (layout: PaneLayout) =>
  layout.modules.includes('editor')

export const initPane = (paneId: EntityId) => {
  store.dispatch(documentPaneActions.createPane({ paneId }))
}

export const addDocumentToPane = async (paneId: EntityId, path: string) => {
  try {
    const { value: doc } = await client.documents.open(path)

    return store.dispatch(
      documentPaneActions.addDocToPane({
        paneId,
        doc,
      })
    )
  } catch (err) {
    errorToast(err)
  }
}

export const addDocumentToActivePane = async (path: string) =>
  pipe(
    state,
    selectPaneId,
    O.map((paneId) => addDocumentToPane(paneId, path))
  )

export const closeDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.removeDocFromPane({
      paneId,
      docId,
    })
  )

  // TODO: Only clear state if document isn't open in any view
  // This isn't currently a problem as we don't support opening the same document
  // in multiple panes.
  clearEditorState(docId)
}

export const setActiveDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeView: O.some(docId) },
    })
  )
}

export const setPreviewPaneVisibility = (
  layoutId: EntityId,
  isVisible: boolean
) => {
  store.dispatch(
    documentPaneActions.setPaneModuleVisibility({
      layoutId,
      module: 'preview',
      isVisible,
    })
  )
}

export const setEditorPaneVisibility = (
  layoutId: EntityId,
  isVisible: boolean
) => {
  store.dispatch(
    documentPaneActions.setPaneModuleVisibility({
      layoutId,
      module: 'editor',
      isVisible,
    })
  )
}
