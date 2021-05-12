import { CHANNEL } from '../../preload'
import { openProjectWindow } from './window'
import { dialog, ipcMain } from 'electron'
import { readdir } from 'fs/promises'

const getProjectFiles = async (directoryPath: string) => {
  const result = await readdir(directoryPath, {})
  return result
}

export const registerProjectHandlers = () => {
  ipcMain.handle(CHANNEL.SHOW_PROJECT_WINDOW, async (_event, directoryPath: string) => {
    return openProjectWindow(directoryPath)
  })

  ipcMain.handle(CHANNEL.SELECT_PROJECT_DIR, async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory'],
    })
    return result
  })

  ipcMain.handle(
    CHANNEL.GET_PROJECT_FILES,
    async (_event, directoryPath: string) => {
      const result = await getProjectFiles(directoryPath)
      return result
    }
  )
}
