import { ipcMain } from 'electron'

/**
 * A utility function for managing the registration and cleanup of IPC handler.
 * Attempting to register a handler on the same channel multiple times will throw an error.
 * Additionally, several windows can depend on a handler, so the handlers should only be
 * cleaned up if no windows require it.

 * For global handlers that need to be present regardless of a window (e.g. global
 * keyboard shortcut listeners), you can pass `null` as the window id.
 */
export const makeHandlers = (registerFn: () => void, removeFn: () => void) => {
  const attachedWindows = new Set<number | null>()

  return {
    register: (windowId: number | null): void => {
      if (attachedWindows.size <= 0) {
        registerFn()
      }
      attachedWindows.add(windowId)
    },
    remove: (windowId: number | null): void => {
      attachedWindows.delete(windowId)
      if (attachedWindows.size <= 0) {
        removeFn()
      }
    },
  }
}

export const removeChannelHandlers = (
  channelObject: Record<string, string>
) => {
  Object.keys(channelObject).map((channel) => {
    ipcMain.removeHandler(channel)
  })
}
