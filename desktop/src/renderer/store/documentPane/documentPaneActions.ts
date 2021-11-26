import { EntityId } from '@reduxjs/toolkit'
import { option as O, taskEither as TE } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { Document } from 'stencila'
import { client, isRPCError } from '../../client'
import { clearEditorState } from '../editorState/editorStateActions'
import { state, store } from '../index'
import { selectPaneId } from './documentPaneSelectors'
import { documentPaneActions } from './documentPaneStore'
import { PaneLayout } from './documentPaneTypes'

export const isPreviewable = (doc: Document) => doc.previewable

export const isPreviewPaneOpen = (layout: PaneLayout) =>
  layout.modules.includes('preview')

export const isEditable = ({ format }: Document) =>
  format.name === 'ipynb' ? false : format.binary === false

export const isEditPaneOpen = (layout: PaneLayout) =>
  layout.modules.includes('editor')

export const initPane = (paneId: EntityId) => {
  store.dispatch(documentPaneActions.createPane({ paneId }))
}

export const createNewDocument = async (path?: string, format?: string) => {
  const {
    value: { global },
  } = await client.config.getAll()

  const defaultFormat = global.editors?.defaultFormat

  createDocument(path, format ?? defaultFormat).then(({ value: doc }) => {
    addDocumentToActivePane(doc)()
  })
}

/**
 * Given a document path, open it with Stencila client and add to the provided Pane.
 * Note that this is different from `addDocumentToPane` which instead accepts an already open `Document`.
 */
export const openDocumentInPane = async (paneId: EntityId, path: string) => {
  const { value: doc } = await client.documents.open(path)

  return store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      doc,
    })
  )
}

/**
 * Given an open `Document`, add it the Pane with the provided ID.
 * Note that this is different from `openDocumentInPane` which instead accepts a file path.
 */
export const addDocumentToPane = async (paneId: EntityId, doc: Document) => {
  return store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      doc,
    })
  )
}

/**
 * Given a document path, open it with Stencila client and add to the currently focussed Pane.
 * Note that this is different from `addDocumentToActivePane` which instead accepts an already open `Document`.
 */
export const openDocumentInActivePane = (path: string) =>
  pipe(
    state,
    selectPaneId,
    TE.fromOption(() => 'Could not select active pane'),
    TE.chain((paneId) =>
      TE.tryCatch(
        () => openDocumentInPane(paneId, path),
        (err) => (isRPCError(err) ? err.message : err)
      )
    )
  )

/**
 * Given a document path, open it with Stencila client and add to the currently focussed Pane.
 * Note that this is different from `openDocumentInActivePane` which instead accepts a file path.
 */
export const addDocumentToActivePane = (doc: Document) =>
  pipe(
    state,
    selectPaneId,
    TE.fromOption(() => 'Could not select active pane'),
    TE.chain((paneId) =>
      TE.tryCatch(
        () => addDocumentToPane(paneId, doc),
        (err) => (isRPCError(err) ? err.message : err)
      )
    )
  )

export const createDocument = async (path?: string, format?: string) =>
  client.documents.create(path, format)

export const updateDocument = (doc: Document) => {
  return store.dispatch(documentPaneActions.updateDoc({ doc }))
}

export const patchDocument = (
  doc: Partial<Omit<Document, 'id'>> & { id: EntityId }
) => {
  return store.dispatch(documentPaneActions.patchDoc({ doc }))
}

export const getDocument = async (docId: EntityId) => {
  const { value: doc } = await client.documents.get(docId)
  updateDocument(doc)
}

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

  return client.documents.unsubscribe({
    documentId: docId,
    topics: ['modified'],
  })
}

export const setActiveDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeView: O.some(docId) },
    })
  )
}

export const cycleTabs = (direction: 'next' | 'previous') => {
  pipe(
    state,
    selectPaneId,
    O.map((id) => {
      store.dispatch(
        documentPaneActions.nextDocInPane({
          paneId: id,
          direction,
        })
      )
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
