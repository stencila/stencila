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
  override renderMediaElem() {
    // This inline style is necessary to override a constructed stylesheet
    // somewhere that is causing <img> to be displayed as block
    const audioStyles = css`
      & {
        display: inline;
        border-radius: 3px;
      }
    `
    return this.mediaSrc ?
      html`<audio class=${audioStyles} src=${this.mediaSrc} controls></audio>` :
      html`<slot></slot>`
  }

  override renderCardContent() {
    const audioStyles = css`
      & {
        display: block;
        width: 100%; // Without this audio controls are small and centered
        height: auto;
        margin: 1rem auto;
        border-radius: 3px;
      }
    `
    return html`
      <div slot="content" class="text-center">
        ${this.mediaSrc ? 
          html`<audio class=${audioStyles} src=${this.mediaSrc} controls></audio>` :
          html`<slot></slot>`}
        <div>
          <slot name="caption"></slot>
        </div>
      </div>
    `
  }
}