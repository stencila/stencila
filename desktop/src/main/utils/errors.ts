import { setUser } from '../../preload/errors'
import { getOrAssignUserId } from '../store/user'

export const setErrorReportingId = () => {
  setUser(getOrAssignUserId())
}
