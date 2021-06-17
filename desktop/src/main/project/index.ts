import { ipcMain } from 'electron'
import { projects } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { PROJECT_CHANNEL } from './channels'
import { openProject } from './handlers'
import { openProjectWindow } from './window'

export const registerProjectHandlers = () => {
  try {
    ipcMain.handle(
      CHANNEL.SHOW_PROJECT_WINDOW,
      async (_event, directoryPath: string) => {
        openProjectWindow(directoryPath)
      }
    )

    ipcMain.handle(CHANNEL.SELECT_PROJECT_DIR, async () => {
      openProject()
    })

    ipcMain.handle(
      CHANNEL.OPEN_PROJECT,
      async (_event, directoryPath: string) => {
        openProjectWindow(directoryPath)
      }
    )

    ipcMain.handle(
      CHANNEL.GET_PROJECT_FILES,
      async (ipcEvent, directoryPath: string) => {
        const project = projects.open(directoryPath)
        projects.subscribe(project.path, ['files'], (_topic, fileEvent) => {
          ipcEvent.sender.send(CHANNEL.GET_PROJECT_FILES, fileEvent)
        })
        return project
      }
    )
  } catch {
    // Handlers likely already registered
  }
}

export const removeProjectHandlers = () => {
  removeChannelHandlers(PROJECT_CHANNEL)
}
