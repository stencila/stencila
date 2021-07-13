import { ipcMain } from 'electron'
import { dispatch, projects } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { valueToSuccessResult } from '../utils/rpc'
import { PROJECT_CHANNEL } from './channels'
import { openProject } from './handlers'
import { openProjectWindow } from './window'

export const registerProjectHandlers = () => {
  try {
    ipcMain.handle(CHANNEL.SELECT_PROJECT_DIR, async () => {
      openProject()
    })

    ipcMain.handle(
      CHANNEL.OPEN_PROJECT,
      async (_event, directoryPath: string) => {
        openProjectWindow(directoryPath)
        return valueToSuccessResult()
      }
    )

    ipcMain.handle(
      CHANNEL.GET_PROJECT_FILES,
      async (ipcEvent, directoryPath: string) => {
        const result = dispatch.projects.open(directoryPath)
        if (result.ok) {
          projects.subscribe(
            result.value.path,
            ['files'],
            (_topic, fileEvent) => {
              ipcEvent.sender.send(CHANNEL.GET_PROJECT_FILES, fileEvent)
            }
          )
        }

        return result
      }
    )
  } catch {
    // Handlers likely already registered
  }
}

export const removeProjectHandlers = () => {
  removeChannelHandlers(PROJECT_CHANNEL)
}
