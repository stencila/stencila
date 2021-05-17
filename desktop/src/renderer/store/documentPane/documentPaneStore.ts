import { createEntityAdapter, createSlice } from '@reduxjs/toolkit'

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
  },
})

export const documentPaneActions = documentPaneSlice.actions
export type DocumentPaneActions = typeof documentPaneActions
