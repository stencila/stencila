import i18next from 'i18next'
import { isDevelopment } from '../preload/utils/env'
import en from './en.json'

export const resources = {
  en,
} as const

i18next.init({
  debug: isDevelopment ?? false,
  initImmediate: false,
  lng: 'en',
  resources,
})

export const i18n = i18next
