import '@shoelace-style/shoelace/dist/components/icon/icon'
import type { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { DocumentView } from '../../types'

import { nodeUi } from './node-ui'

/**
 * A component for displaying information about a node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-node-card')
@withTwind()
export class NodeCard extends LitElement {
  /**
   * The type of node that this card is for
   *
   * Used to determine the title, icon and colors of the card.
   */
  @property()
  type: NodeType

  override render() {
    const { iconLibrary, icon, title, colour, borderColour } = nodeUi(this.type)

    const headerStyles = apply([
      'flex justify-between items-center',
      'w-full',
      'p-4',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}] rounded-t`,
      'font-medium',
    ])

    const bodyStyles = apply([
      'w-full h-full',
      'p-4',
      `bg-[${colour}]`,
      `border border-[${borderColour}] rounded-b`,
    ])

    return html`
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
    `
  }
}

/**
 * Generate the Tailwind classes for the parent element of a `<stencila-node-card>` element
 *
 * @param view The view that the card is currently being rendered in
 */
export const nodeCardParentStyles = (view: DocumentView) =>
  view !== 'source' ? 'group relative' : ''

/**
 * Generate the Tailwind classes for a `<stencila-node-card>` element
 *
 * @param view The view that the card is currently being rendered in
 */
export const nodeCardStyles = (view: DocumentView) =>
  view !== 'source'
    ? 'hidden absolute z-10 top-full right-0 group-hover:block'
    : 'flex flex-col h-full'
