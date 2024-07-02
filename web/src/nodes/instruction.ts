import { NodeType } from '@stencila/types'
import { property } from 'lit/decorators.js'

import '../ui/nodes/card'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/instructions/instruction-messages'
import '../ui/nodes/properties/provenance/provenance'
import '../ui/nodes/properties/suggestion'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `Instruction` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md
 */
export abstract class Instruction extends Executable {
  protected type: NodeType

  @property({ type: Array })
  candidates?: string[]

  @property()
  assignee?: string
}
