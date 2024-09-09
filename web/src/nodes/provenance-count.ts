import SlTooltip from '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import {
  NodeType,
  ProvenanceCategory,
  provenanceCategories,
} from '@stencila/types'
import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property } from 'lit/decorators'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import { renderProvenanceCategory } from '../ui/nodes/properties/provenance-category'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `ProvenanceCount` node
 */
@customElement('stencila-provenance-count')
@withTwind()
export class ProvenanceCount extends Entity {
  @property({ attribute: 'provenance-category' })
  provenanceCategory: ProvenanceCategory

  @property({ type: Number, attribute: 'character-count' })
  characterCount: number

  @property({ type: Number, attribute: 'character-percent' })
  characterPercent: number

  /**
   * The refs used by this element.
   */
  private tooltipRef: Ref<SlTooltip> = createRef()
  private buttonRef: Ref<HTMLElement> = createRef()

  override render() {
    const nodeType = this.ancestors.split('.').reverse()[0] as NodeType

    const { colour, borderColour, textColour } = nodeUi(nodeType)

    const styles = apply([
      'relative',
      'inline-block',
      'cursor-default',
      `bg-[${colour}]`,
      `text-[${textColour}] text-2xs leading-none`,
      'px-2 py-1',
      `border border-[${textColour}]/0 rounded-full`,
      'transition-all duration-200 ease-in',
      `hover:bg-[${borderColour}] hover:border-[${colour}]`,
    ])

    // A percentage of 0 means <1%
    const percent = this.characterPercent === 0 ? '<1' : this.characterPercent

    return html`<div
      class="relative flex items-center"
      @click=${(e: Event) => e.stopImmediatePropagation()}
    >
      <sl-tooltip
        content=${this.tooltipText()}
        trigger="manual"
        ${ref(this.tooltipRef)}
        ><strong class=${styles} ${ref(this.buttonRef)}>
          <div
            class="font-normal pointer-events-none inline-flex items-center gap-x-1"
          >
            ${renderProvenanceCategory(this.provenanceCategory)}${percent}%
          </div></strong
        ></sl-tooltip
      >
    </div>`
  }

  override firstUpdated(changedProperties: PropertyValues) {
    super.firstUpdated(changedProperties)

    const tooltip = this.tooltipText()

    if (this.buttonRef.value) {
      this.buttonRef.value.addEventListener('mouseover', () => {
        this.tooltipRef.value.open = tooltip !== undefined
      })

      this.buttonRef.value.addEventListener('mouseout', () => {
        this.tooltipRef.value.open = false
      })
    }
  }

  private tooltipText() {
    return provenanceCategories[this.provenanceCategory]
  }
}
