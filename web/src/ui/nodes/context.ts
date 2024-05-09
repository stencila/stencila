import { createContext } from '@lit/context'

/**
 * Context object to be used for the state of any Entity
 */
export type InstructionContext = {
  readonly nodeId?: string

  /**
   * Whether the entity's respective node card is open
   */
  cardOpen?: boolean
}

/**
 * Context object for state of Entity
 */
export const instructionContext = createContext<InstructionContext>('node-card')
