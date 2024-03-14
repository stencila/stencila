import '@shoelace-style/shoelace/dist/components/icon/icon'
import type { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { DocumentView } from '../../types'

import { nodeUi } from './icons-and-colours'

/**
 * A component for displaying information about a node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-ui-node-card')
@withTwind()
export class UINodeCard extends LitElement {
  /**
   * The type of node that this card is for
   *
   * Used to determine the title, icon and colors of the card.
   */
  @property()
  type: NodeType

  /**
   * The view that this card is within
   *
   * Used for adapting styling for the view.
   */
  @property()
  view: DocumentView

  override render() {
    const { iconLibrary, icon, title, colour, borderColour } = nodeUi(this.type)

    const cardStyles = this.view === 'source' ? 'flex flex-col h-full' : 'my-2'

    const headerStyles = apply([
      'flex justify-between items-center',
      'w-full',
      'p-4',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}] ${this.view === 'source' ? '' : 'rounded-t'}`,
      'font-medium',
    ])

    const bodyStyles = apply([
      'w-full h-full',
      'p-4',
      `bg-[${colour}]`,
      `border border-[${borderColour}] rounded-b`,
    ])

    return html` <div class=${cardStyles}>
      <div class=${headerStyles}>
        <span class="items-center font-bold flex">
          <sl-icon library=${iconLibrary} name=${icon} class="pr-2"></sl-icon>
          ${title}
        </span>
        <span class="items-center font-bold flex">
          <slot name="header-right"></slot>
        </span>
      </div>
      <div class=${bodyStyles}>
        <slot name="body"></slot>
      </div>
    </div>`
  }
}

// TODO: delete these
export const nodeCardParentStyles = (view: DocumentView) =>
  view !== 'source' ? 'group relative' : ''

export const nodeCardStyles = (view: DocumentView) =>
  view !== 'source'
    ? 'absolute z-10 top-full right-0 group-hover:block'
    : 'flex flex-col h-full'
