import { property } from 'lit/decorators.js'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `Instruction` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md
 */
export abstract class Instruction extends Executable {
  @property({ attribute: 'instruction-type' })
  instructionType: string

  @property()
  prompt?: string

  @property({ type: Number })
  replicates?: number

  @property({ attribute: 'active-suggestion', type: Number })
  activeSuggestion?: number
}
