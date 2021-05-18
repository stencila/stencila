import { createSlice, EntityId } from '@reduxjs/toolkit'
import { File } from '../../types'
import { NormalizedProject } from './entities'
import { fetchProject } from './projectActions'

export type ProjectStoreEntities = {
  projects: Record<EntityId, NormalizedProject | undefined>
  files: Record<EntityId, File | undefined>
}

export type NormalizedProjectStore = {
  entities: ProjectStoreEntities
  ids: string[]
}

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
  reducers: {},
  extraReducers: (builder) => {
    builder.addCase(fetchProject.fulfilled, (state, { payload }) => {
      state = {
        ...state,
        entities: {
          projects: payload.entities.projects,
          files: payload.entities.files,
        },
        ids:
          typeof payload.result === 'string'
            ? [payload.result]
            : payload.result,
      }
      return state
    })
  },
})

export const projectActions = projectSlice.actions
export type ProjectActions = typeof projectActions
