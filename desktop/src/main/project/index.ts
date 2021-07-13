import { dispatch, projects } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  ProjectsOpen,
  ProjectsOpenUsingFilePicker,
  ProjectsWindowOpen,
} from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/rpc'
import { PROJECT_CHANNEL } from './channels'
import { openProject } from './handlers'
import { openProjectWindow } from './window'

export const registerProjectHandlers = () => {
  try {
    handle<ProjectsOpenUsingFilePicker>(
      CHANNEL.SELECT_PROJECT_DIR,
      async () => {
        return openProject().then(() => valueToSuccessResult())
      }
    )

    handle<ProjectsWindowOpen>(
      CHANNEL.OPEN_PROJECT_WINDOW,
      async (_event, directoryPath) => {
        openProjectWindow(directoryPath)
        return valueToSuccessResult()
      }
    )

    handle<ProjectsOpen>(
      CHANNEL.GET_PROJECT_FILES,
      async (ipcEvent, directoryPath) => {
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
