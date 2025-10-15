import { css } from '@twind/core'
import { html } from 'lit'

import { colorToHex } from '../utilities/colorUtils'
import { getCSSVariables } from '../utilities/cssVariables'

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
 * Build Cytoscape theme from CSS custom properties (with caching)
 */
export function buildCytoscapeTheme(rootElement: HTMLElement): CytoscapeTheme {
  // Return cached theme if available
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

  // Provide fallback values for critical properties
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

  // Build and cache the theme
  cachedTheme = {
    // Canvas styles
    background: colorToHex(cssVars.background) || fallbacks.background,

    // Node styles
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

    // Edge styles
    edgeColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,
    edgeWidth: cssVars.edgeWidth || '1px',
    textColor: colorToHex(cssVars.textColor) || fallbacks.textColor,

    // State styles
    activeBackground:
      colorToHex(cssVars.activeBackground) ||
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      fallbacks.nodeBackground,
    activeBorderColor:
      colorToHex(cssVars.activeBorderColor) ||
      colorToHex(cssVars.nodeBorderColor) ||
      fallbacks.nodeBorderColor,

    // Typography
    fontFamily: cssVars.fontFamily || fallbacks.fontFamily,
    fontSize: cssVars.fontSize || fallbacks.fontSize,
  }

  return cachedTheme
}

/**
 * Compile and render a Cytoscape graph
 */
export async function compileCytoscape(
  contentUrl: string,
  container: HTMLElement,
  isStaticView: boolean
): Promise<unknown> {
  const { default: cytoscape } = await import('cytoscape')

  const graph = JSON.parse(contentUrl)
  graph.container = container

  // Configure for static mode if enabled
  if (isStaticView) {
    graph.userZoomingEnabled = false
    graph.userPanningEnabled = false
    graph.boxSelectionEnabled = false
    graph.autoungrabify = true
  }

  // Apply theme styles
  const theme = buildCytoscapeTheme(container)
  const themeStyle = [
    {
      selector: 'node',
      style: {
        'background-color': theme.nodeBackground,
        'border-color': theme.nodeBorderColor,
        'border-width': theme.nodeBorderWidth,
        color: theme.nodeTextColor,
        'font-family': theme.fontFamily,
        'font-size': theme.fontSize,
      },
    },
    {
      selector: 'node:selected',
      style: {
        'background-color': theme.activeBackground,
        'border-color': theme.activeBorderColor,
      },
    },
    {
      selector: 'node.secondary',
      style: {
        'background-color': theme.nodeBackgroundSecondary,
      },
    },
    {
      selector: 'node.tertiary',
      style: {
        'background-color': theme.nodeBackgroundTertiary,
      },
    },
    {
      selector: 'edge',
      style: {
        'line-color': theme.edgeColor,
        width: theme.edgeWidth,
        'target-arrow-color': theme.edgeColor,
        color: theme.textColor,
        'font-family': theme.fontFamily,
        'font-size': theme.fontSize,
      },
    },
  ]

  // Merge theme styles with existing styles if any
  if (graph.style) {
    if (Array.isArray(graph.style)) {
      graph.style = [...themeStyle, ...graph.style]
    } else {
      graph.style = [...themeStyle, graph.style]
    }
  } else {
    graph.style = themeStyle
  }

  return cytoscape(graph)
}

/**
 * Render Cytoscape container
 */
export function renderCytoscapeContainer() {
  const containerStyles = css`
    & {
      position: relative;
      width: 100%;
      aspect-ratio: 1;
    }
  `
  return html`
    <div slot="content" class="overflow-x-auto">
      <div class=${containerStyles} id="stencila-cytoscape-container"></div>
    </div>
  `
}
