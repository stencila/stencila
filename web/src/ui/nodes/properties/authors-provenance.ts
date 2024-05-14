import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { withTwind } from '../../../twind'

import '../../animation/collapsible'

/**
 * UI Authors and Provenance
 *
 * A collapsible section displayed at the top of the page to show authors &
 * provenance.
 */
@customElement('stencila-ui-authors-provenance')
@withTwind()
export class UIAuthorsProvenance extends LitElement {
  @state()
  collapsed: boolean = true

  protected override render() {
    return html`
      <div
        @click=${() => {
          this.collapsed = !this.collapsed
        }}
        class=${`${!this.collapsed ? 'mb-8' : 'mb-0'} transition-[margin] duration-200 ease-in`}
      >
        ${this.renderHeader()}
        <stencila-ui-collapsible-animation
          class=${!this.collapsed ? 'opened' : ''}
        >
          <div
            class="p-4 border border-black/20 rounded-tl-none rounded-b rounded-tr"
          >
            <slot></slot>
          </div>
        </stencila-ui-collapsible-animation>
      </div>
    `
  }

  /**
   * Output the header element which acts as a event handler for click events.
   */
  private renderHeader() {
    const collapsedClasses = this.collapsed
      ? ['text-black/50']
      : [
          'border-l border-l-black/20 border-r border-r-black/20 rounded-t',
          'hover:bg-[rgba(0,0,0,0.025)] hover:contrast-[105%]',
          'text-black',
        ]
    const containerClasses = apply([
      ...collapsedClasses,
      [
        'relative z-[1]',
        'flex gap-x-2 items-center',
        'border-t border-t-black/20 border-b border-b-white',
        'p-2 -mb-px',
        'max-w-fit',
        'transition-all',
        'cursor-pointer',
        'after:content-[""] after:block after:absolute after:w-[calc(100%-2px)] after:h-px after:bg-white after:bottom-0 after:left-[1px]',
      ],
    ])

    return html`<div class=${containerClasses}>
      <sl-icon library="stencila" name="authors" class="text-xs"></sl-icon
      ><span class="font-sans text-2xs leading-none block"
        >Authors and Provenance</span
      >
      ${this.renderCollapse()}
    </div>`
  }

  /**
   * Render the collapse card under the header.
   *
   * Note: this code is inspired by @linkcode `stencila-ui-base-card` and could
   * be abstracted further.
   */
  private renderCollapse() {
    const classes = apply(['flex items-center', `brightness-75`])

    return html`<div class=${classes}>
      <stencila-chevron-button
        default-pos=${this.collapsed ? 'up' : 'down'}
        .disableEvents=${true}
        class="inline-flex text-xs"
      ></stencila-chevron-button>
    </div>`
  }
}
