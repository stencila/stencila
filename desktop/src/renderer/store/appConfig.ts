import { createStore } from '@stencil/store'
import { Config } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { CombinedConfig } from '../../preload/types'
import { client } from '../client'

export const isErrorReportingEnabled = () =>
  client.config
    .getAll()
    .then(
      ({ value: config }) =>
        config.global.telemetry?.desktop?.error_reports ?? false
    )

export let configState: CombinedConfig

export const initConfigStore = async () =>
  await client.config.getAll().then(({ value: config }) => {
    configState = createStore<CombinedConfig>(config).state
  })

export const configEventListener = () => {
  window.api.receive(
    CHANNEL.CONFIG_SET,
    // TODO: Fix type signature
    // @ts-ignore
    <K extends keyof CombinedConfig>(event: { key: K; value: Config[K] }) => {
      configState[event.key] = event.value
    }
  )
}
