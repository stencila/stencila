import { createSelector, EntityId } from '@reduxjs/toolkit'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { RootState } from '..'
import { makeLayoutId } from './documentPaneStore'
import { NormalizedDocumentPaneStore } from './documentPaneTypes'

export const selectPaneId = (state: RootState): O.Option<EntityId> => {
  return state.panes.activePane
}

export const selectPaneViews = (state: RootState) => (paneId: EntityId) => {
  return state.panes.entities.panes[paneId]?.views ?? []
}

export const selectDoc = (state: RootState) => (docId: EntityId) => {
  return state.panes.entities.views[docId]
}

export const isTemporaryDocument =
  (state: RootState) =>
  (viewId: EntityId): boolean => {
    return state.panes.entities.views[viewId]?.temporary ?? false
  }

export const selectLayout = (state: RootState) => (layoutId: EntityId) => {
  return state.panes.entities.layouts[layoutId]
}

export const selectView =
  (state: RootState) => (paneId: EntityId) => (viewId: EntityId) => {
    return {
      view: selectDoc(state)(viewId),
      layout: selectLayout(state)(makeLayoutId(paneId)(viewId)),
    }
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

export const selectLayoutModuleCount = createSelector(
  (panes: NormalizedDocumentPaneStore) => panes.entities.layouts,
  (layouts) => (layoutId: EntityId) => layouts[layoutId]?.modules.length ?? 0
)
