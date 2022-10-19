import { html } from 'lit'
import { property } from 'lit/decorators'
import { TW } from 'twind'

import StencilaElement from '../utils/element'

/**
 * A base component to represent the `Entity` node type
 */
export default class StencilaEntity extends StencilaElement {
  /**
   * The id of the entity
   */
  @property()
  id: string

  /**
   * Render a 'tag' showing the type and id of the entity
   */
  protected renderTag(tw: TW, color: string = 'neutral') {
    return html`<stencila-tag color=${color}>${this.id}</stencila-tag>`
  }
}
