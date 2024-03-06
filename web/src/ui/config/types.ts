import { Secret } from '../../types/api'

import { API_ICONS } from './icons'

/**
 * State that manages a secrets value from the API & keeps track of the
 * modified state.
 */
export type SecretState = {
  original: Secret
  modifiedValue?: string
}

/**
 * The name of each secret (mapped on to the API_ICONS - likely to change
 * to something more solid).
 */
export type SecretName = keyof typeof API_ICONS

/**
 * Status of saving the state of the current form.
 */
export type SavedState = 'idle' | 'saving' | 'done' | 'error'
