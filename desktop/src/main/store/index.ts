import { CHANNEL } from '../../preload/channels'
import { GetAppConfig, ReadAppConfig, SetAppConfig } from '../../preload/types'
import { handle, valueToSuccessResult } from '../utils/rpc'
import { getAppConfig, readAppConfig, setAppConfig } from './handlers'

export const registerAppConfigStoreHandlers = () => {
  handle<ReadAppConfig>(CHANNEL.READ_APP_CONFIG, async () => {
    const config = readAppConfig()
    return valueToSuccessResult({ ...config })
  })

  // @ts-expect-error
  handle<GetAppConfig>(CHANNEL.GET_APP_CONFIG, async (_event, key) => {
    return valueToSuccessResult(getAppConfig(key))
  })

  handle<SetAppConfig>(
    CHANNEL.SET_APP_CONFIG,
    async (_event, { key, value }) => {
      return valueToSuccessResult(setAppConfig(key)(value))
    }
  )
}
