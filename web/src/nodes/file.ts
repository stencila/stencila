import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `File` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md
 */
@customElement('stencila-file')
@withTwind()
export class File extends Entity {
  @property()
  name: string

  @property()
  path: string

  @property({ attribute: 'media-type' })
  mediaType?: string

  @property({ attribute: 'transfer-encoding' })
  transferEncoding?: string

  @property({ type: Number })
  size?: number

  override render() {
    let tooltip = this.path || this.name

    if (this.mediaType) {
      tooltip += ' ('
      if (this.mediaType) {
        tooltip += this.mediaType
      }
      tooltip += ')'
    }

    return html`<sl-tooltip content=${tooltip}
      ><sl-tag class="font-sans m-1">${this.name}</sl-tag></sl-tooltip
    >`
  }
}
