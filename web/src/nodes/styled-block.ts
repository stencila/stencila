import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/card'
import '../ui/nodes/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code'

import { Styled } from './styled'

/**
 * Web component representing a Stencila Schema `StyledBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled-block.md
 */
@customElement('stencila-styled-block')
export class StyledBlock extends Styled {
  private contentCSS = apply(['w-full', '-mx-4'])
  /**
   * In static view just render the `content` with styles applied
   */
  override renderStaticView() {
    this.adoptCss()

    return html`<div class="styled">
      <div class="${this.classes}">
        <slot name="content"></slot>
      </div>
    </div>`
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with authors and code read-only.
   */
  override renderDynamicView() {
    this.adoptCss()

    return html` <stencila-ui-block-in-flow type="StyledBlock" view="dynamic">
      <div slot="body">
        <stencila-ui-node-authors type="StyledBlock">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="StyledBlock"
          code=${this.code}
          language=${this.styleLanguage}
          read-only
          collapsed
        >
        </stencila-ui-node-code>
      </div>

      <div slot="content" class=${`styled`}>
        <div class="${this.classes}">
          <slot name="content"></slot>
        </div>
      </div>
    </stencila-ui-block-in-flow>`
  }

  /**
   * In source view just render authors
   *
   * TODO: Also render compiled CSS and styled content to help with debugging?
   */
  override renderSourceView() {
    this.adoptCss()

    return html` <stencila-ui-block-in-flow type="StyledBlock" view="source">
      <div slot="body">
        <stencila-ui-node-authors type="StyledBlock">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="StyledBlock"
          code=${this.code}
          language=${this.styleLanguage}
          read-only
          collapsed
        >
        </stencila-ui-node-code>
      </div>
      <div slot="content" class=${`styled ${this.contentCSS}`}>
        <div class="${this.classes}">
          <slot name="content"></slot>
        </div>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
