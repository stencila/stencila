import { v4 as uuidv4 } from 'uuid'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { getConfig, setConfig } from './handlers'

/**
 * Assign the user a random, non-identifiable id.
 * This is used to group error reports over time.
 */
export const getOrAssignUserId = (): string => {
  const currentId = getConfig().app[UnprotectedStoreKeys.USER_ID]

  if (currentId && typeof currentId === 'string' && currentId !== '') {
    return currentId
  }

  const newId = uuidv4()
  setConfig(UnprotectedStoreKeys.USER_ID, newId)
  return newId
}
