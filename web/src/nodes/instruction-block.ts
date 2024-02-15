import { NodeType } from '@stencila/types'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Instruction } from './instruction'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
@withTwind()
export class InstructionBlock extends Instruction {
  override type: NodeType = 'InstructionBlock'
}
