import { customElement } from 'lit/decorators.js'

import { Instruction } from './instruction'

/**
 * Web component representing a Stencila Schema `InstructionInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-inline.md
 */
@customElement('stencila-instruction-inline')
export class InstructionInline extends Instruction {}
