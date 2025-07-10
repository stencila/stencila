import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { patchValue } from '../clients/commands'
import { withTwind } from '../twind'
import { booleanConverter } from '../utilities/booleanConverter'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `WalkthroughStep` node
 *
 * This component provides the following functionality:
 *
 * - turns on/off visibility of the content of a walkthrough step (based on the `isCollapsed` property)
 * - add buttons to show the next step or expand all steps
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/walkthrough-step.md
 */
@customElement('stencila-walkthrough-step')
@withTwind()
export class WalkthroughStep extends Entity {
  @property({ attribute: 'is-collapsed', converter: booleanConverter })
  isCollapsed?: boolean

  @state()
  isNext: boolean = false

  /**
   * Emit an event to expand the walkthrough step
   */
  private expand(e: Event) {
    e.stopImmediatePropagation()

    this.dispatchEvent(
      patchValue('WalkthroughStep', this.id, 'isCollapsed', false)
    )
  }

  /**
   * Emit an event to expand all walkthrough steps
   */
  private expandAll(e: Event) {
    e.stopImmediatePropagation()

    this.dispatchEvent(
      patchValue(
        'Walkthrough',
        this.closestGlobally('stencila-walkthrough')?.id,
        'isCollapsed',
        false
      )
    )
  }

  override connectedCallback(): void {
    super.connectedCallback()

    // Set `isNext: true` on the first step
    const previous = this.previousElementSibling as WalkthroughStep
    if ((!previous || !previous.isCollapsed) && this.isCollapsed) {
      this.isNext = true
    } else {
      this.isNext = false
    }
  }

  override updated(changedProperties: PropertyValues): void {
    super.updated(changedProperties)

    // Update `isNext` on adjacent steps when collapsed state changes
    if (changedProperties.has('isCollapsed')) {
      if (!this.isCollapsed) {
        // When expanding, update the next collapsed step to be isNext
        const next = this.nextElementSibling as WalkthroughStep
        if (next && next.isCollapsed) {
          next.isNext = true
        }
        
        // Also check if any previous collapsed step should no longer be isNext
        let prev = this.previousElementSibling as WalkthroughStep
        while (prev) {
          if (prev.isNext) {
            prev.isNext = false
          }
          prev = prev.previousElementSibling as WalkthroughStep
        }
      } else {
        // When collapsing, check if this should become isNext
        const previous = this.previousElementSibling as WalkthroughStep
        if (!previous || !previous.isCollapsed) {
          this.isNext = true
        }
      }
    }
  }

  override render() {
    const contentStyle = apply(
      'transition-all duration-1000 ease-in-out',
      this.isCollapsed ? 'max-h-0 opacity-0' : 'max-h-[5000px] opacity-100'
    )

    return html`
      ${this.hasRoot() ? this.renderStepActions() : ''}
      <div class=${contentStyle}>
        <slot name="content"></slot>
      </div>
    `
  }

  protected renderStepActions() {
    const actionsStyle = apply(
      'flex gap-4 my-4 font-sans text-xs',
      'transition-all duration-500 ease-in-out',
      this.isCollapsed && this.isNext ? 'max-h-20em opacity-100 cursor-pointer' : 'max-h-0 opacity-0'
    )

    return html`
      <div class=${actionsStyle}>
        <sl-tooltip content="Expand next step">
          <span
            class="flex gap-1 items-center opacity-50"
            @click=${(e: Event) => this.expand(e)}
          >
            <stencila-ui-icon-button
              name="arrowRight"
            ></stencila-ui-icon-button>
            <span>Next</span>
          </span>
        </sl-tooltip>
        <sl-tooltip content="Expand all steps">
          <span
            class="flex gap-1 items-center opacity-50"
            @click=${(e: Event) => this.expandAll(e)}
          >
            <stencila-ui-icon-button
              name="chevronDown"
            ></stencila-ui-icon-button>
            <span>Expand all</span>
          </span>
        </sl-tooltip>
      </div>
    `
  }
}
