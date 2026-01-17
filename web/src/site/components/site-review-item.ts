import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { ref, createRef, Ref } from 'lit/directives/ref.js'

import type { ReviewItem, ReviewItemAnchor } from './site-review-types'

/**
 * Selection info passed from parent for input mode
 */
export interface SelectionInfo {
  start: ReviewItemAnchor
  end: ReviewItemAnchor
  selectedText: string
}

/**
 * Event detail for item-add event
 */
export interface ItemAddDetail {
  type: 'comment' | 'suggestion'
  content: string
  selection: SelectionInfo | null
}

/**
 * Event detail for item-edit event
 */
export interface ItemEditDetail {
  index: number
  content: string
}

/**
 * Event detail for item-delete event
 */
export interface ItemDeleteDetail {
  index: number
}

/**
 * Event detail for item-click event
 */
export interface ItemClickDetail {
  index: number
  item: ReviewItem
}

/**
 * Site review item component
 *
 * Operates in two modes:
 * - 'display': Shows an existing review item with edit/delete capabilities
 * - 'input': Modal for creating a new comment or suggestion
 *
 * Encapsulates editing state, menu state, and input state internally.
 */
@customElement('stencila-site-review-item')
export class StencilaSiteReviewItem extends LitElement {
  // =========================================================================
  // Display Mode Properties
  // =========================================================================

  /**
   * The review item to display (display mode only)
   */
  @property({ type: Object })
  item?: ReviewItem

  /**
   * Index of this item in the parent's pendingItems array
   */
  @property({ type: Number })
  index?: number

  /**
   * Whether this item is currently active/selected
   */
  @property({ type: Boolean })
  active = false

  // =========================================================================
  // Input Mode Properties
  // =========================================================================

  /**
   * Component mode: 'display' for existing items, 'input' for new item modal
   */
  @property({ type: String })
  mode: 'display' | 'input' = 'display'

  /**
   * Type of input: comment or suggestion (input mode)
   */
  @property({ type: String })
  type: 'comment' | 'suggestion' = 'comment'

  /**
   * Selection info from parent (input mode, optional for page-level comments)
   */
  @property({ type: Object })
  selection?: SelectionInfo | null

  /**
   * Position for popover mode (input mode only)
   * When provided, renders as a positioned popover instead of centered modal
   */
  @property({ type: Object })
  popoverPosition?: { top: number; left: number; maxWidth: number } | null

  /**
   * Page title to show in input header
   */
  @property({ type: String, attribute: 'page-title' })
  pageTitle = ''

  // =========================================================================
  // Internal State
  // =========================================================================

  /**
   * Whether the item is in edit mode (display mode)
   */
  @state()
  private isEditing = false

  /**
   * Content being edited
   */
  @state()
  private editContent = ''

  /**
   * Original content when editing started (for change detection)
   */
  private editOriginalContent = ''

  /**
   * Whether the dropdown menu is open
   */
  @state()
  private menuOpen = false

  /**
   * Position of the open menu (for fixed positioning)
   */
  @state()
  private menuPosition: { top: number; right: number } | null = null

  /**
   * Content for new item input (input mode)
   */
  @state()
  private inputContent = ''

  /**
   * Whether the input modal is animating (flying to FAB)
   */
  @state()
  private isFlying = false

  /**
   * Ref for the input/edit textarea (for auto-focus)
   */
  private textareaRef: Ref<HTMLTextAreaElement> = createRef()

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // In input mode, initialize content for suggestions
    if (this.mode === 'input' && this.type === 'suggestion' && this.selection) {
      this.inputContent = this.selection.selectedText
    }

