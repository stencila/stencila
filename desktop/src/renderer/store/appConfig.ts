import { createStore } from '@stencil/store'
import { CHANNEL } from '../../preload/channels'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { AppConfigStore } from '../../preload/types'
import { client } from '../client'

export const isErrorReportingEnabled = () =>
  client.config.ui
    .get(UnprotectedStoreKeys.REPORT_ERRORS)
    .then(({ value: isEnabled }) => isEnabled ?? false)

export let configState: AppConfigStore

export const initConfigStore = async () =>
  await client.config.ui.getAll().then(({ value: config }) => {
    configState = createStore<AppConfigStore>(config).state
  })

export const configEventListener = () => {
  window.api.receive(
    CHANNEL.CONFIG_APP_SET,
    // @ts-ignore
    <K extends UnprotectedStoreKeys>(event: {
      key: K
      value: AppConfigStore[K]
    }) => {
      configState[event.key] = event.value
    }
  )
}
