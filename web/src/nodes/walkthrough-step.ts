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
    if (!previous) {
      this.isNext = true
    }
  }

  override updated(changedProperties: PropertyValues): void {
    super.updated(changedProperties)

    // Update `isNext` on the next step
    if (!this.isCollapsed) {
      const next = this.nextElementSibling as WalkthroughStep
      if (next) {
        next.isNext = true
      }
    }
  }

  override render() {
    const actionsStyle = apply(
      'flex gap-4 my-4 font-sans text-sm',
      'transition-all duration-500 ease-in-out',
      this.isNext ? 'max-h-screen opacity-100' : 'max-h-0 opacity-0'
    )

    const contentStyle = apply(
      'transition-all duration-1000 ease-in-out',
      this.isCollapsed ? 'max-h-0 opacity-0' : 'max-h-[5000px] opacity-100'
    )

    return html`<div class=${actionsStyle}>
        <sl-tooltip content="Expand next step">
          <span
            class="flex gap-2 items-center opacity-50"
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
            class="flex gap-2 items-center opacity-50"
            @click=${(e: Event) => this.expandAll(e)}
          >
            <stencila-ui-icon-button
              name="chevronDown"
            ></stencila-ui-icon-button>
            <span>Expand all</span>
          </span>
        </sl-tooltip>
      </div>
      <div class=${contentStyle}>
        <slot name="content"></slot>
      </div>`
  }
}
