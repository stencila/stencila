import { CHANNEL } from '../../preload/channels'
import { GetAppConfig, ReadAppConfig, SetAppConfig } from '../../preload/types'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { getAppConfig, readAppConfig, setAppConfig } from './handlers'

export const registerAppConfigStoreHandlers = () => {
  handle<ReadAppConfig>(CHANNEL.CONFIG_APP_READ, async () => {
    const config = readAppConfig()
    return valueToSuccessResult({ ...config })
  })

  // @ts-expect-error
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
