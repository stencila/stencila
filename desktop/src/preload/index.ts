import { contextBridge, ipcRenderer } from 'electron'
import { apis } from './apis'
import { CHANNEL } from './channels'
import { enableCrashReports } from './errors'

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('api', { ...apis })

// This function needs to be able to run in both the `preload` and `web` contexts,
// therefore it cannot rely on NodeJS apis.
const isReportErrorsEnabled = () =>
  ipcRenderer.invoke(CHANNEL.GET_APP_CONFIG, 'REPORT_ERRORS')

enableCrashReports(isReportErrorsEnabled)
