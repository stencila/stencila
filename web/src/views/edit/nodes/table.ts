/**
 * Property popover for `table` nodes, exposing only the persistent id.
 */
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { EditPersistentIdNodePropertiesBase } from './persistent-id-base'

@customElement('stencila-edit-table-properties')
export class EditTableProperties extends EditPersistentIdNodePropertiesBase {
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
