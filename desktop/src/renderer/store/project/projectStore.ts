import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { Project } from '../../types'

type ProjectState = {
  activeProject?: Project
  panes: []
}

const initialState: ProjectState = {
  panes: [],
}

type SetProjectAction = PayloadAction<{
  project: Project
}>

// TODO: Think about how to handle multiple Project windows
export const projectSlice = createSlice({
  name: 'project',
  initialState,
  reducers: {
    insert: (state, action: SetProjectAction) => {
      state.activeProject = action.payload.project
    },
  },
})

export const projectActions = projectSlice.actions
export type ProjectActions = typeof projectActions
