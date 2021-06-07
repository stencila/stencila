import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { File } from '../../types'
import { fetchProject } from './projectActions'
import { NormalizedProjectStore } from './ProjectStoreTypes'

const initialState: NormalizedProjectStore = {
  entities: {
    projects: {},
    files: {},
  },
  ids: [],
}

export const projectSlice = createSlice({
  name: 'projects',
  initialState,
  reducers: {
    updateProjectFiles: (
      state,
      { payload }: PayloadAction<Record<string, File>>
    ) => {
      state.entities.files = payload
    },
  },
  extraReducers: (builder) => {
    builder.addCase(fetchProject.fulfilled, (state, { payload }) => {
      state.entities = payload.entities
      state.ids =
        typeof payload.result === 'string' ? [payload.result] : payload.result
    })
  },
})

export const projectActions = projectSlice.actions
export type ProjectActions = typeof projectActions
