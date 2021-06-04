import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'
import { CHANNEL } from '../../../preload'
import { store } from '../index'
import { documentPaneActions } from './documentPaneStore'

export const initPane = () => {
  store.dispatch(documentPaneActions.createPane())
}

export const addDocumentToPane = async (paneId: EntityId, docPath: string) => {
  const document = (await window.api.invoke(
    CHANNEL.OPEN_DOCUMENT,
    docPath
  )) as Document

  return store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      doc: { type: 'editor', ...document },
    })
  )
}

export const closeDocument = (paneId: EntityId, docPath: string) => {
  store.dispatch(
    documentPaneActions.removeDocFromPane({
      paneId,
      docPath,
    })
  )
}

export const setActiveDocument = (paneId: EntityId, docId: string) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeView: O.some(docId) },
    })
  )
}
