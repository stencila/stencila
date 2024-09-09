import { ProvenanceCategory } from '@stencila/types'
import { html } from 'lit'

/**
 * Render icons representing a Stencila `ProvenanceCategory`
 */
export const renderProvenanceCategory = (category: ProvenanceCategory) => {
  const icons = provenanceIcons[category]

  if (!icons) {
    // If typing is correct this should never happen, but in case it
    // does, fallback to text repr of category
    return html`<span class="text-sm">${category}</span>`
  }

  return html`${icons.map((icon) => {
    return html`<stencila-ui-icon
      name=${icon}
      class="text-sm"
    ></stencila-ui-icon>`
  })}`
}

// Icons associated with each component of the category
const Hw = 'person'
const He = 'person'
const Mw = 'robot'
const Me = 'robot'
const Hv = 'checkCircleFill'
const Mv = 'checkCircle'

type ProvenanceIcon =
  | typeof Hw
  | typeof Mw
  | typeof He
  | typeof Me
  | typeof Hv
  | typeof Mv

export const provenanceIcons: Record<ProvenanceCategory, ProvenanceIcon[]> = {
  HwHeHv: [Hw, He, Hv],
  HwHe: [Hw, He],
  HwHv: [Hw, Hv],
  Hw: [Hw],
  HwMv: [Hw, Mv],
  MwHeHv: [Mw, He, Hv],
  MwHe: [Mw, He],
  MwHeMv: [Mw, He, Mv],
  HwMeHv: [Hw, Me, Hv],
  HwMe: [Hw, Me],
  HwMeMv: [Hw, Me, Mv],
  MwHv: [Mw, Hv],
  MwMeHv: [Mw, Me, Hv],
  Mw: [Mw],
  MwMv: [Mw, Mv],
  MwMe: [Mw, Me],
  MwMeMv: [Mw, Me, Mv],
}
