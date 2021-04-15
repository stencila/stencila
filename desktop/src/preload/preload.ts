import { contextBridge, ipcRenderer } from 'electron'
import { Channel, Handler, isChannel } from '../preload'
import { IpcRendererAPI } from '../preload/types'
import { rendererStore } from './store'

const apis: IpcRendererAPI = {
  store: rendererStore,
  invoke: (channel, data) => {
    if (isChannel(channel)) {
      return ipcRenderer.invoke(channel, data)
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
}

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('api', apis)
