import { NodeType } from '@stencila/types'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Instruction } from './instruction'

/**
 * Web component representing a Stencila Schema `InstructionInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-inline.md
 */
@customElement('stencila-instruction-inline')
@withTwind()
export class InstructionInline extends Instruction {
  override type: NodeType = 'InstructionInline'
}
