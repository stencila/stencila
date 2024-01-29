import { apply, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../../twind'
import { DocumentView } from '../../types'

/**
 * UI View Container
 *
 * The outer container that wraps the currently displayed view. Based on the
 * current view, will change the layout via css as needed.
 */
@customElement('stencila-ui-view-container')
@withTwind()
export class UIViewContainer extends LitElement {
  /**
   * The view we are rendering
   */
  @property()
  view: DocumentView = 'live'

  @state()
  private _hasSide: boolean = true

  override render() {
    const classes: {
      [Prop in DocumentView]: string | undefined
    } = {
      live: this.displayModeClasses(),
      static: this.displayModeClasses(),
      dynamic: this.displayModeClasses(),
      source: undefined,
      split: this.displayModeClasses(),
      visual: this.displayModeClasses(),
    }

    return html`<div class="flex flex-row">
      <div class=${classes[this.view ?? 'live']}><slot></slot></div>
      <div class="grow ${!this._hasSide ? 'hidden' : ''}">
        <slot name="side"></slot>
      </div>
    </div>`
  }

  displayModeClasses() {
    const styles = css``

    const twClasses = apply(['py-11 px-16', 'max-w-[65ch] lg:max-w-[120ch]'])

    return `${twClasses} ${styles}`
  }

  /**
   * If a side slot has not been defined, we set the `_hasSide` state to false
   * and hide the element in css.
   */
  override updated() {
    const slot = this.shadowRoot.querySelectorAll('slot')

    if (slot.length > 0 && slot[1].name === 'side') {
      this._hasSide = slot[1].assignedElements({ flatten: true }).length !== 0
    }
  }
}
