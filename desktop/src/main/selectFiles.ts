import { dialog, ipcMain } from 'electron'
import { readdir } from 'fs/promises'
import { CHANNEL } from '../preload'

export const registerFileHandlers = () => {
  ipcMain.handle(CHANNEL.SELECT_DIRS, async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory'],
    })
    return result
  })

  ipcMain.handle(CHANNEL.READ_DIR, async (_event, arg: string) => {
    const result = await readdir(arg, {})
    return result
  })
}
