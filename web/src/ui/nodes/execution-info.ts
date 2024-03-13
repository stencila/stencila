import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Execution Info
 *
 * A wrapper block for displaying execution information within the details of a
 * block as displayed in the info view.
 */
@customElement('stencila-ui-execution-info')
@withTwind()
export class UIExecutionInfo extends LitElement {
  /**
   * The text & fill colour (for icons) used within an instance of this block.
   */
  @property()
  colour: string = 'black'

  override render() {
    return html`<div
      class="flex flex-row gap-x-3 w-full text-${this
        .colour} stencila-ui-execution-info"
    >
      <div class="pt-0.5 grow-0 shrink-0">
        <slot name="icon">Icon is missing</slot>
      </div>
      <div class="grow flex flex-col justify-center items-start">
        <slot name="content">Content<br />more here..</slot>
      </div>
    </div>`
  }
}