    // Add click-outside listener for menu
    document.addEventListener('click', this.handleDocumentClick)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('click', this.handleDocumentClick)
  }

  /**
   * Close menu when clicking outside
   */
  private handleDocumentClick = (e: MouseEvent) => {
    if (this.menuOpen) {
      const path = e.composedPath()
      if (!path.includes(this)) {
        this.menuOpen = false
        this.menuPosition = null
      }
    }
  }

  /**
   * Whether the edited content has changed from the original
   */
  private get hasEditChanges(): boolean {
    return this.editContent !== this.editOriginalContent
  }

  /**
   * Focus the textarea when entering edit or input mode
   */
  override updated(changedProperties: Map<string, unknown>) {
    super.updated(changedProperties)

    // Auto-focus textarea when entering edit mode
    if (changedProperties.has('isEditing') && this.isEditing) {
      requestAnimationFrame(() => {
        this.textareaRef.value?.focus()
      })
    }

    // Auto-focus textarea when input mode component is added
    if (this.mode === 'input' && changedProperties.has('mode')) {
      requestAnimationFrame(() => {
        this.textareaRef.value?.focus()
      })
    }
  }

  override firstUpdated() {
    // Auto-focus on first render for input mode
    if (this.mode === 'input') {
      requestAnimationFrame(() => {
        this.textareaRef.value?.focus()
      })
    }
  }

  // =========================================================================
  // Display Mode Methods
  // =========================================================================

  /**
   * Handle click on the item (not on menu)
   */
  private handleItemClick(e: Event) {
    const target = e.target as Element
    if (target.closest('.item-menu-container')) {
      return
    }

    e.stopPropagation()

    if (this.item && this.index !== undefined) {
      this.dispatchEvent(
        new CustomEvent<ItemClickDetail>('item-click', {
          detail: { index: this.index, item: this.item },
          bubbles: true,
          composed: true,
        })
      )
    }
  }

  /**
   * Toggle the dropdown menu
   */
  private toggleMenu(e: Event) {
    e.stopPropagation()

    if (this.menuOpen) {
      this.menuOpen = false
      this.menuPosition = null
    } else {
      const button = e.currentTarget as HTMLElement
      const rect = button.getBoundingClientRect()
      this.menuPosition = {
        top: rect.bottom + 4,
        right: window.innerWidth - rect.right,
      }
      this.menuOpen = true
    }
  }

  /**
   * Start editing the item
   */
  private startEditing(e: Event) {
    e.stopPropagation()
    if (this.item) {
      this.isEditing = true
      this.editContent = this.item.content
      this.editOriginalContent = this.item.content
      this.menuOpen = false
      this.menuPosition = null
    }
  }

  /**
   * Save the edit
   */
  private saveEdit(e: Event) {
    e.stopPropagation()

    if (this.index !== undefined && this.hasEditChanges) {
      this.dispatchEvent(
        new CustomEvent<ItemEditDetail>('item-edit', {
          detail: { index: this.index, content: this.editContent.trim() },
          bubbles: true,
          composed: true,
        })
      )
    }

    this.isEditing = false
    this.editContent = ''
    this.editOriginalContent = ''
  }

  /**
   * Cancel editing
   */
  private cancelEdit(e: Event) {
    e.stopPropagation()
    this.isEditing = false
    this.editContent = ''
    this.editOriginalContent = ''
  }

  /**
   * Delete the item
   */
  private deleteItem(e: Event) {
    e.stopPropagation()

    if (this.index !== undefined) {
      this.dispatchEvent(
        new CustomEvent<ItemDeleteDetail>('item-delete', {
          detail: { index: this.index },
          bubbles: true,
          composed: true,
        })
      )
    }

    this.menuOpen = false
    this.menuPosition = null
  }

  /**
   * Handle keyboard in edit textarea
   */
  private handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      this.cancelEdit(e)
    } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault()
      this.saveEdit(e)
    }
  }

  // =========================================================================
  // Input Mode Methods
  // =========================================================================

  /**
   * Handle adding the new item
   */
  private handleAdd(e: Event) {
    e.stopPropagation()

    if (!this.inputContent.trim()) {
      return
    }

    // Trigger fly animation
    this.isFlying = true

    setTimeout(() => {
      this.dispatchEvent(
        new CustomEvent<ItemAddDetail>('item-add', {
          detail: {
            type: this.type,
            content: this.inputContent.trim(),
            selection: this.selection ?? null,
          },
          bubbles: true,
          composed: true,
        })
      )

      // Reset state after dispatch
      this.isFlying = false
      this.inputContent = ''
    }, 300) // Duration of fly animation
  }

  /**
   * Handle canceling input
   */
  private handleCancel(e: Event) {
    e.stopPropagation()

    this.dispatchEvent(
      new CustomEvent('item-cancel', {
        bubbles: true,
        composed: true,
      })
    )

    this.inputContent = ''
  }

  /**
   * Handle keyboard in input textarea
   */
  private handleInputKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault()
      this.handleCancel(e)
    } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault()
      this.handleAdd(e)
    }
  }

  // =========================================================================
  // Render Methods
  // =========================================================================

  override render() {
    if (this.mode === 'input') {
      return this.renderInput()
    }
    return this.renderDisplay()
  }

  /**
   * Render the display mode (existing item)
   */
  private renderDisplay() {
    if (!this.item) {
      return null
    }

    if (this.isEditing) {
      return this.renderEditing()
    }

    return html`
      <div
        class="review-item"
        data-index=${this.index}
        data-active=${this.active}
        @click=${this.handleItemClick}
      >
        <div class="item-menu-container">
          <button
            class="item-menu-btn"
            @click=${this.toggleMenu}
            aria-label="Item options"
          >
            <span class="i-lucide:ellipsis-vertical"></span>
          </button>
          ${this.menuOpen && this.menuPosition
            ? html`
                <div
                  class="item-menu-dropdown"
                  style="position: fixed; top: ${this.menuPosition.top}px; right: ${this.menuPosition.right}px;"
                >
                  <button @click=${this.startEditing}>
                    <span class="i-lucide:pencil"></span>
                    Edit
                  </button>
                  <button class="danger" @click=${this.deleteItem}>
                    <span class="i-lucide:trash-2"></span>
                    Delete
                  </button>
                </div>
              `
            : null}
        </div>
        <div class="item-header">
          <span class="type-icon i-lucide:${this.item.type === 'comment' ? 'message-circle' : 'pencil'}"></span>
          <span class="item-type">${this.item.type === 'comment' ? 'Comment' : 'Suggestion'}</span>
        </div>
        ${this.item.type === 'comment'
          ? html`<div class="item-content">${this.item.content}</div>`
          : html`<div class="replacement-text">${this.item.content}</div>`}
      </div>
    `
  }

  /**
   * Render the editing state
   */
  private renderEditing() {
    if (!this.item) {
      return null
    }

    return html`
      <div class="review-item editing" data-index=${this.index}>
        <div class="item-header">
          <span class="type-icon i-lucide:${this.item.type === 'comment' ? 'message-circle' : 'pencil'}"></span>
          <span class="item-type">${this.item.type === 'comment' ? 'Comment' : 'Suggestion'}</span>
        </div>
        <textarea
          ${ref(this.textareaRef)}
          class="edit-textarea"
          .value=${this.editContent}
          @input=${(e: Event) => (this.editContent = (e.target as HTMLTextAreaElement).value)}
          @keydown=${this.handleEditKeydown}
        ></textarea>
        <div class="edit-actions">
          <button class="edit-btn cancel" @click=${this.cancelEdit}>Cancel</button>
          <button
            class="edit-btn save"
            @click=${this.saveEdit}
            ?disabled=${!this.hasEditChanges}
          >Save</button>
        </div>
      </div>
    `
  }

  /**
   * Render the input mode (new item modal or popover)
   */
  private renderInput() {
    const submitTip = `(${/Mac|iPhone|iPad|iPod/.test(navigator.userAgent) ? '⌘' : 'Ctrl'}+Enter)`

    // Use popover positioning if available (selection-based input)
    if (this.popoverPosition) {
      const { top, left, maxWidth } = this.popoverPosition
      const positionStyle = `position: fixed; top: ${top}px; left: ${left}px; max-width: ${maxWidth}px;`

      return html`
        <div
          class="review-input-popover ${this.isFlying ? 'flying' : ''}"
          style=${positionStyle}
        >
          <textarea
            ${ref(this.textareaRef)}
            .value=${this.inputContent}
            @input=${(e: Event) => (this.inputContent = (e.target as HTMLTextAreaElement).value)}
            @keydown=${this.handleInputKeydown}
            placeholder=${this.type === 'comment'
              ? `Add comment ${submitTip}`
              : `Suggest replacement ${submitTip}`}
            rows="3"
          ></textarea>
          <div class="popover-actions">
            <button class="btn secondary btn-sm" @click=${this.handleCancel}>
              Cancel
            </button>
            <button class="btn primary btn-sm" @click=${this.handleAdd}>
              ${this.type === 'comment' ? 'Comment' : 'Suggest'}
            </button>
          </div>
        </div>
      `
    }

    // Fallback to centered modal (page-level comments without selection)
    return html`
      <div class="modal input ${this.isFlying ? 'flying' : ''}">
        <div class="item-header">
          <span class="type-icon i-lucide:${this.type === 'comment' ? 'message-circle' : 'pencil'}"></span>
          <span class="item-path">${this.pageTitle || window.location.pathname}</span>
        </div>
        <textarea
          ${ref(this.textareaRef)}
          .value=${this.inputContent}
          @input=${(e: Event) => (this.inputContent = (e.target as HTMLTextAreaElement).value)}
          @keydown=${this.handleInputKeydown}
          placeholder=${this.type === 'comment'
            ? `Add your comment ${submitTip}`
            : `Suggest replacement text ${submitTip}`}
        ></textarea>
        <div class="buttons">
          <button class="btn secondary" @click=${this.handleCancel}>
            Cancel
          </button>
          <button class="btn primary" @click=${this.handleAdd}>
            ${this.type === 'comment' ? 'Comment' : 'Suggest'}
          </button>
        </div>
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-site-review-item': StencilaSiteReviewItem
  }
}
