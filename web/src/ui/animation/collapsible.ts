import { LitElement, css, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Collapsible Animation
 *
 * A wrapper component to constrain an element so we can successfully animate
 * in-and-out.
 */
@customElement('stencila-ui-collapsible-animation')
@withTwind()
export class UINodeCollapsibleAnimation extends LitElement {
  static override styles = css`
    :host div {
      overflow: hidden;
      opacity: 0;
      max-height: 0;
      transition:
        max-height 400ms cubic-bezier(0, 1, 0, 1),
        opacity 800ms;
      transform: translate3d(0, 0, 0);
    }

    :host(.opened) div {
      max-height: 10000px;
      opacity: 1;
      transition:
        max-height 700ms ease-in-out,
        opacity 400ms;
      transform: translate3d(0, 0, 0);
      overflow: visible;
    }
  `

  override render() {
    return html`<div><slot></slot></div>`
  }
}
