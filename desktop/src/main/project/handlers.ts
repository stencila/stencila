import { dialog } from 'electron'
import { openProjectWindow } from './window'

export const openProject = async () => {
  const { filePaths } = await dialog.showOpenDialog({
    properties: ['openDirectory', 'createDirectory'],
  })
  const projectPath = filePaths[0]

  if (projectPath !== undefined) {
    openProjectWindow(projectPath)
  }
}
