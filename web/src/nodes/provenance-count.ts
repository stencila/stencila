import SlTooltip from '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { ProvenanceCategory, provenanceCategories } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../twind'

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
  private buttonRef: Ref<HTMLSpanElement> = createRef()

  override render() {
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
        >${this.provenanceCategory}
        ${this.characterPercent
          ? html`<span class="font-normal pointer-events-none"
              >${this.characterPercent}%</span
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
    return provenanceCategories[this.provenanceCategory]
  }
}
