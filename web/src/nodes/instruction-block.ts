import { customElement } from 'lit/decorators.js'

import { Instruction } from './instruction'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
export class InstructionBlock extends Instruction {}
