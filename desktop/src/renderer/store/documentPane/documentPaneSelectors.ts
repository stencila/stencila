import { EntityId } from '@reduxjs/toolkit'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { RootState } from '..'

export const selectPaneId = (state: RootState): O.Option<EntityId> => {
  return state.panes.activePane
}

export const selectPaneViews = (state: RootState) => (paneId: EntityId) => {
  return state.panes.entities.panes[paneId]?.views ?? []
}

export const selectDoc = (state: RootState) => (docId: EntityId) => {
  return state.panes.entities.documents[docId]
}

export const selectActiveView = (state: RootState): O.Option<EntityId> => {
  return pipe(
    selectPane(state),
    O.chain((pane) => pane.activeView)
  )
}

export const selectPane = (state: RootState) => {
  return pipe(
    state.panes.ids,
    A.head,
    O.chain((id) => O.some(state.panes.entities.panes[id]) ?? O.none),
    O.chain(O.fromNullable)
  )
}
