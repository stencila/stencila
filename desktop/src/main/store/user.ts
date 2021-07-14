import { v4 as uuidv4 } from 'uuid'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { getAppConfig, setAppConfig } from '../store/handlers'

/**
 * Assign the user a random, non-identifiable id.
 * This is used to group error reports over time.
 */
export const getOrAssignUserId = (): string => {
  const currentId = getAppConfig(UnprotectedStoreKeys.USER_ID)
  if (currentId && typeof currentId === 'string' && currentId !== '') {
    return currentId
  }

  const newId = uuidv4()
  setAppConfig(UnprotectedStoreKeys.USER_ID)(newId)
  return newId
}
