import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { DocumentView, NodeType } from '../../types'
import { getNodeIcon } from '../../ui/nodes/nodeMapping'

/**
 * A component for displaying information about a `Block` node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-block-executebox')
@withTwind()
export class BlockExecutableBox extends LitElement {
  @property()
  currentNode: NodeType

  @property()
  view: DocumentView

  @property()
  override title: string

  override render() {
    const icon = getNodeIcon(this.currentNode)
    const styles = apply([
      'w-full',
      'p-4',
      'bg-white',
      'rounded drop-shadow-2xl',
    ])

    return html`
      <div class=${styles}>
        <div class="mb-2">
          <sl-icon name=${icon} library="stencila"></sl-icon>
          ${this.title}
        </div>
        <slot name="authors"></slot>
        <slot name="messages"></slot>
        <slot name="suggestion"></slot>
      </div>
    `
  }
}
