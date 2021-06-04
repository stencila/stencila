import { createSlice, EntityId, PayloadAction } from '@reduxjs/toolkit'
import { array as A, option as O, string } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import {
  PaneView,
  DocumentPane,
  NormalizedDocumentPaneStore,
} from './documentPaneTypes'

const initialState: NormalizedDocumentPaneStore = {
  entities: {
    panes: {},
    views: {},
  },
  ids: [],
}

export const documentPaneSlice = createSlice({
  name: 'documentPanes',
  initialState: initialState,
  reducers: {
    createPane: (state) => {
      const newPaneId = state.ids.length + 1
      state.entities.panes[newPaneId] = {
        id: newPaneId,
        activeView: O.none,
        views: [],
      }
      state.ids = [...state.ids, newPaneId]
    },
    updatePane: (
      state,
      {
        payload,
      }: PayloadAction<{ id: EntityId; changes: Partial<DocumentPane> }>
    ) => {
      const prevPane = state.entities.panes[payload.id]
      if (prevPane) {
        state.entities.panes[payload.id] = {
          ...prevPane,
          ...payload.changes,
        }
      }
    },
    addDocToPane: (
      state,
      { payload }: PayloadAction<{ paneId: EntityId; view: PaneView }>
    ) => {
      const pane = state.entities.panes[payload.paneId]
      if (pane) {
        if (!pane.views.includes(payload.view.id)) {
          pane.views = [...pane.views, payload.view.id]

          state.entities.views[payload.view.id] = payload.view
        }
        pane.activeView = O.some(payload.view.id)
      }
      return state
    },
    removeDocFromPane: (
      state,
      { payload }: PayloadAction<{ paneId: EntityId; docId: EntityId }>
    ) => {
      const pane = state.entities.panes[payload.paneId]

      if (pane) {
        const docIndex = pane.views.indexOf(payload.docId)

        // Remove document from list
        if (pane.views.includes(payload.docId)) {
          pane.views = pipe(
            pane.views,
            A.deleteAt(docIndex),
            O.getOrElse<EntityId[]>(() => [])
          )
        }

        // If document being closed is not the currently active document,
        // change focus to the closest tab
        if (
          pipe(
            pane.activeView,
            O.map((doc) => doc === payload.docId),
            O.getOrElse(() => false)
          )
        ) {
          pane.activeView = pipe(
            A.lookup(docIndex)(pane.views),
            O.alt(() => A.lookup(docIndex - 1)(pane.views))
          )
        }
      }

      return state
    },
  },
})

export const documentPaneActions = documentPaneSlice.actions
export type DocumentPaneActions = typeof documentPaneActions
