import { ipcMain } from 'electron'

export const removeChannelHandlers = (
  channelObject: Record<string, string>
) => {
  Object.keys(channelObject).map((channel) => {
    ipcMain.removeHandler(channel)
  })
}
