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

  /**
   * Array of the parent node's ids
   */
  currentParentNodes?: string[]
}

export const infoviewContext = createContext<InfoViewContext>('infoview')
