import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

/**
 * Web component representing a Stencila Schema `Text` node
 *
 * Similar to an HTML <span>. This component currently only exists to allow for
 * editing of a `Text` node when it one of the `parts` of an `InstructionMessage`.
 *
 * Important: This element serves as parts of more contextual Stencila node entities
 * (e.g. `Paragraph`, `Heading`, `List`). It should NEVER subclass the abstract `Entity` class,
 * as it will affect some of the context related functionality between its parent node
 * and the text content.
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
