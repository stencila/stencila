import { parse } from 'path'
import { projects } from 'stencila'
import { projectHandlers } from '.'
import { documentHandlers } from '../document'
import { createWindow } from '../window'

const getProjectName = (path: string): string => parse(path).base

const projectUrl = '/project'

export const openProjectWindow = (directoryPath: string) => {
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

  projectWindow.webContents.on('did-finish-load', () => {
    projectWindow?.show()
  })

  projectWindow?.loadURL(projectUrl)

  return projectWindow
}
