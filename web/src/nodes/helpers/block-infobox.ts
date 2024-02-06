import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'
import { DocumentView, NodeType } from '../../types'
import {
  getNodeBorderColour,
  getNodeColour,
  getNodeIcon,
} from '../../ui/nodes/nodeMapping'
import '@shoelace-style/shoelace/dist/components/icon/icon'

/**
 * A component for displaying information about a `Block` node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-block-infobox')
@withTwind()
export class BlockInfobox extends LitElement {
  @property()
  icon: string = ''

  @property()
  currentNode: NodeType

  @property()
  view: DocumentView

  @state()
  private _hasAuthorsChildren: boolean = false

  @property()
  override title: string = ''

  override render() {
    const colour = getNodeColour(this.currentNode)
    const borderColour = getNodeBorderColour(this.currentNode)
    const icon = getNodeIcon(this.currentNode)
    const headerStyles = apply([
      'w-full',
      'p-4',
      `border border-[${borderColour}] rounded-t`,
      'font-medium',
    ])
    const bodyStyles = apply([
      'w-full',
      'p-4',
      `border border-[${borderColour}] rounded-b`,
    ])

    // TODO: design this
    return html`
      <div class=${headerStyles} style="background-color: ${borderColour}">
        <span class=${'items-center font-medium flex'} style="font-bold">
          <sl-icon name=${icon} library="stencila" class=${'pr-2'}></sl-icon>
          ${this.title}
        </span>
      </div>
      <div class=${bodyStyles} style="background-color: ${colour};">
        <div class=${this._hasAuthorsChildren ? `block` : 'hidden'}>
          <span class=${`items-center flex`}>
            <sl-icon
              name=${'authors'}
              library="stencila"
              class=${'pr-2'}
            ></sl-icon
            >Authors</span
          >
          <div
            class=${`border-b border-[${borderColour}] rounded-full my-2`}
          ></div>
          <slot name="authors"> </slot>
        </div>

        <slot name="items"></slot>
      </div>
    `
  }

  override updated() {
    const slot: HTMLSlotElement =
      this.shadowRoot.querySelector(`slot[name='authors']`)

    if (slot) {
      this._hasAuthorsChildren =
        slot.assignedElements({ flatten: true }).length !== 0
    }
  }
}
