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
  override renderContent() {
    const styles = css`
      & video {
        width: 100%;
        max-width: 100%;
        height: auto;
      }
    `
    
    return html`
      <div slot="content" class=${styles}>
        ${this.mediaSrc 
          ? html`<video src=${this.mediaSrc} controls></video>` 
          : html`<slot></slot>`}
      </div>
    `
  }
}