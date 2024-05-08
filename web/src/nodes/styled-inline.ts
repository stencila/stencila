import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

import { Styled } from './styled'

/**
 * Web component representing a Stencila Schema `StyledInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled-inline.md
 */
@customElement('stencila-styled-inline')
export class StyledInline extends Styled {
  /**
   * In static view just render the `content` with styles applied
   */
  override renderStaticView() {
    this.adoptCss()

    return html`<span class="styled">
      <span class="${this.classes}">
        <slot name="content"></slot>
      </span>
    </span>`
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with authors and code read-only.
   */
  override renderDynamicView() {
    this.adoptCss()

    return html` <stencila-ui-inline-on-demand
      type="StyledInline"
      view="dynamic"
    >
      <div slot="body">
        <stencila-ui-node-authors type="StyledInline">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
        <stencila-ui-node-provenance type="StyledInline">
          <slot name="provenance"></slot>
        </stencila-ui-node-provenance>

        <stencila-ui-node-code
          type="StyledInline"
          code=${this.code}
          language=${this.styleLanguage}
          read-only
        >
        </stencila-ui-node-code>
      </div>

      <span slot="content" class="styled">
        <span class="${this.classes}">
          <slot name="content"></slot>
        </span>
      </span>
    </stencila-ui-inline-on-demand>`
  }

  /**
   * In source view just render authors
   *
   * TODO: Also render compiled CSS and styled content to help with debugging?
   */
  override renderSourceView() {
    return html` <stencila-ui-block-in-flow type="StyledBlock" view="source">
      <div slot="body">
        <stencila-ui-node-authors type="StyledBlock">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
