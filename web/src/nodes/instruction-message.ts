import { html } from 'lit'
import { customElement } from 'lit/decorators.js'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../twind'
import type { ImageDropContainer } from '../ui/inputs/imagedrop'
import '../ui/inputs/imagedrop'

import { Entity } from './entity'

import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `InstructionMessage` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-message.md
 */
@customElement('stencila-instruction-message')
@withTwind()
export class InstructionMessage extends Entity {
  /**
   * ref for the images-drop component,
   * use `this.imageDropRef.value.files` to get the `Files[]`
   */
  protected imageDropRef: Ref<ImageDropContainer> = createRef()

  override render() {
    return html`
      <div>
        <div class="flex justify-between px-3 py-2">
          <slot name="parts"></slot>
          <stencila-image-drop-container
            ${ref(this.imageDropRef)}
            class="w-1/4 text-xs"
          ></stencila-image-drop-container>
        </div>

        <stencila-ui-node-authors type="InstructionMessage">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </div>
    `
  }
}
