import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

type CardContentListItem = {
  icon?: string
  text: string
}

/**
 * Node details card
 *
 * Displays a piece of information in a node card. This content will include
 * an icon or avatar on one side, plus textual information on the other side.
 *
 * The text information can
 */

@customElement('stencila-ui-node-detail-card')
@withTwind()
export class NodeDetailCard extends LitElement {
  /**
   * A label to display in small text above the title for the card
   * - e.g. Author Role.
   */
  @property()
  label: string | undefined

  /**
   * The content to render (below the title). This can be a string or a list.
   */
  @property()
  content: string | CardContentListItem[]

  /**
   * The colour used by text & as a fill for icons. This may be something is
   * retrieved from the `nodeColour` function.
   */
  @property()
  colour: string = 'black'

  override render() {
    return html`
      <div class="flex flex-row gap-x-2 w-full mb-4 text-${this.colour}">
        <div class="pt-0.5 basis-6">Icon or avatar</div>
        <div class="grow ml-4">
          <div class="grid grid-cols-5">
            <div class="flex flex-col col-span-4">
              ${this.renderLabel()}
              <span class="text-sm leading-tight col-span-2"
                >${this.title}</span
              >
              ${this.renderContent()}
            </div>
            ${this.renderSidebar()}
          </div>
        </div>
      </div>
    `
  }

  private renderContentAsList(list: CardContentListItem[]) {
    return html`<ul class="py-1 pl-3">
      ${list.map(({ icon, text }) => {
        return html`<li class="text-xs flex ${icon ? 'gap-x-2' : ''}">
          <sl-icon
            name=${icon}
            library="stencila"
            class="text-xxs mt-1 fill-${this.colour} ${icon ? '' : 'hidden'}"
          ></sl-icon
          >${text}
        </li>`
      })}
    </ul>`
  }

  private renderContent() {
    return this.isContentArray(this.content)
      ? this.renderContentAsList(this.content)
      : html`<span class="text-xs leading-tight">${this.content}</span>`
  }

  private renderLabel() {
    return (
      this.label &&
      html`<span class="text-xxs leading-tight">${this.label}</span>`
    )
  }

  private renderSidebar() {
    const classes = apply([
      this.label && 'mt-4',
      'col-span-1',
      'text-[8px] text-right leading-tight',
      'flex justify-self-center',
    ])
    return html`<div class=${classes}>
      <slot name="sidebar"></slot>
    </div>`
  }

  private isContentArray(
    content: typeof this.content
  ): content is CardContentListItem[] {
    return Array.isArray(content)
  }
}
