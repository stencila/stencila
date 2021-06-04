import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'
import { CHANNEL } from '../../../preload'
import { store } from '../index'
import { documentPaneActions } from './documentPaneStore'

export const initPane = () => {
  store.dispatch(documentPaneActions.createPane())
}

export const addDocumentToPane = async (paneId: EntityId, docId: EntityId) => {
  const document = (await window.api.invoke(
    CHANNEL.OPEN_DOCUMENT,
    docId
  )) as Document

  return store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      view: { type: 'editor', ...document },
    })
  )
}

export const closeDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.removeDocFromPane({
      paneId,
      docId,
    })
  )
}

export const setActiveDocument = (paneId: EntityId, docId: EntityId) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeView: O.some(docId) },
    })
  )
}
