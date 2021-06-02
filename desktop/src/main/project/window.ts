import { parse } from 'path'
import { projects } from 'stencila'
import { createWindow } from '../../app/window'

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

  projectWindow.on('closed', () => {
    projects.close(directoryPath)
  })

  projectWindow.webContents.on('did-finish-load', () => {
    projectWindow?.show()
  })

  projectWindow?.loadURL(projectUrl)

  return projectWindow
}
