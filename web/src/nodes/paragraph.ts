import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { Entity } from './entity'
import './helpers/block-infobox'

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
export abstract class Paragraph extends Entity {
  render() {
    return html`
      <slot name="content"></slot>

      <!-- TODO: For demoing only, remove -->
      View: ${this.documentView()}
      
      <stencila-block-infobox icon="paragraph" title="Paragraph">
        <slot name="authors" slot="authors"></slot>
      </stencila-block-infobox>
    `
  }
}
