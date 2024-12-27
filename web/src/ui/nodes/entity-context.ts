import { createContext } from '@lit/context'

/**
 * Context for the state of an `Entity` node
 */
export interface EntityContext {
  /**
   * The id of the node
   */
  nodeId?: string

  /**
   * Whether the node's card is open
   */
  cardOpen?: boolean
}

export const entityContext = createContext<EntityContext>('entity-context')
