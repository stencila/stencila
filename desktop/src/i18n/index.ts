import i18next from 'i18next'
import en from './en.json'

export const resources = {
  en
} as const;

i18next.init({
  debug: process.env.NODE_ENV === 'development' ?? false,
  initImmediate: false,
  lng: 'en',
  resources
})

export const i18n = i18next
