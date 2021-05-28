import {
  createEntityAdapter,
  createSlice,
  EntityId,
  PayloadAction,
} from '@reduxjs/toolkit'
import { array as A, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'

type DocumentPane = {
  id: string
  documents: string[]
  activeDocument?: string
}

const documentPaneAdapter = createEntityAdapter<DocumentPane>()

export const documentPaneSlice = createSlice({
  name: 'documentPanes',
  initialState: documentPaneAdapter.getInitialState(),
  reducers: {
    createPane: documentPaneAdapter.addOne,
    updatePane: documentPaneAdapter.updateOne,
    addDocToPane: (
      state,
      { payload }: PayloadAction<{ paneId: EntityId; docPath: string }>
    ) => {
      const pane = state.entities[payload.paneId]
      if (pane) {
        if (!pane.documents.includes(payload.docPath)) {
          pane.documents = [...pane.documents, payload.docPath]
        }
        pane.activeDocument = payload.docPath
      }
      return state
    },
    removeDocFromPane: (
      state,
      { payload }: PayloadAction<{ paneId: EntityId; docPath: string }>
    ) => {
      const pane = state.entities[payload.paneId]

      if (pane) {
        const docIndex = pane.documents.indexOf(payload.docPath)

        // Remove document from list
        if (pane.documents.includes(payload.docPath)) {
          pane.documents = pipe(
            pane.documents,
            A.deleteAt(docIndex),
            O.getOrElse<string[]>(() => [])
          )
        }

        // If document being closed is not the currently active document,
        // change focus to the next closest tab
        if (pane.activeDocument === payload.docPath) {
          pane.activeDocument =
            pane.documents[docIndex - 1] ??
            pane.documents[docIndex + 1] ??
            undefined
        }
      }

      return state
    },
  },
})

export const documentPaneActions = documentPaneSlice.actions
export type DocumentPaneActions = typeof documentPaneActions
