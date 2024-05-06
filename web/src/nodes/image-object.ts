import { css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/card'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `ImageObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md
 */
@customElement('stencila-image-object')
@withTwind()
export class ImageObject extends Entity {
  /**
   * In static view just render the image
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  private imgStyles = css`
    & img {
      width: 100%;
    }
  `

  /**
   * In dynamic view, in addition to the image, render a node card.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="ImageObject" view="dynamic">
        <div slot="body"></div>
        <div slot="content" class=${this.imgStyles}>
          <slot></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render the same as for dynamic view, including the
   * image itself, which won't be displayed in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-node-card
        type="ImageObject"
        view="source"
        collapsible=${true}
      >
        <div slot="body">
          <slot></slot>
        </div>
      </stencila-ui-node-card>
    `
  }
}
