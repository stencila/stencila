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
  private imgStyles = css`
    & img {
      width: 100%;
    }
  `

  /**
   * In dynamic view, in addition to the image, render a node card.
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand type="ImageObject">
        <div slot="body"></div>
        <div slot="content" class=${this.imgStyles}>
          <slot></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
