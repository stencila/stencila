import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

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
export class Text extends Entity {
  override render() {
    if (this.ancestors.endsWith('.InstructionMessage')) {
      const { borderColour } = nodeUi('InstructionBlock')
      const styles = apply([
        'h-12 w-full',
        'px-1',
        'text-black text-sm',
        `border border-[${borderColour}] rounded-sm`,
        'outline-black',
        'resize-none',
      ])
      return html`
        <div class="text-sm">
          <label class="mb-1">Instruction Text:</label>
          <textarea class=${styles}>${this.textContent}</textarea>
        </div>
      `
    }

    return html`<slot></slot>`
  }
}
