import { colorToHex } from './colorUtils'
import { getCSSVariables } from './cssVariables'

/**
 * Module-level theme cache
 */
let cachedTheme: CytoscapeTheme | null = null

/**
 * Clear cache on theme changes
 */
if (typeof window !== 'undefined') {
  window.addEventListener('stencila-color-scheme-changed', () => {
    cachedTheme = null
  })
  window.addEventListener('stencila-theme-changed', () => {
    cachedTheme = null
  })
}

/**
 * Cytoscape theme object built from CSS custom properties
 */
export interface CytoscapeTheme {
  background: string
  nodeBackground: string
  nodeBackgroundSecondary: string
  nodeBackgroundTertiary: string
  nodeBorderColor: string
  nodeBorderWidth: string
  nodeTextColor: string
  edgeColor: string
  edgeWidth: string
  textColor: string
  activeBackground: string
  activeBorderColor: string
  fontFamily: string
  fontSize: string
}

/**
 * Build Cytoscape theme from CSS custom properties.
 */
export function buildCytoscapeTheme(rootElement: HTMLElement): CytoscapeTheme {
  if (cachedTheme) {
    return cachedTheme
  }

  const cssVars = getCSSVariables(rootElement, {
    background: '--diagram-background',
    nodeBackground: '--diagram-node-background',
    nodeBackgroundSecondary: '--diagram-node-background-secondary',
    nodeBackgroundTertiary: '--diagram-node-background-tertiary',
    nodeBorderColor: '--diagram-node-border-color',
    nodeBorderWidth: '--diagram-node-border-width',
    nodeTextColor: '--diagram-node-text-color',
    edgeColor: '--diagram-edge-color',
    edgeWidth: '--diagram-edge-width',
    textColor: '--diagram-text-color',
    fontFamily: '--diagram-font-family',
    fontSize: '--diagram-font-size',
    activeBackground: '--diagram-active-background',
    activeBorderColor: '--diagram-active-border-color',
  })

  const fallbacks = {
    background: '#ffffff',
    nodeBackground: '#e0e0e0',
    nodeBorderColor: '#999999',
    nodeTextColor: '#000000',
    edgeColor: '#666666',
    textColor: '#000000',
    fontFamily: 'sans-serif',
    fontSize: '12px',
  }

  cachedTheme = {
    background: colorToHex(cssVars.background) || fallbacks.background,
    nodeBackground:
      colorToHex(cssVars.nodeBackground) || fallbacks.nodeBackground,
    nodeBackgroundSecondary:
      colorToHex(cssVars.nodeBackgroundSecondary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    nodeBackgroundTertiary:
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    nodeBorderColor:
      colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,
    nodeBorderWidth: cssVars.nodeBorderWidth || '1px',
    nodeTextColor:
      colorToHex(cssVars.nodeTextColor) || fallbacks.nodeTextColor,
    edgeColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,
    edgeWidth: cssVars.edgeWidth || '1px',
    textColor: colorToHex(cssVars.textColor) || fallbacks.textColor,
    activeBackground:
      colorToHex(cssVars.activeBackground) ||
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      fallbacks.nodeBackground,
    activeBorderColor:
      colorToHex(cssVars.activeBorderColor) ||
      colorToHex(cssVars.nodeBorderColor) ||
      fallbacks.nodeBorderColor,
    fontFamily: cssVars.fontFamily || fallbacks.fontFamily,
    fontSize: cssVars.fontSize || fallbacks.fontSize,
  }

  return cachedTheme
}
