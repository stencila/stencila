import { EntityId } from '@reduxjs/toolkit'
import { RootState } from '..'

export const selectProject = (state: RootState) => {
  const id = state.projects.ids[0]
  if (id) {
    return state.projects.entities.projects[id]?.path
  }
}

export const selectProjectPath = (state: RootState) => {
  const id = state.projects.ids[0]
  if (id) {
    return state.projects.entities.projects[id]?.path
  }
}

export const selectProjectFiles = (state: RootState) => {
  const rootPath = selectProjectPath(state)
  if (rootPath) {
    const project = state.projects.entities.projects[rootPath]
    return project?.files
  }
}

export const selectProjectFile = (state: RootState) => (docId: EntityId) => {
  const files = state.projects.entities.files
  return files ? files[docId] : undefined
}

export const selectProjectRootFiles = (state: RootState) => {
  const rootPath = selectProjectPath(state)
  if (rootPath) {
    const projectFiles = selectProjectFile(state)(rootPath)

    if (projectFiles) {
      return projectFiles.children
    }
  }
}

export const getProjectTheme = (state: RootState): string => {
  const id = state.projects.ids[0] ?? ''
  const project = state.projects.entities.projects[id]
  return project?.theme ?? 'stencila'
}

export const getProjectMainFilePath = (
  state: RootState
): string | undefined => {
  const id = state.projects.ids[0] ?? ''
  return state.projects.entities.projects[id]?.mainPath
}
