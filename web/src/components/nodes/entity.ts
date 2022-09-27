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
   * Emit a patch with operations to replace each of the changed
   * properties
   *
   * Excludes protected and private property with a name starting
   * with an underscore and which are assumed to be used for
   * component state only.
   */
  protected async update(changedProperties: Map<string, unknown>) {
    super.update(changedProperties)

    const ops = Array.from(changedProperties.keys())
      .filter((name) => !name.startsWith('_'))
      .map((name) => ({
        type: 'Replace',
        address: [name],
        items: 1,
        length: 1,
        value: this[name],
      }))

    const patch = {
      target: this.id,
      ops,
    }

    return this.emit('stencila-document-patch', patch)
  }

  /**
   * Render a 'tag' showing the type and id of the entity
   */
  protected renderTag(tw: TW, color: string = 'neutral') {
    return html`<stencila-tag color=${color}>${this.id}</stencila-tag>`
  }
}
