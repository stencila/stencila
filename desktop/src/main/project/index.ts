import { dispatch, projects } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  ProjectsOpen,
  ProjectsOpenUsingFilePicker,
  ProjectsWindowOpen,
} from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { PROJECT_CHANNEL } from './channels'
import { openProject } from './handlers'
import { openProjectWindow } from './window'

export const registerProjectHandlers = () => {
  try {
    handle<ProjectsOpenUsingFilePicker>(
      CHANNEL.PROJECTS_OPEN_FROM_FILE_PICKER,
      async () => {
        return openProject().then(() => valueToSuccessResult())
      }
    )

    handle<ProjectsWindowOpen>(
      CHANNEL.PROJECTS_WINDOW_OPEN,
      async (_event, directoryPath) => {
        openProjectWindow(directoryPath)
        return valueToSuccessResult()
      }
    )

    handle<ProjectsOpen>(
      CHANNEL.PROJECTS_OPEN,
      async (ipcEvent, directoryPath) => {
        const result = dispatch.projects.open(directoryPath)
        if (result.ok) {
          projects.subscribe(
            result.value.path,
            ['files'],
            (_topic, fileEvent) => {
              ipcEvent.sender.send(CHANNEL.PROJECTS_OPEN, fileEvent)
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
