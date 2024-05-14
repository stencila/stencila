import { ProvenanceCategory } from '@stencila/types'
import { html } from 'lit'

/**
 * Provide access to status icons for rendering provenance categories
 */

type IconSize = '2xs' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'

type ProvenanceStatusIcons = {
  [Property in ProvenanceCategory]: {
    icons: (typeof human | typeof machine)[]
    pass?: typeof pass | typeof filled
  }
}

const human = {
  icon: 'user',
  library: 'lucide',
} as const

const machine = {
  icon: 'bot-message-square',
  library: 'lucide',
} as const

const pass = {
  icon: 'bx-check-circle',
  library: 'boxicons',
} as const

const filled = {
  icon: 'bxs-check-circle',
  library: 'boxicons',
} as const

export const provenanceStatusIcons: ProvenanceStatusIcons = {
  HwHeHv: {
    icons: [human, human],
    pass: filled,
  },
  HwHe: {
    icons: [human, human],
  },
  HwHv: {
    icons: [human],
  },
  Hw: {
    icons: [human],
  },
  HwMv: {
    icons: [human],
    pass: pass,
  },
  MwHeHv: {
    icons: [machine, human],
    pass: filled,
  },
  MwHe: {
    icons: [machine, human],
  },
  MwHeMv: {
    icons: [machine, human],
    pass: pass,
  },
  HwMeHv: {
    icons: [human, machine],
    pass: filled,
  },
  HwMe: {
    icons: [human, machine],
  },
  HwMeMv: {
    icons: [human, machine],
    pass: pass,
  },
  MwHv: {
    icons: [machine],
    pass: filled,
  },
  MwMeHv: {
    icons: [machine, machine],
    pass: filled,
  },
  Mw: {
    icons: [machine],
  },
  MwMv: {
    icons: [machine],
    pass: pass,
  },
  MwMe: {
    icons: [machine, machine],
  },
  MwMeMv: {
    icons: [machine, machine],
    pass: pass,
  },
}

export const renderProvenanceStatus = (
  category: ProvenanceCategory,
  size: IconSize
) => {
  const textSize = `text-${size}`
  const statusIcons = provenanceStatusIcons[category]

  if (!statusIcons) {
    return null
  }

  const { icons, pass } = statusIcons

  return html`${icons.map(({ icon, library }) => {
    return html`<sl-icon
      library=${library}
      name=${icon}
      class=${textSize}
    ></sl-icon>`
  })}${pass &&
  html`<sl-icon
    library=${pass.library}
    name=${pass.icon}
    class=${textSize}
  ></sl-icon>`}`
}
