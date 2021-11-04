import { IconNames } from '@stencila/components'

const iconNameMap: Record<string, IconNames> = {
  strong: 'bold',
  em: 'italic',
  heading: 'heading',
  paragraph: 'text',
  blockquote: 'double-quotes-r',
  codeBlock: 'code-view',
}

// Helper function to create menu icons
export const menuButton = (
  text: string,
  name: string
): HTMLStencilaButtonElement => {
  const iconEl = document.createElement('stencila-button')
  iconEl.icon = iconNameMap[name]

  iconEl.color = 'key'
  iconEl.minimal = true
  iconEl.buttonType = 'button'
  iconEl.size = 'small'

  iconEl.iconOnly = true
  iconEl.title = text
  iconEl.textContent = name
  iconEl.tooltip = name

  return iconEl
}
