import { css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

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

  override render() {
    return this.ancestors.endsWith('.CodeChunk')
      ? html`
          <stencila-ui-block-on-demand type="ImageObject">
            <div slot="content" class=${this.imgStyles}>
              <slot></slot>
            </div>
          </stencila-ui-block-on-demand>
        `
      : html`
          <stencila-ui-inline-on-demand type="ImageObject">
            <div slot="body">
              <stencila-ui-node-authors type="ImageObject">
                <slot name="authors"></slot>
              </stencila-ui-node-authors>
            </div>
            <div slot="content" class=${this.imgStyles}>
              <slot></slot>
            </div>
          </stencila-ui-inline-on-demand>
        `
  }
}
