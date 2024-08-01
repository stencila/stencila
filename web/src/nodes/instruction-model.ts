import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

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
  @property()
  namePattern?: string

  @property({ type: Number })
  qualityWeight?: number

  @property({ type: Number })
  speedWeight?: number

  @property({ type: Number })
  costWeight?: number

  @property({ type: Number })
  minimumScore?: number

  @property({ type: Number })
  temperature?: number

  override render() {
    // TODO: render properties as <input>s (of appropriate type)
    // Note that all the numbers have the range 0 to 100.
    return html``
  }
}
