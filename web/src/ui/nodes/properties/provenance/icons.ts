import { ProvenanceCategory } from '@stencila/types'
import { html } from 'lit'

/**
 * Provide access to status icons for rendering provenance categories
 */

type IconSize = '2xs' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'

type ProvenanceStatusIcons = {
  [Property in ProvenanceCategory]: {
    actors: (typeof human | typeof machine)[]
    verified?: typeof machineVerified | typeof humanVerified
  }
}

const human = {
  icon: 'person',
} as const

const machine = {
  icon: 'robot',
} as const

const humanVerified = {
  icon: 'checkCircleFill',
} as const

const machineVerified = {
  icon: 'checkCircle',
} as const

export const provenanceStatusIcons: ProvenanceStatusIcons = {
  HwHeHv: {
    actors: [human, human],
    verified: humanVerified,
  },
  HwHe: {
    actors: [human, human],
  },
  HwHv: {
    actors: [human],
  },
  Hw: {
    actors: [human],
  },
  HwMv: {
    actors: [human],
    verified: machineVerified,
  },
  MwHeHv: {
    actors: [machine, human],
    verified: humanVerified,
  },
  MwHe: {
    actors: [machine, human],
  },
  MwHeMv: {
    actors: [machine, human],
    verified: machineVerified,
  },
  HwMeHv: {
    actors: [human, machine],
    verified: humanVerified,
  },
  HwMe: {
    actors: [human, machine],
  },
  HwMeMv: {
    actors: [human, machine],
    verified: machineVerified,
  },
  MwHv: {
    actors: [machine],
    verified: humanVerified,
  },
  MwMeHv: {
    actors: [machine, machine],
    verified: humanVerified,
  },
  Mw: {
    actors: [machine],
  },
  MwMv: {
    actors: [machine],
    verified: machineVerified,
  },
  MwMe: {
    actors: [machine, machine],
  },
  MwMeMv: {
    actors: [machine, machine],
    verified: machineVerified,
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

  const { actors, verified } = statusIcons

  return html`${actors.map(({ icon }) => {
    return html`<stencila-ui-icon
      name=${icon}
      class=${textSize}
    ></stencila-ui-icon>`
  })}${verified &&
  html`<stencila-ui-icon
    name=${verified.icon}
    class=${textSize}
  ></stencila-ui-icon>`}`
}
