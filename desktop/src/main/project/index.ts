import { dialog, ipcMain } from 'electron'
import fs from 'fs'
import { projects } from 'stencila'
import { CHANNEL } from '../../preload'
import { openProjectWindow, projectWindow } from './window'

export const registerProjectHandlers = () => {
  ipcMain.handle(
    CHANNEL.SHOW_PROJECT_WINDOW,
    async (_event, directoryPath: string) => {
      return openProjectWindow(directoryPath)
    }
  )

  ipcMain.handle(CHANNEL.SELECT_PROJECT_DIR, async () => {
    const { filePaths } = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory'],
    })
    const projectPath = filePaths[0]

    if (projectPath !== undefined) {
      openProjectWindow(projectPath)
    }
  })

  ipcMain.handle(
    CHANNEL.OPEN_PROJECT,
    async (_event, directoryPath: string) => {
      return openProjectWindow(directoryPath)
    }
  )

  ipcMain.handle(
    CHANNEL.GET_PROJECT_FILES,
    async (_event, directoryPath: string) => {
      return projects.open(directoryPath, (_topic, event) => {
        projectWindow?.webContents.send(CHANNEL.GET_PROJECT_FILES, event)
      })
    }
  )

  ipcMain.handle(
    CHANNEL.GET_DOCUMENT_CONTENTS,
    async (_event, filePath: string) => fs.readFileSync(filePath).toString()
  )
}
