import { CHANNEL } from '../../preload/channels'
import { GetAppConfig, ReadAppConfig, SetAppConfig } from '../../preload/types'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { UNPROTECTED_STORE_CHANNEL } from './channels'
import { getAppConfig, readAppConfig, setAppConfig } from './handlers'

const registerAppConfigStoreHandlers = () => {
  handle<ReadAppConfig>(CHANNEL.CONFIG_APP_READ, async () => {
    const config = readAppConfig()
    return valueToSuccessResult({ ...config })
  })

  handle<GetAppConfig>(CHANNEL.CONFIG_APP_GET, async (_event, key) => {
    return valueToSuccessResult(getAppConfig(key))
  })

  handle<SetAppConfig>(
    CHANNEL.CONFIG_APP_SET,
    async (_event, { key, value }) => {
      return valueToSuccessResult(setAppConfig(key)(value))
    }
  )
}

const removeAppConfigStoreHandlers = () => {
  removeChannelHandlers(UNPROTECTED_STORE_CHANNEL)
}

export const appStoreHandlers = makeHandlers(
  registerAppConfigStoreHandlers,
  removeAppConfigStoreHandlers
)
