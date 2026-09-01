/** Property inspector for native ResearchObject wrapper nodes. */
import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import {
  type ResearchObjectNodeName,
  isResearchObjectNodeName,
} from '../../../tiptap/research-objects'
import '../properties/relations'
import type {
  EditNodeRelationDraft,
  ResearchObjectTargetOption,
} from './node-properties'
import {
  listResearchObjectTargets,
  validateRelationDrafts,
} from './node-properties'
import { EditPersistentIdNodePropertiesBase } from './persistent-id-base'

@customElement('stencila-edit-research-object-properties')
export class EditResearchObjectProperties extends EditPersistentIdNodePropertiesBase {
  @state()
  private draftRelations: EditNodeRelationDraft[] = []

  @state()
  private relationsError?: string

  protected override resetDrafts() {
    super.resetDrafts()
    this.draftRelations = (this.target?.relations ?? []).map((relation) => ({
      ...relation,
    }))
    this.relationsError = undefined
  }

  private source(): ResearchObjectNodeName {
    const typeName = this.target?.typeName ?? ''
    return isResearchObjectNodeName(typeName) ? typeName : 'claim'
  }

  private relationTargets(): ResearchObjectTargetOption[] {
    const editor = this.editor
    const target = this.target
    return editor && target
      ? listResearchObjectTargets(editor.state, target.pos)
      : []
  }

  private updateRelations(
    event: CustomEvent<{ value: EditNodeRelationDraft[] }>
  ) {
    this.draftRelations = event.detail.value
    this.relationsError = undefined
  }

  private saveProperties(event: SubmitEvent) {
    event.preventDefault()

    const patch = this.persistentIdPatch()
    if (!patch) {
      return
    }

    const validation = validateRelationDrafts(this.draftRelations)
    if (validation.ok === false) {
      this.relationsError = validation.message
      return
    }

    this.dispatchPropertyPatch({
      ...patch,
      relations: this.draftRelations.length ? this.draftRelations : null,
    })
  }

  override render() {
    return html`
      <form
        class="stencila-edit-node-properties-popover stencila-edit-node-properties-popover-wide"
        @submit=${this.saveProperties}
        @keydown=${this.handlePropertiesKeydown}
      >
        ${this.renderHeader()} ${this.renderPersistentIdProperty()}
        <stencila-edit-relations-property
          .rows=${this.draftRelations}
          .source=${this.source()}
          .targets=${this.relationTargets()}
          .error=${this.relationsError}
          @edit-property-value-change=${this.updateRelations}
        ></stencila-edit-relations-property>
        ${this.renderActions(this.renderRemovePersistentIdAction())}
      </form>
    `
  }
}
