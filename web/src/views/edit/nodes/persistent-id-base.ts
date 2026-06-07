/**
 * Base class for node property popovers that expose a persistent id.
 *
 * Most editable nodes can carry a persistent id, so the draft state, validation,
 * and the "Remove id" action are shared here; concrete popovers add any further
 * node-specific fields on top.
 */
import type { PropertyValues } from 'lit'
import { html } from 'lit'
import { state } from 'lit/decorators'

import type { EditNodePropertyPatch } from './node-properties'
import {
  normalizePersistentIdInput,
  validatePersistentIdInput,
} from './node-properties'
import '../properties/persistent-id'
import { EditNodePropertiesBase } from './base'

export abstract class EditPersistentIdNodePropertiesBase extends EditNodePropertiesBase {
  @state()
  protected draftPersistentId = ''

  @state()
  protected persistentIdError?: string

  protected override willUpdate(changedProperties: PropertyValues<this>) {
    if (changedProperties.has('target') && this.targetChanged()) {
      this.resetDrafts()
    }
  }

  /**
   * Seed draft fields from the current target.
   *
   * Called when the popover binds to a new node so the form reflects the node's
   * stored values rather than stale input. Subclasses extend this to reset their
   * own fields.
   */
  protected resetDrafts() {
    this.draftPersistentId = this.target?.persistentId ?? ''
    this.persistentIdError = undefined
  }

  protected updatePersistentId(event: CustomEvent<{ value: string }>) {
    this.draftPersistentId = event.detail.value
    this.persistentIdError = undefined
  }

  /**
   * Build the persistent-id portion of a save patch, or `undefined` if the input
   * is invalid.
   *
   * Empty input clears the id; otherwise the value is validated and, on failure,
   * surfaced as `persistentIdError` so the form can stay open with feedback.
   */
  protected persistentIdPatch(): EditNodePropertyPatch | undefined {
    const editor = this.editor
    const target = this.target
    if (!editor || !target) {
      return undefined
    }

    const normalizedId = normalizePersistentIdInput(this.draftPersistentId)
    if (!normalizedId) {
      return { persistentId: null }
    }

    const validation = validatePersistentIdInput(
      this.draftPersistentId,
      editor.state,
      target.pos
    )
    if (validation.ok === false) {
      this.persistentIdError = validation.message
      return undefined
    }

    return { persistentId: validation.value }
  }

  protected removePersistentId = () => {
    this.dispatchPropertyPatch({ persistentId: null })
  }

  protected renderPersistentIdProperty() {
    return html`
      <stencila-edit-persistent-id-property
        autofocus
        .value=${this.draftPersistentId}
        .error=${this.persistentIdError}
        @edit-property-value-change=${this.updatePersistentId}
      ></stencila-edit-persistent-id-property>
    `
  }

  /**
   * Render the "Remove id" action, disabled when the node has no id to remove.
   *
   * Provided as a leading action to the base `renderActions` so the
   * persistent-id concern stays out of the generic base.
   */
  protected renderRemovePersistentIdAction() {
    return html`
      <button
        type="button"
        class="stencila-edit-node-properties-action"
        ?disabled=${!this.target?.persistentId}
        @click=${this.removePersistentId}
      >
        <span class="i-lucide:trash-2" aria-hidden="true"></span>
        <span>Remove id</span>
      </button>
    `
  }
}
