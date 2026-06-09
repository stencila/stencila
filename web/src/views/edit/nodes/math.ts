/**
 * Property popover for native math nodes.
 */
import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import '../properties/programming-language'
import type { EditNodePropertyPatch } from './node-properties'
import { EditPersistentIdNodePropertiesBase } from './persistent-id-base'

@customElement('stencila-edit-math-properties')
export class EditMathProperties extends EditPersistentIdNodePropertiesBase {
  @state()
  private draftMathLanguage = ''

  protected override resetDrafts() {
    super.resetDrafts()
    this.draftMathLanguage = this.target?.mathLanguage ?? ''
  }

  private updateMathLanguage(event: CustomEvent<{ value: string }>) {
    this.draftMathLanguage = event.detail.value
  }

  private saveProperties(event: SubmitEvent) {
    event.preventDefault()

    const patch = this.persistentIdPatch()
    if (!patch) {
      return
    }

    const mathPatch: EditNodePropertyPatch = {
      ...patch,
      mathLanguage: this.draftMathLanguage.trim() || null,
    }
    this.dispatchPropertyPatch(mathPatch)
  }

  override render() {
    return html`
      <form
        class="stencila-edit-node-properties-popover"
        @submit=${this.saveProperties}
        @keydown=${this.handlePropertiesKeydown}
      >
        ${this.renderHeader()} ${this.renderPersistentIdProperty()}
        <stencila-edit-programming-language-property
          label="Math language"
          placeholder="tex"
          .value=${this.draftMathLanguage}
          @edit-property-value-change=${this.updateMathLanguage}
        ></stencila-edit-programming-language-property>
        ${this.renderActions(this.renderRemovePersistentIdAction())}
      </form>
    `
  }
}
