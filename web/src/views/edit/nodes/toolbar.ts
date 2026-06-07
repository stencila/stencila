/**
 * Floating inspector for properties of the selected or hovered Tiptap node.
 *
 * Specific to the document edit view: selecting or hovering an editable node
 * shows a compact read-only property inspector. The cog action in that inspector
 * drills into the existing editable property form for the node.
 */
import { autoUpdate, computePosition, flip, offset, shift } from '@floating-ui/dom'
import type { Editor } from '@tiptap/core'
import {
  LitElement,
  type PropertyValues,
  type TemplateResult,
  html,
  nothing,
} from 'lit'
import { customElement, property, state } from 'lit/decorators'

import './code-block'
import type {
  EditNodePropertyNodeType,
  EditNodePropertyTarget,
} from './node-properties'
import {
  editNodePropertyTargetKey,
  findEditNodePropertyTarget,
  findEditNodePropertyTargetAtPosition,
} from './node-properties'
import './stencila-block'
import './table'

type PopoverMode = 'summary' | 'edit'

/**
 * Delay before a hovered node's inspector appears.
 *
 * Sweeping the pointer across nodes reschedules this timer, so the inspector
 * only shows once the pointer settles, avoiding flicker. Selection shows the
 * inspector immediately (no delay).
 */
const HOVER_SHOW_DELAY_MS = 200

/**
 * An edit target together with the node's DOM element, used as the floating-ui
 * reference for positioning the inspector over its node.
 */
interface PositionedEditNodePropertyTarget extends EditNodePropertyTarget {
  referenceElement: HTMLElement
}

@customElement('stencila-edit-node-toolbar')
export class EditNodeToolbar extends LitElement {
  @property({ attribute: false })
  editor?: Editor

  @state()
  private selectedTarget?: PositionedEditNodePropertyTarget

  @state()
  private hoverTarget?: PositionedEditNodePropertyTarget

  @state()
  private popoverMode: PopoverMode = 'summary'

  @state()
  private dismissed = false

  private attachedEditor?: Editor

  /**
   * Pending timer for the hover show delay, cleared when the pointer leaves or
   * moves to a different node before the delay elapses.
   */
  private hoverTimer?: ReturnType<typeof setTimeout>

  /**
   * Active floating-ui `autoUpdate` loop and the node DOM element it is bound
   * to, so the loop is only rebuilt when the targeted node actually changes.
   */
  private floatingBinding?: { cleanup: () => void; reference: HTMLElement }

  /**
   * Re-evaluate the inspector target.
   *
   * Bound to editor transactions (selection or content changes) and pointer
   * movement, which determine *which* node is targeted. Once a target exists,
   * floating-ui's `autoUpdate` keeps it aligned through scroll and resize.
   */
  private refreshTarget = () => {
    this.updateTargets()
  }

  protected override createRenderRoot() {
    return this
  }

