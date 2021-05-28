import {
  createEntityAdapter,
  createSlice,
  EntityId,
  PayloadAction
} from '@reduxjs/toolkit'
import { array as A, option as O, string } from 'fp-ts'
import { pipe } from 'fp-ts/function'

type DocumentPane = {
  id: string
  documents: string[]
  activeDocument: O.Option<string>
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
        pane.activeDocument = O.some(payload.docPath)
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
        // change focus to the closest tab
        if (
          O.getEq(string.Eq).equals(
            pane.activeDocument,
            O.some(payload.docPath)
          )
        ) {
          pane.activeDocument = pipe(
            A.lookup(docIndex)(pane.documents),
            O.alt(() => A.lookup(docIndex - 1)(pane.documents))
          )
        }
      }

      return state
    },
  },
})

export const documentPaneActions = documentPaneSlice.actions
export type DocumentPaneActions = typeof documentPaneActions
