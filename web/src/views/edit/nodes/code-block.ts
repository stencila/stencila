/**
 * Property popover for `codeBlock` nodes: persistent id, programming language
 * and a "render as demo" flag.
 */
import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import '../properties/boolean'
import '../properties/programming-language'
import type { EditNodePropertyPatch } from './node-properties'
import { EditPersistentIdNodePropertiesBase } from './persistent-id-base'

@customElement('stencila-edit-code-block-properties')
export class EditCodeBlockProperties extends EditPersistentIdNodePropertiesBase {
  @state()
  private draftProgrammingLanguage = ''

  @state()
  private draftIsDemo = false

  protected override resetDrafts() {
    super.resetDrafts()
    this.draftProgrammingLanguage = this.target?.programmingLanguage ?? ''
    this.draftIsDemo = this.target?.isDemo ?? false
  }

  private updateProgrammingLanguage(event: CustomEvent<{ value: string }>) {
    this.draftProgrammingLanguage = event.detail.value
  }

  private updateIsDemo(event: CustomEvent<{ value: boolean }>) {
    this.draftIsDemo = event.detail.value
  }

  private saveProperties(event: SubmitEvent) {
    event.preventDefault()

    const patch = this.persistentIdPatch()
    if (!patch) {
      return
    }

    // Empty language clears it; demo defaults to off, so store `null` rather
    // than `false` to keep the attribute absent unless explicitly enabled.
    const codeBlockPatch: EditNodePropertyPatch = {
      ...patch,
      programmingLanguage: this.draftProgrammingLanguage.trim() || null,
      isDemo: this.draftIsDemo ? true : null,
    }
    this.dispatchPropertyPatch(codeBlockPatch)
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
          .value=${this.draftProgrammingLanguage}
          @edit-property-value-change=${this.updateProgrammingLanguage}
        ></stencila-edit-programming-language-property>
        <stencila-edit-boolean-property
          label="Render as demo"
          .checked=${this.draftIsDemo}
          @edit-property-value-change=${this.updateIsDemo}
        ></stencila-edit-boolean-property>
        ${this.renderActions(this.renderRemovePersistentIdAction())}
      </form>
    `
  }
}