  override disconnectedCallback() {
    this.cancelHoverTimer()
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

  /**
   * Subscribe to transactions and pointer movement of the current editor,
   * detaching from any previous one.
   */
  private attachEditor(editor: Editor | undefined) {
    if (this.attachedEditor && !this.attachedEditor.isDestroyed) {
      this.attachedEditor.off('transaction', this.refreshTarget)
      this.attachedEditor.view.dom.removeEventListener(
        'pointermove',
        this.handleEditorPointerMove
      )
      this.attachedEditor.view.dom.removeEventListener(
        'pointerleave',
        this.handleEditorPointerLeave
      )
    }

    this.attachedEditor = editor
    if (this.attachedEditor) {
      this.attachedEditor.on('transaction', this.refreshTarget)
      this.attachedEditor.view.dom.addEventListener(
        'pointermove',
        this.handleEditorPointerMove
      )
      this.attachedEditor.view.dom.addEventListener(
        'pointerleave',
        this.handleEditorPointerLeave
      )
    }
    this.updateTargets()
  }

  private activeTarget(): PositionedEditNodePropertyTarget | undefined {
    return this.selectedTarget ?? this.hoverTarget
  }

  private activeTargetKey(): string | undefined {
    return this.targetKeyOf(this.activeTarget())
  }

  /**
   * Recompute the selected node target, then the hover target.
   *
   * A selected node takes over the inspector entirely: any hover target (and its
   * pending show timer) is cleared so the inspector does not jump away while
   * editing a selected table cell or block. The hover target is only resolved
   * when nothing is selected.
   */
  private updateTargets() {
    const editor = this.attachedEditor
    const previousKey = this.activeTargetKey()

    if (!editor) {
      this.selectedTarget = undefined
      this.hoverTarget = undefined
      this.popoverMode = 'summary'
      this.dismissed = false
      return
    }

    const selectedTarget = findEditNodePropertyTarget(editor.state)
    this.selectedTarget = selectedTarget
      ? this.positionTarget(selectedTarget)
      : undefined
    if (this.selectedTarget) {
      this.cancelHoverTimer()
      this.hoverTarget = undefined
    } else {
      const hoverTarget = this.hoverTarget
        ? findEditNodePropertyTargetAtPosition(editor.state, this.hoverTarget.pos)
        : undefined
      this.hoverTarget = hoverTarget
        ? this.positionTarget(hoverTarget)
        : undefined
    }

    this.resetModeIfTargetChanged(previousKey)
  }

  /**
   * Resolve the node's DOM element to anchor the floating inspector against, or
   * `undefined` if the node is not currently rendered.
   */
  private positionTarget(
    target: EditNodePropertyTarget
  ): PositionedEditNodePropertyTarget | undefined {
    const node = this.attachedEditor?.view.nodeDOM(target.pos)
    const referenceElement =
      node instanceof HTMLElement
        ? node
        : node?.parentElement instanceof HTMLElement
          ? node.parentElement
          : undefined

    if (!referenceElement) {
      return undefined
    }

    return { ...target, referenceElement }
  }

  /**
   * (Re)bind floating-ui so the inspector stays positioned over its node. Runs
   * after each render (from `updated`) so the floating element exists in the
   * DOM.
   */
  private updateFloating() {
    const target = this.dismissed ? undefined : this.activeTarget()
    this.syncFloating(target?.referenceElement)
  }

  /**
   * Bind, rebind, or tear down the floating-ui loop.
   *
   * The loop is rebuilt only when the reference (node DOM) element changes;
   * otherwise the existing loop keeps tracking it through scroll, resize and
   * layout shifts (and the inspector resizing, e.g. summary -> edit form).
   * ProseMirror may recreate a node's DOM across transactions, so the reference
   * is compared by element identity.
   *
   * `flip()` moves the panel below the node when there is no room above (top
   * edge) and `shift()` keeps it within the viewport (right/left/bottom edges).
   */
  private syncFloating(reference: HTMLElement | undefined) {
    const floating = this.querySelector<HTMLElement>(
      '.stencila-edit-node-toolbar'
    )
    const existing = this.floatingBinding

    if (!reference || !floating) {
      existing?.cleanup()
      this.floatingBinding = undefined
      return
    }

    if (existing && existing.reference === reference) {
      return
    }

    existing?.cleanup()
    const cleanup = autoUpdate(reference, floating, () => {
      void computePosition(reference, floating, {
        strategy: 'fixed',
        placement: 'top-start',
        middleware: [offset(4), flip(), shift({ padding: 8 })],
      }).then(({ x, y }) => {
        Object.assign(floating.style, { left: `${x}px`, top: `${y}px` })
      })
    })
    this.floatingBinding = { cleanup, reference }
  }

  private stopFloating() {
    this.floatingBinding?.cleanup()
    this.floatingBinding = undefined
  }

  private targetKeyOf(
    target: PositionedEditNodePropertyTarget | undefined
  ): string | undefined {
    return target ? editNodePropertyTargetKey(target) : undefined
  }

  private resetModeIfTargetChanged(previousKey: string | undefined) {
    const nextKey = this.activeTargetKey()
    if (!nextKey || previousKey !== nextKey) {
      this.popoverMode = 'summary'
      this.dismissed = false
    }
  }

  private setHoverTarget(target: EditNodePropertyTarget | undefined) {
    const previousKey = this.activeTargetKey()
    this.hoverTarget = target ? this.positionTarget(target) : undefined
    this.resetModeIfTargetChanged(previousKey)
  }

  private cancelHoverTimer() {
    if (this.hoverTimer !== undefined) {
      clearTimeout(this.hoverTimer)
      this.hoverTimer = undefined
    }
  }

  private clearHoverTarget() {
    this.cancelHoverTimer()
    this.setHoverTarget(undefined)
  }

  /**
   * Commit a pending hover (after the show delay) by re-resolving the target
   * from the current editor state, so a node edited or moved during the delay is
   * positioned correctly.
   */
  private commitHover(pos: number) {
    this.hoverTimer = undefined
    const editor = this.attachedEditor
    if (!editor) {
      return
    }
    this.setHoverTarget(findEditNodePropertyTargetAtPosition(editor.state, pos))
  }

  private handleEditorPointerMove = (event: PointerEvent) => {
    const editor = this.attachedEditor
    if (!editor) {
      return
    }

    if (this.selectedTarget) {
      this.clearHoverTarget()
      return
    }

    const position = editor.view.posAtCoords({
      left: event.clientX,
      top: event.clientY,
    })
    if (!position) {
      this.clearHoverTarget()
      return
    }

    const target = findEditNodePropertyTargetAtPosition(
      editor.state,
      position.inside >= 0 ? position.inside : position.pos
    )
    if (!target) {
      this.clearHoverTarget()
      return
    }

    // Already showing this node: leave it be, and drop any pending switch to a
    // node the pointer has moved back from.
    const targetKey = editNodePropertyTargetKey(target)
    if (targetKey === this.targetKeyOf(this.hoverTarget)) {
      this.cancelHoverTimer()
      return
    }

    // A different node: (re)start the show delay so sweeping the pointer across
    // nodes does not flicker the inspector.
    this.cancelHoverTimer()
    const pos = target.pos
    this.hoverTimer = setTimeout(() => this.commitHover(pos), HOVER_SHOW_DELAY_MS)
  }

  private handleEditorPointerLeave = (event: PointerEvent) => {
    const relatedTarget = event.relatedTarget
    if (relatedTarget instanceof Node && this.contains(relatedTarget)) {
      return
    }

    this.clearHoverTarget()
  }

  private handleInspectorPointerLeave(event: PointerEvent) {
    const relatedTarget = event.relatedTarget
    const editorDom = this.attachedEditor?.view.dom
    if (
      relatedTarget instanceof Node &&
      editorDom instanceof HTMLElement &&
      editorDom.contains(relatedTarget)
    ) {
      return
    }

    this.clearHoverTarget()
  }

  /**
   * Keep the editor selection intact when an inspector button is pressed by
   * preventing the mousedown from stealing focus and collapsing the selection.
   */
  private keepEditorFocused(event: MouseEvent) {
    event.preventDefault()
  }

  private openEditMode() {
    if (this.activeTarget()) {
      this.popoverMode = 'edit'
    }
  }

  /**
   * Hide the inspector for the current node. It reappears when the active node
   * changes (see `resetModeIfTargetChanged`).
   */
  private dismiss() {
    this.dismissed = true
  }

  private closePopover() {
    this.popoverMode = 'summary'
    this.attachedEditor?.view.focus()
  }

  private handlePropertiesChanged() {
    this.popoverMode = 'summary'
    this.updateTargets()
  }

  private readonly editPopoverRenderers: Record<
    EditNodePropertyNodeType,
    (target: PositionedEditNodePropertyTarget) => TemplateResult
  > = {
    codeBlock: (target) => html`
      <stencila-edit-code-block-properties
        .editor=${this.attachedEditor}
        .target=${target}
        @edit-node-properties-close=${this.closePopover}
        @edit-node-properties-change=${this.handlePropertiesChanged}
      ></stencila-edit-code-block-properties>
    `,
    table: (target) => html`
      <stencila-edit-table-properties
        .editor=${this.attachedEditor}
        .target=${target}
        @edit-node-properties-close=${this.closePopover}
        @edit-node-properties-change=${this.handlePropertiesChanged}
      ></stencila-edit-table-properties>
    `,
    stencilaBlock: (target) => html`
      <stencila-edit-stencila-block-properties
        .editor=${this.attachedEditor}
        .target=${target}
        @edit-node-properties-close=${this.closePopover}
        @edit-node-properties-change=${this.handlePropertiesChanged}
      ></stencila-edit-stencila-block-properties>
    `,
  }

  private renderEditPopover(target: PositionedEditNodePropertyTarget) {
    return this.editPopoverRenderers[target.typeName](target)
  }

  private renderSummary(target: PositionedEditNodePropertyTarget) {
    return html`
      <div
        class="stencila-edit-node-inspector"
        role="group"
        aria-label=${`${target.displayName} properties`}
      >
        <span
          class=${`stencila-edit-node-inspector-type ${target.typeIcon}`}
          role="img"
          aria-label=${target.displayName}
          title=${target.displayName}
        ></span>
        <span class="stencila-edit-node-inspector-label"
          >${target.summaryLabel}</span
        >
        ${target.persistentId
          ? html`<span class="stencila-edit-node-inspector-id"
              >#${target.persistentId}</span
            >`
          : nothing}
        <button
          type="button"
          class="stencila-edit-node-inspector-edit"
          aria-label="Edit properties"
          title="Edit properties"
          @mousedown=${this.keepEditorFocused}
          @click=${this.openEditMode}
        >
          <span class="i-lucide:sliders-horizontal" aria-hidden="true"></span>
        </button>
        <button
          type="button"
          class="stencila-edit-node-inspector-dismiss"
          aria-label="Dismiss"
          title="Dismiss"
          @mousedown=${this.keepEditorFocused}
          @click=${this.dismiss}
        >
          <span class="i-lucide:x" aria-hidden="true"></span>
        </button>
      </div>
    `
  }

  /**
   * Render the single floating inspector. It starts as the read-only summary and
   * can drill into the editable property form for the active node.
   */
  private renderPanel(target: PositionedEditNodePropertyTarget) {
    const content =
      this.popoverMode === 'edit'
        ? this.renderEditPopover(target)
        : this.renderSummary(target)
    return html`
      <div
        class="stencila-edit-node-toolbar"
        @pointerleave=${this.handleInspectorPointerLeave}
      >
        ${content}
      </div>
    `
  }

  override render() {
    const target = this.dismissed ? undefined : this.activeTarget()
    if (!target) {
      return nothing
    }

    return this.renderPanel(target)
  }
}
