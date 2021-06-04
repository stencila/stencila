import { createSlice, EntityId, PayloadAction } from '@reduxjs/toolkit'
import { array as A, option as O, string } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import {
  DocumentView,
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
      { payload }: PayloadAction<{ paneId: EntityId; doc: DocumentView }>
    ) => {
      const pane = state.entities.panes[payload.paneId]
      if (pane) {
        if (!pane.views.includes(payload.doc.id)) {
          pane.views = [...pane.views, payload.doc.id]

          state.entities.views[payload.doc.id] = payload.doc
        }
        pane.activeView = O.some(payload.doc.id)
      }
      return state
    },
    removeDocFromPane: (
      state,
      { payload }: PayloadAction<{ paneId: EntityId; docPath: string }>
    ) => {
      const pane = state.entities.panes[payload.paneId]

      if (pane) {
        const docIndex = pane.views.indexOf(payload.docPath)

        // Remove document from list
        if (pane.views.includes(payload.docPath)) {
          pane.views = pipe(
            pane.views,
            A.deleteAt(docIndex),
            O.getOrElse<string[]>(() => [])
          )
        }

        // If document being closed is not the currently active document,
        // change focus to the closest tab
        if (
          O.getEq(string.Eq).equals(
            pane.activeView,
            O.some(payload.docPath)
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
