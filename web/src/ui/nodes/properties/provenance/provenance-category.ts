import SlTooltip from '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { ProvenanceCategory, provenanceCategories } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../../../../twind'

/**
 * UI Provenance Category
 *
 * A token element that displays a ProvenanceCategory and an associated tooltip
 */
@customElement('stencila-ui-node-provenance-category')
@withTwind()
export class UIProvenanceCategory extends LitElement {
  @property()
  category: ProvenanceCategory

  @property({ type: Number })
  percentage: number | undefined

  /**
   * The refs used by this element.
   */
  private tooltipRef: Ref<SlTooltip> = createRef()
  private buttonRef: Ref<HTMLSpanElement> = createRef()

  protected override render(): unknown {
    const styles = apply([
      'inline-block',
      'cursor-default',
      'bg-white bg-blend-multiply',
      'text-black text-2xs leading-none',
      'px-2 py-1',
      'border border-white rounded-full',
      'transition-all duration-200 ease-in',
      'hover:bg-white/0 hover:border-black',
    ])
    return html`<sl-tooltip
      content=${this.tooltipText()}
      ${ref(this.tooltipRef)}
      ><strong class=${styles} ${ref(this.buttonRef)}
        >${this.category}
        ${this.percentage
          ? html`<span class="font-normal pointer-events-none"
              >${this.percentage}%</span
            >`
          : ''}</strong
      ></sl-tooltip
    >`
  }

  override firstUpdated() {
    const tooltip = this.tooltipText()

    this.buttonRef.value.addEventListener('mouseover', () => {
      this.tooltipRef.value.open = tooltip !== undefined
    })

    this.buttonRef.value.addEventListener('mouseout', () => {
      this.tooltipRef.value.open = false
    })
  }

  private tooltipText() {
    return provenanceCategories[this.category]
  }
}
