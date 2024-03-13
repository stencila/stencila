import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Execution Text
 *
 * A simple text element to display some label like content within an execution
 * info block - that is, text that is less than a paragraph in length.
 */
@customElement('stencila-ui-execution-text')
@withTwind()
export class UIExecutionText extends LitElement {
  @property({ attribute: 'text-size' })
  textSize: 'sm' | 'xs' | '2xs' = 'xs'

  override render() {
    return html`<div class=${`text-${this.textSize} leading-4`}>
      <slot></slot>
    </div>`
  }
}
