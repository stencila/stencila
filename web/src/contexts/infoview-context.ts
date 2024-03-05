import { createContext } from '@lit/context'

import { NodeId } from '../types'

export type InfoViewContext = {
  /**
   * Whether the info view is open or not
   */
  infoViewOpen: boolean

  /**
   * Id of the currently active node
   */
  currentNodeId?: NodeId
}

export const infoviewContext = createContext<InfoViewContext>('infoview')
