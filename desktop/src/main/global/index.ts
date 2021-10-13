import { app, shell } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { GetAppVersion, OpenLink } from '../../preload/types'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { GLOBAL_CHANNEL } from './channels'

const registerGlobalHandlers = () => {
  handle<OpenLink>(CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER, (_event, link) =>
    shell.openExternal(link).then(() => valueToSuccessResult())
  )

  handle<GetAppVersion>(CHANNEL.GET_APP_VERSION, async (_event) => {
    return valueToSuccessResult(app.getVersion())
  })
}

const removeGlobalHandlers = () => {
  removeChannelHandlers(GLOBAL_CHANNEL)
}

export const globalHandlers = makeHandlers(
  registerGlobalHandlers,
  removeGlobalHandlers
)
