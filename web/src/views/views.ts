import { DocumentView } from '../types'

export const VIEWS: Record<Exclude<DocumentView, 'directory'>, string> = {
  dynamic: 'Live updating and able to see and interact with code',
  interactive: 'Live updating and able to interact with parameters',
  live: 'Live updating view',
  static: 'Fixed, read-only view',
}
