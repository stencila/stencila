import { Channel, Handler } from './index'

declare global {
  interface Window {
    api: IpcRendererAPI
  }
}

// type SELECT_DIRS = (channel: typeof CHANNEL.SELECT_DIRS) => Promise<number>
// type READ_DIR = (channel: typeof CHANNEL.READ_DIR, path: string) => Promise<string[]>
type INVOKE = (channel: Channel, ...args: unknown[]) => Promise<unknown>

type Invoke = INVOKE

export interface IpcRendererAPI {
  invoke: Invoke
  send(channel: Channel, ...args: unknown[]): void
  receive: (channel: Channel, func: Handler) => void
  remove: (channel: Channel, func: Handler) => void
  removeAll: (channel: Channel) => void

  /** @return A function that removes this listener. */
  // on(channel: string, listener: (...args: unknown[]) => void): () => void;
}
