import { CONFIG_CHANNEL } from '../main/config/channels'
import { PROJECT_CHANNEL } from '../main/project/channels'

export const CHANNEL = {
  TO_MAIN: 'TO_MAIN',
  ...CONFIG_CHANNEL,
  ...PROJECT_CHANNEL,
} as const

export type Channel = keyof typeof CHANNEL

export type Handler = (...args: unknown[]) => void

export const isChannel = (maybeChannel: string): maybeChannel is Channel => {
  return Object.keys(CHANNEL).includes(maybeChannel)
}
