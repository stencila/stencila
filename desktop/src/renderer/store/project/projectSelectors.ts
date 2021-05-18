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

export const selectProjectFile = (state: RootState) => (filePath: string) => {
  const files = state.projects.entities.files
  return files ? files[filePath] : undefined
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
