import { createContext } from '@lit/context'

/**
 * Context object to be used for the state of any Entity
 */
export interface EntityContext {
  nodeId?: string

  /**
   * Whether the entity's respective node card is open
   */
  cardOpen?: boolean
}

/**
 * Context object for state of Entity
 */
export const entityContext = createContext<EntityContext>('entity-node-card')

export interface InstructionContext extends EntityContext {
  childEntitys: string[]
}

export const instructionContext = createContext<InstructionContext>(
  'instruction-node-card'
)
