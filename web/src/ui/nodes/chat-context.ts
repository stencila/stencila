import { createContext } from '@lit/context'
import { InstructionType } from '@stencila/types'

/**
 * Context for the state of a `Chat` node
 *
 * Used to allow the UI of descendant nodes to change based
 * on the properties of the chat.
 */
export interface ChatContext {
  /**
   * The instruction type of the chat's prompt
   */
  instructionType?: InstructionType
}

export const chatContext = createContext<ChatContext>('chat-context')
