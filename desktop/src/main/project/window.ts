import { BrowserWindow } from 'electron'
import { parse } from 'path'
import { createWindow } from '../../app/window'

const getProjectName = (path: string): string => parse(path).base

let projectWindow: BrowserWindow | null

const projectUrl = '/project'

export const openProjectWindow = (directoryPath: string) => {
  projectWindow = createWindow(`${projectUrl}${directoryPath}`, {
    width: 800,
    height: 800,
    minWidth: 600,
    minHeight: 600,
    show: false,
    title: getProjectName(directoryPath),
  })

  projectWindow.on('closed', () => {
    projectWindow = null
  })

  projectWindow.webContents.on('did-finish-load', () => {
    projectWindow?.show()
  })

  projectWindow?.loadURL(projectUrl)
}
