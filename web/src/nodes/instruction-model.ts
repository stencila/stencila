import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `InstructionModel` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edit/instruction-model.md
 */
@customElement('stencila-instruction-model')
@withTwind()
export class InstructionModel extends Entity {
  override render() {
    return html`TODO: render model options`
  }
}
