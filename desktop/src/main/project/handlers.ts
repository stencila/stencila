import { dialog, OpenDialogOptions } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { onUiLoaded } from '../window/windowUtils'
import { openProjectWindow } from './window'

/**
 * Open native system file browser from which user can navigate to directory they want
 * to open as a project.
 */
export const openProject = async (options: Partial<OpenDialogOptions> = {}) => {
  const { filePaths } = await dialog.showOpenDialog({
    properties: ['openDirectory', 'createDirectory'],
    ...options,
  })

  const projectPath = filePaths[0]

  if (projectPath !== undefined) {
    return openProjectWindow(projectPath)
  }
}

/**
 * Currently this function is almost identical to the `openProject` command,
 * however it automatically opens a new untitled file in the editor panel once the project window is ready.
 * TODO: Have custom dialogue window allowing advanced features such as selecting a project template,
 * bootstrapping a project based on GitHub repo or other file sources.
 */
export const newProject = async () => {
  openProject().then((win) => {
    const createDoc = () => {
      win?.webContents.send(CHANNEL.DOCUMENTS_CREATE)
      win?.webContents.removeListener('ipc-message', createDoc)
    }

    onUiLoaded(win?.webContents)(createDoc)
  })
}
