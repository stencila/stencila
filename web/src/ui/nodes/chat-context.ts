import { createContext } from '@lit/context'
import { InstructionType } from '@stencila/types'

import { NodeId } from '../../types'

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

  /**
   * The id of the node that is the source of the chat's `target`
   */
  source?: NodeId
}

export const chatContext = createContext<ChatContext>('chat-context')
