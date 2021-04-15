export const CHANNEL = {
  TO_MAIN: 'TO_MAIN',
  SELECT_DIRS: 'SELECT_DIRS',
  READ_DIR: 'READ_DIR',
  READ_DIR_RESULTS: 'READ_DIR_RESULTS',
} as const

export type Channel = keyof typeof CHANNEL

export type Handler = (...args: unknown[]) => void

export const isChannel = (maybeChannel: string): maybeChannel is Channel => {
  return Object.keys(CHANNEL).includes(maybeChannel)
}
