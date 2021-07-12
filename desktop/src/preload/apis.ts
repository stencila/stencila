import { ipcRenderer } from 'electron'
import { IpcRendererAPI } from '../preload/types'
import { Channel, Handler, isChannel } from './channels'

export const apis: IpcRendererAPI = {
  // TODO: Refine type definitions to remove `@ts-ignore`
  // @ts-ignore
  invoke: (...args: [Channel, ...unknown]) => {
    const channel = args[0]
    if (isChannel(channel)) {
      return ipcRenderer.invoke.apply(null, args)
    }
    return Promise.reject(`Invalid channel ${channel}`)
  },
  send: (channel: Channel, data: unknown) => {
    if (isChannel(channel)) {
      ipcRenderer.send(channel, data)
    }
  },
  receive: (channel: Channel, func: Handler) => {
    if (isChannel(channel)) {
      // Deliberately strip event as it includes `sender`
      ipcRenderer.on(channel, (_event, ...args) => func(...args))
    }
  },
  remove: (channel: Channel, func: Handler) => {
    if (isChannel(channel)) {
      // Deliberately strip event as it includes `sender`
      ipcRenderer.removeListener(channel, (_event, ...args) => func(...args))
    }
  },
  removeAll: (channel: Channel) => {
    ipcRenderer.removeAllListeners(channel)
  },
}
