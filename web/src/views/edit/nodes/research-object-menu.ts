/** Contextual menu for wrapping selected blocks in a ResearchObject. */
import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
import type { Editor } from '@tiptap/core'
import { LitElement, type PropertyValues, html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import {
  RESEARCH_OBJECTS,
  type ResearchObjectNodeName,
  activeResearchObject,
  canWrapInResearchObject,
  unwrapResearchObject,
  wrapInResearchObject,
} from '../../../tiptap/research-objects'

@customElement('stencila-edit-research-object-menu')
export class EditResearchObjectMenu extends LitElement {
  @property({ attribute: false })
  editor?: Editor

  @state()
  private visible = false

  @state()
  private active: ResearchObjectNodeName | null = null

  private attachedEditor?: Editor
  private floatingCleanup?: () => void

  private refresh = () => this.updateVisibility()
  private hide = () => {
    this.visible = false
    this.active = null
  }

  protected override createRenderRoot() {
    return this
  }

  override disconnectedCallback() {
    this.stopFloating()
    this.attachEditor(undefined)
    super.disconnectedCallback()
  }

  protected override updated(changedProperties: PropertyValues<this>) {
    if (changedProperties.has('editor')) {
      this.attachEditor(this.editor)
    }
    this.updateFloating()
  }

  private attachEditor(editor: Editor | undefined) {
    if (this.attachedEditor && !this.attachedEditor.isDestroyed) {
      this.attachedEditor.off('transaction', this.refresh)
      this.attachedEditor.off('blur', this.hide)
      this.attachedEditor.off('focus', this.refresh)
    }

    this.attachedEditor = editor
    if (editor) {
      editor.on('transaction', this.refresh)
      editor.on('blur', this.hide)
      editor.on('focus', this.refresh)
    }
    this.updateVisibility()
  }

  private updateVisibility() {
    const editor = this.attachedEditor
    if (!editor || editor.isDestroyed || !editor.isEditable) {
      this.hide()
      return
    }

    this.active = activeResearchObject(editor)
    this.visible =
      !editor.state.selection.empty &&
      (canWrapInResearchObject(editor) || this.active !== null)
  }

  private selectionReference() {
    const editor = this.attachedEditor
    if (!editor || editor.isDestroyed) {
      return undefined
    }

    return {
      contextElement: editor.view.dom,
      getBoundingClientRect: (): DOMRect => {
        const { selection } = editor.state
        const start = editor.view.coordsAtPos(selection.from)
        const end = editor.view.coordsAtPos(selection.to, -1)
        const left = Math.min(start.left, end.left)
        const top = Math.min(start.top, end.top)
        const right = Math.max(start.right, end.right)
        const bottom = Math.max(start.bottom, end.bottom)
        return new DOMRect(left, top, Math.max(right - left, 1), bottom - top)
      },
    }
  }

  private updateFloating() {
    this.stopFloating()

    const floating = this.querySelector<HTMLElement>(
      '.stencila-edit-research-object-menu'
    )
    const reference = this.visible ? this.selectionReference() : undefined
    if (!reference || !floating) {
      return
    }

    this.floatingCleanup = autoUpdate(reference, floating, () => {
      void computePosition(reference, floating, {
        strategy: 'fixed',
        placement: 'top',
        middleware: [offset(8), flip(), shift({ padding: 8 })],
      }).then(({ x, y }) => {
        Object.assign(floating.style, { left: `${x}px`, top: `${y}px` })
      })
    })
  }

  private stopFloating() {
    this.floatingCleanup?.()
    this.floatingCleanup = undefined
  }

  private keepEditorFocused(event: MouseEvent) {
    event.preventDefault()
  }

  override render() {
    if (!this.visible) {
      return nothing
    }

    return html`
      <div
        class="stencila-edit-research-object-menu"
        role="toolbar"
        aria-label="Research object actions"
      >
        ${RESEARCH_OBJECTS.map(
          (item) => html`
            <button
              type="button"
              class="stencila-edit-research-object-menu-button"
              aria-label=${`Wrap in ${item.title}`}
              title=${`Wrap in ${item.title} (${item.shortcut})`}
              @mousedown=${this.keepEditorFocused}
              @click=${() =>
                this.attachedEditor &&
                wrapInResearchObject(this.attachedEditor, item.name)}
            >
              <span class=${item.icon} aria-hidden="true"></span>
              <span>${item.title}</span>
            </button>
          `
        )}
        ${this.active
          ? html`
              <button
                type="button"
                class="stencila-edit-research-object-menu-button"
                aria-label="Unwrap research object"
                title="Unwrap"
                @mousedown=${this.keepEditorFocused}
                @click=${() =>
                  this.attachedEditor &&
                  unwrapResearchObject(this.attachedEditor)}
              >
                <span class="i-lucide:ungroup" aria-hidden="true"></span>
                <span>Unwrap</span>
              </button>
            `
          : nothing}
      </div>
    `
  }
}
