import { css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { MediaObject } from './media-object'

/**
 * Web component representing a Stencila Schema `VideoObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/video-object.md
 */
@customElement('stencila-video-object')
@withTwind()
export class VideoObject extends MediaObject {
  override renderMediaElem() {
    return html`<video src=${this.mediaSrc} controls></video>` 
  }

  override renderCardContent() {
    const videoStyles = css`
      & {
        display: block;
        max-width: 100%;
        height: auto;
        margin: 1rem auto;
        border-radius: 3px;
      }
    `
    
    return html`
      <div slot="content">
        ${this.mediaSrc ?  html`<video class=${videoStyles} src=${this.mediaSrc} controls></video>` : html`<slot></slot>`}
        <div>
          <slot name="caption"></slot>
        </div>
      </div>
    `
  }
}