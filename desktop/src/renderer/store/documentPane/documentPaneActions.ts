import { EntityId } from '@reduxjs/toolkit'
import { store } from '../index'
import { documentPaneActions } from './documentPaneStore'

export const initPane = (path: string) => {
  store.dispatch(
    documentPaneActions.createPane({ id: path + 'main', documents: [] })
  )
}

export const addDocumentToPane = (paneId: EntityId, docPath: string) => {
  store.dispatch(
    documentPaneActions.addDocToPane({
      paneId,
      docPath,
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

export const setActiveDocument = (paneId: EntityId, filePath: string) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeDocument: filePath },
    })
  )
}
