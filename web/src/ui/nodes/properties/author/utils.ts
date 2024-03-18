import { API_ICONS } from '../../../config/icons'

/**
 * Defines the available icons for software applications (both stencila and
 * external assistants).
 */

export type SoftwareIcon = Readonly<{
  icon: string
  library: string
}>

type IconDefinition = {
  [index: string]: {
    readonly icon: string
    readonly library: string
  }
}

export const stencilaIcons = {
  insert: {
    icon: 'ai-sparkle',
    library: 'stencila',
  },
  edit: {
    icon: 'pencil',
    library: 'stencila',
  },
} as const

export const assistantIcons = (() => {
  return (
    Object.keys(API_ICONS) as (keyof typeof API_ICONS)[]
  ).reduce<IconDefinition>((acc, key) => {
    const icon = API_ICONS[key]
    acc[icon] = {
      icon,
      library: 'stencila',
    }
    return acc
  }, {})
})()
