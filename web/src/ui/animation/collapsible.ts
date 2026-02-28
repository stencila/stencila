import { LitElement, css, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../twind'

/**
 * UI Collapsible Animation
 *
 * A wrapper component to constrain an element so we can successfully animate
 * in-and-out with natural disclosure motion.
 *
 * ## Animation Design
 *
 * Uses Material Design-inspired easing curves to create natural disclosure
 * behavior:
 *
 * **Opening (Expanding):**
 * - Height: `cubic-bezier(0.2, 0, 0, 1)` - "Emphasized decelerate" - eager to
 *   reveal content
 * - Opacity: Same curve and timing as height - synchronized reveal
 * - Timing: 600ms for both - content and space appear together
 *
 * **Closing (Collapsing):**
 * - Height: `cubic-bezier(0.4, 0, 0.6, 1)` - "Standard" - controlled,
 *   predictable collapse
 * - Opacity: Same curve and timing as height - synchronized fade
 * - Timing: 500ms for both - prevents empty space during collapse
 *
 * This asymmetric timing creates disclosure that feels "eager to open, graceful
 * to close".
 */
@customElement('stencila-ui-collapsible-animation')
@withTwind()
export class UINodeCollapsibleAnimation extends LitElement {
  static override styles = css`
    :host div {
      max-height: 0;
      overflow: hidden;
      opacity: 0;
      transition:
        max-height 500ms cubic-bezier(0.4, 0, 0.6, 1),
        opacity 500ms cubic-bezier(0.4, 0, 0.6, 1);
    }

    :host(.opened) div {
      max-height: 10000px;
      overflow: visible;
      opacity: 1;
      transition:
        max-height 600ms cubic-bezier(0.2, 0, 0, 1),
        opacity 600ms cubic-bezier(0.2, 0, 0, 1);
    }
  `

  override render() {
    return html`<div><slot></slot></div>`
  }
}
