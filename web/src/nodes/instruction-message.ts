import { html } from 'lit'
import { customElement } from 'lit/decorators.js'
// import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../twind'
// import type { UIImageUpload } from '../ui/inputs/image-upload'
import '../ui/inputs/image-upload'

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
   * Ref for the images-drop component.
   *
   * Use `this.imageDropRef.value.files` to get the `Files[]`
   */
  //protected imageDropRef: Ref<UIImageUpload> = createRef()

  override render() {
    return html`
      <slot name="parts"></slot>

      <stencila-ui-node-authors type="InstructionMessage">
        <stencila-ui-node-provenance slot="provenance">
          <slot name="provenance"></slot>
        </stencila-ui-node-provenance>
        <slot name="authors"></slot>
      </stencila-ui-node-authors>
    `
  }

  /*
  This is currently not being used because the upload
  functionality is not fully implemented.

  private renderImageUpload() {
    return html`<stencila-ui-image-upload
      ${ref(this.imageDropRef)}
      class="text-xs"
    ></stencila-ui-image-upload>`
  }
  */
}
