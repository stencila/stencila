import { EntityId } from '@reduxjs/toolkit'
import { RootState } from '..'

export const selectPaneId = (state: RootState) => {
  return state.panes.ids[0]
}

export const selectPaneDocs = (state: RootState) => (paneId?: EntityId) => {
  return paneId ? state.panes.entities[paneId]?.documents ?? [] : []
}

export const selectPane = (state: RootState) => {
  const id = state.panes.ids[0]
  if (id) {
    return state.panes.entities[id]
  }
}
