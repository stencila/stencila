import { parse } from 'path'
import { dispatch, projects } from 'stencila'
import { projectHandlers } from '.'
import { CHANNEL } from '../../preload/channels'
import { captureError } from '../../preload/errors'
import { documentHandlers } from '../document'
import { createWindow } from '../window'
import { onUiLoaded } from '../window/windowUtils'
import { registerProjectMenu } from './menu'

const getProjectName = (path: string): string => parse(path).base

const projectUrl = '/project'

export const openProjectWindow = (
  directoryPath: string
): Electron.CrossProcessExports.BrowserWindow => {
  const projectWindow = createWindow(`${projectUrl}${directoryPath}`, {
    width: 800,
    height: 800,
    minWidth: 600,
    minHeight: 600,
    show: false,
    title: getProjectName(directoryPath),
  })

  // The ID needs to be stored separately from the window object. Otherwise an error
  // is thrown because the time remove handlers are called the window object is already destroyed.
  const windowId = projectWindow.id

  projectHandlers.register(windowId)
  documentHandlers.register(windowId)

  projectWindow.on('closed', () => {
    projects.close(directoryPath)

    projectHandlers.remove(windowId)
    documentHandlers.remove(windowId)
  })

  onUiLoaded(projectWindow.webContents)(() => {
    projectWindow.show()

    const serverUrl = dispatch.server.serve(directoryPath)
    projectWindow.webContents.send(CHANNEL.PROJECTS_SERVER_START, serverUrl)
  })

  projectWindow.on('focus', () => {
    registerProjectMenu()
  })

  projectWindow.loadURL(projectUrl).catch((err) => {
    captureError(err)
  })

  return projectWindow
}
