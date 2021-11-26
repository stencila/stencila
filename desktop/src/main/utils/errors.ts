import { setUser } from '../../preload/errors'
import { getOrAssignUserId } from '../config/user'

export const setErrorReportingId = () => {
  setUser(getOrAssignUserId())
}
