import { CHANNEL } from '../../preload/channels'
import { getFocusedWindow } from '../window/windowUtils'

export const createNewDocument = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENTS_CREATE)
}

export const saveActiveDoc = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENT_WRITE_ACTIVE)
}

export const saveActiveDocAs = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENT_WRITE_ACTIVE_AS)
}

export const closeActiveTab = async () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENTS_CLOSE_ACTIVE)
}

export const cycleToNextTab = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.TABS_NEXT)
}

export const cycleToPreviousTab = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.TABS_PREVIOUS)
}
