/**
 * Base class for node property popovers in the edit view.
 *
 * Concrete popovers (code block, table, opaque Stencila block) differ only in
 * which fields they show, so the shared chrome — the dispatch of property
 * patches back to the editor, the close/keydown handling, the header and the
 * action buttons — lives here to keep each popover small and consistent.
 */
import type { Editor } from '@tiptap/core'
import { LitElement, type TemplateResult, html, nothing } from 'lit'
import { property } from 'lit/decorators'

import type { EditNodePropertyPatch, EditNodePropertyTarget } from './node-properties'
import {
  editNodePropertyTargetKey,
  setEditNodePropertiesTransaction,
} from './node-properties'

export const EDIT_NODE_PROPERTIES_CLOSE_EVENT = 'edit-node-properties-close'
export const EDIT_NODE_PROPERTIES_CHANGE_EVENT = 'edit-node-properties-change'

export abstract class EditNodePropertiesBase extends LitElement {
  @property({ attribute: false })
  editor?: Editor

  @property({ attribute: false })
  target?: EditNodePropertyTarget

  private activeTargetKey?: string

  protected override createRenderRoot() {
    return this
  }

  /**
   * Report whether `target` now refers to a different node than last seen.
   *
   * Subclasses use this to reset their draft state only when the popover moves
   * to a new node, not on every unrelated re-render of the same node.
   */
  protected targetChanged(): boolean {
    const nextTargetKey = this.target
      ? editNodePropertyTargetKey(this.target)
      : undefined
    if (nextTargetKey === this.activeTargetKey) {
      return false
    }

    this.activeTargetKey = nextTargetKey
    return true
  }

  protected closeProperties() {
    this.dispatchEvent(
      new CustomEvent(EDIT_NODE_PROPERTIES_CLOSE_EVENT, {
        bubbles: true,
        composed: true,
      })
    )
  }

  /**
   * Close the popover on Escape, stopping propagation so the key does not also
   * reach the editor (which would, for example, clear the selection).
   */
  protected handlePropertiesKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation()
      this.closeProperties()
    }
  }

  /**
   * Apply a property patch to the target node and notify listeners.
   *
   * The transaction is scrolled into view and focus returned to the editor so
   * editing continues naturally after a save.
   */
  protected dispatchPropertyPatch(patch: EditNodePropertyPatch) {
    const editor = this.editor
    const target = this.target
    if (!editor || !target) {
      return
    }

    const transaction = setEditNodePropertiesTransaction(editor.state, target, patch)
    if (!transaction) {
      return
    }

    editor.view.dispatch(transaction.scrollIntoView())
    editor.view.focus()
    this.dispatchEvent(
      new CustomEvent(EDIT_NODE_PROPERTIES_CHANGE_EVENT, {
        bubbles: true,
        composed: true,
        detail: { patch },
      })
    )
  }

  protected renderHeader() {
    return html`
      <div class="stencila-edit-node-properties-popover-header">
        <span>${this.target?.displayName ?? 'Node'}</span>
      </div>
    `
  }

  /**
   * Render the popover's action row: the shared Cancel/Save buttons plus any
   * node-specific `leadingActions` (e.g. a "Remove id" button) supplied by a
   * subclass.
   */
  protected renderActions(leadingActions: TemplateResult | typeof nothing = nothing) {
    return html`
      <div class="stencila-edit-node-properties-actions">
        ${leadingActions}
        <button
          type="button"
          class="stencila-edit-node-properties-action"
          @click=${this.closeProperties}
        >
          <span class="i-lucide:x" aria-hidden="true"></span>
          <span>Cancel</span>
        </button>
        <button type="submit" class="stencila-edit-node-properties-action primary">
          <span class="i-lucide:check" aria-hidden="true"></span>
          <span>Save</span>
        </button>
      </div>
    `
  }
}
