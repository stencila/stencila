/**
 * Property popover for opaque `stencilaBlock` placeholders, exposing only the
 * persistent id stored inside the serialized node payload.
 */
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { EditPersistentIdNodePropertiesBase } from './persistent-id-base'

@customElement('stencila-edit-stencila-block-properties')
export class EditStencilaBlockProperties extends EditPersistentIdNodePropertiesBase {
  private saveProperties(event: SubmitEvent) {
    event.preventDefault()

    const patch = this.persistentIdPatch()
    if (patch) {
      this.dispatchPropertyPatch(patch)
    }
  }

  override render() {
    return html`
      <form
        class="stencila-edit-node-properties-popover"
        @submit=${this.saveProperties}
        @keydown=${this.handlePropertiesKeydown}
      >
        ${this.renderHeader()} ${this.renderPersistentIdProperty()}
        ${this.renderActions(this.renderRemovePersistentIdAction())}
      </form>
    `
  }
}
