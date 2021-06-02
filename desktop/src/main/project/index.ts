import { dialog, ipcMain } from 'electron'
import { projects } from 'stencila'
import { CHANNEL } from '../../preload'
import { openProjectWindow } from './window'

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
    async (ipcEvent, directoryPath: string) => {
      return projects.open(directoryPath, (_topic, projectEvent) => {
        ipcEvent.sender.send(CHANNEL.GET_PROJECT_FILES, projectEvent)
      })
    }
  )
}
