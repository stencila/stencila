import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/inline-on-demand'

/**
 * Web component representing a Stencila Schema `WalkthroughStep` node
 *
 * This component currently only exists to turn on/off visibility of the
 * content of a walkthrough step (based on `isCollapsed`).
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/walkthrough-step.md
 */
@customElement('stencila-walkthrough-step')
@withTwind()
export class WalkthroughStep extends LitElement {
  @property({ attribute: 'is-collapsed' })
  isCollapsed?: string

  override render() {
    const styles = apply(
      'overflow-hidden',
      'transition-all duration-1000 ease-in-out',
      this.isCollapsed == 'true'
        ? 'max-h-0 opacity-0'
        : 'max-h-screen opacity-100'
    )

    return html`<div class="${styles}">
      <slot name="content"></slot>
    </div>`
  }
}
