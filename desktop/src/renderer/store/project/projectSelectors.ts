import { RootState } from '..'

export const selectProject = (state: RootState) => state.projects.activeProject

export const selectProjectPath = (state: RootState) =>
  state.projects.activeProject?.path

export const selectProjectFiles = (state: RootState) => {
  const rootPath = state.projects.activeProject?.path
  const files = state.projects.activeProject?.files
  if (rootPath && files) {
    return files[rootPath]
  }
}

export const selectProjectFile = (state: RootState) => (filePath: string) => {
  const files = state.projects.activeProject?.files
  return files ? files[filePath] : undefined
}
