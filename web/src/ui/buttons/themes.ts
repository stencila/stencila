/**
 * Supplies a "theme" to a button. Specifies:
 * - Initial bg, text & border
 * - Hover bg, text & border
 */
export type ButtonTheme = {
  bgDefault?: string
  bgHover?: string
  textDefault?: string
  textHover?: string
  borderDefault?: string
  borderHover?: string
  textDecorationDefault?: string
  textDecorationHover?: string
  px?: string
}

export const BlueSolid: ButtonTheme = {
  bgDefault: 'blue-700',
  bgHover: 'blue-800',
  textDefault: 'white',
  textHover: 'white',
}

/**
 * Theme for a blue button with no border, bg but uses underline on hover
 */
export const BlueTextInline: ButtonTheme = {
  bgDefault: 'transparent',
  bgHover: 'transparent',
  textDefault: 'blue-800',
  textHover: 'blue-900',
  borderDefault: 'transparent',
  borderHover: 'transparent',
  textDecorationDefault: 'none',
  textDecorationHover: 'underline',
}

export const BlueTextInlineSmall: ButtonTheme = {
  ...BlueTextInline,
  px: '24px',
}

export const ButtonThemes = {
  'blue-solid': BlueSolid,
  'blue-inline-text': BlueTextInline,
  'blue-inline-text--small': BlueTextInlineSmall,
}
