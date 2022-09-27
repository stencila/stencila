import { property } from 'lit/decorators'

import StencilaElement from '../utils/element'

/**
 * A base component to represent the `Entity` node type
 */
export default class StencilaEntity extends StencilaElement {
  @property()
  id: string

  /**
   * Emit a patch with operations to replace each of the changed
   * properties
   */
  protected async update(changedProperties: Map<string, unknown>) {
    super.update(changedProperties)

    const ops = Array.from(changedProperties.keys()).map((name) => ({
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
}
