import { store } from '../index'
import { documentPaneActions } from './documentPaneStore'

export const initPane = (path: string) => {
  store.dispatch(
    documentPaneActions.createPane({ id: path + 'main', documents: [] })
  )
}

export const setActiveDocument = (paneId: string, filePath: string) => {
  store.dispatch(
    documentPaneActions.updatePane({
      id: paneId,
      changes: { activeDocument: filePath },
    })
  )
}
