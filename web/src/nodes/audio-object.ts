import { css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { MediaObject } from './media-object'

/**
 * Web component representing a Stencila Schema `AudioObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/audio-object.md
 */
@customElement('stencila-audio-object')
@withTwind()
export class AudioObject extends MediaObject {
  override renderContent() {
    const styles = css`
      & audio {
        width: 100%;
        max-width: 100%;
      }
    `
    return html`
      <div slot="content" class=${styles}>
        ${this.mediaSrc 
          ? html`<audio src=${this.mediaSrc} controls></audio>` 
          : html`<slot></slot>`}
      </div>
    `
  }
}