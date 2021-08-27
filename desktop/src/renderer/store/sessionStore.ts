import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'

export const SessionsStoreKeys = {
  PROJECT_PATH: 'PROJECT_PATH',
} as const

export const sessionStore = {
  get: (key: keyof typeof SessionsStoreKeys) =>
    pipe(key, () => window.sessionStorage.getItem(key), O.fromNullable),
  set: (key: keyof typeof SessionsStoreKeys) => (value: string) =>
    window.sessionStorage.setItem(key, value),
}
