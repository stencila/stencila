import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../../twind'
import { DocumentView } from '../../types'

/**
 * This type is used to quickly access any additional styles to add to a view's
 * rendering.
 */
type StyleMap = {
  [Prop in DocumentView]?: {
    outer: string | undefined
    inner: string | undefined
  }
}

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

  /**
   * Manages if/how we show the side panel slot.
   */
  @state()
  private _hasSide: boolean = true

  override render() {
    const classes: StyleMap = {
      live: this.displayModeClasses(),
      static: this.displayModeClasses(),
      dynamic: this.displayModeClasses(),
      source: this.sourceModeClasses(),
      split: this.splitModeClasses(),
      visual: this.displayModeClasses(),
    }

    const { inner, outer } = classes[this.view ?? 'live']

    return html` <div class="bg-white border border-grey-200 h-full ${outer}">
      <div class="flex flex-row h-full">
        <div class=${inner}><slot></slot></div>
        <div class="grow ${!this._hasSide ? 'hidden' : ''}">
          <slot name="side"></slot>
        </div>
      </div>
    </div>`
  }

  /**
   * Any "display" mode (live, fixed views etc) need to be able to scroll any
   * overflow, set a max width to the content & add sufficient padding between
   * the content and the chrome.
   *
   */
  private displayModeClasses() {
    return {
      inner: apply(['py-11 px-16', 'max-w-[65ch] lg:max-w-[120ch]']),
      outer:
        'overflow-y-scroll min-h-[calc(100vh-5rem)] max-h-[calc(100vh-5rem)]',
    }
  }

  /**
   * The source view needs to stretch the full height of the screen and have no
   * padding.
   */
  private sourceModeClasses() {
    const twClasses = apply(['p-0', 'w-full h-screen', 'overflow-y-hidden'])

    return {
      inner: `${twClasses}`,
      outer: '',
    }
  }

  /**
   * Split mode stretches the full screen and has no padding.
   */
  private splitModeClasses() {
    return {
      inner: apply(['p-0', 'w-full']),
      outer: `overflow-y-hidden`,
    }
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
