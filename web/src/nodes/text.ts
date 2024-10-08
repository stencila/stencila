import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

// import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Text` node
 *
 * Similar to an HTML <span>. This component currently only exists to allow for
 * editing of a `Text` node when it one of the `parts` of an `InstructionMessage`.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md
 */
@customElement('stencila-text')
@withTwind()
export class Text extends LitElement {
  override render() {
    return html`<slot></slot>`
  }
}
