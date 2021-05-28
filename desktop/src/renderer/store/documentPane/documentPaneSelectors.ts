import { EntityId } from '@reduxjs/toolkit'
import { RootState } from '..'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'

export const selectPaneId = (state: RootState) => {
  return state.panes.ids[0]
}

export const selectPaneDocs = (state: RootState) => (paneId?: EntityId) => {
  return paneId ? state.panes.entities[paneId]?.documents ?? [] : []
}

export const selectActiveDoc = (state: RootState) => {
  return pipe(
    selectPane(state),
    O.chain((pane) => pane.activeDocument)
  )
}

export const selectPane = (state: RootState) => {
  return pipe(
    state.panes.ids,
    A.head,
    O.chain((id) => O.some(state.panes.entities[id]) ?? O.none),
    O.chain(O.fromNullable)
  )
}
