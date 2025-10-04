import { html } from 'lit'
import { css } from '@twind/core'
import { unsafeSVG } from 'lit/directives/unsafe-svg'
import { colorToHex } from '../utilities/colorUtils'
import { getCSSVariables } from '../utilities/cssVariables'

/**
 * Module-level theme cache
 */
let cachedTheme: MermaidTheme | null = null

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
 * Mermaid theme object built from CSS custom properties
 */
export interface MermaidTheme {
  background: string
  fontFamily: string
  fontSize: string
  textColor: string
  primaryColor: string
  primaryTextColor: string
  primaryBorderColor: string
  secondaryColor: string
  tertiaryColor: string
  nodeBorder: string
  nodeBorderWidth: string
  lineColor: string
  edgeWidth: string
  actorBkg: string
  actorBorder: string
  actorTextColor: string
  signalColor: string
  signalTextColor: string
  activationBkgColor: string
  activationBorderColor: string
  labelBoxBkgColor: string
  labelTextColor: string
  stateBkg: string
  stateLabelColor: string
  transitionColor: string
  transitionLabelColor: string
  specialStateColor: string
  arrowheadColor: string
  fillType0: string
  fillType1: string
  fillType2: string
  taskBkgColor: string
  taskBorderColor: string
  activeTaskBkgColor: string
  activeTaskBorderColor: string
  doneTaskBkgColor: string
  doneTaskBorderColor: string
  gridColor: string
  sectionBkgColor: string
  altSectionBkgColor: string
  clusterBkg: string
  edgeLabelBackground: string
}

/**
 * Build Mermaid theme from CSS custom properties (with caching)
 */
export function buildMermaidTheme(rootElement: HTMLElement): MermaidTheme {
  // Return cached theme if available
  if (cachedTheme) {
    return cachedTheme
  }

  const cssVars = getCSSVariables(rootElement, {
    // Core diagram properties
    background: '--diagram-background',
    fontFamily: '--diagram-font-family',
    fontSize: '--diagram-font-size',
    textColor: '--diagram-text-color',

    // Node properties
    nodeBackground: '--diagram-node-background',
    nodeBackgroundSecondary: '--diagram-node-background-secondary',
    nodeBackgroundTertiary: '--diagram-node-background-tertiary',
    nodeBorderColor: '--diagram-node-border-color',
    nodeBorderWidth: '--diagram-node-border-width',
    nodeTextColor: '--diagram-node-text-color',

    // Edge properties
    edgeColor: '--diagram-edge-color',
    edgeWidth: '--diagram-edge-width',
    edgeTextColor: '--diagram-edge-text-color',

    // State properties
    activeBackground: '--diagram-active-background',
    activeBorderColor: '--diagram-active-border-color',
    inactiveBackground: '--diagram-inactive-background',
    inactiveBorderColor: '--diagram-inactive-border-color',
    completeBackground: '--diagram-complete-background',
    completeBorderColor: '--diagram-complete-border-color',

    // Structure properties
    gridColor: '--diagram-grid-color',
    sectionBackground: '--diagram-section-background',
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
    // Core Mermaid theme variables
    background: colorToHex(cssVars.background) || fallbacks.background,
    fontFamily: cssVars.fontFamily || fallbacks.fontFamily,
    fontSize: cssVars.fontSize || fallbacks.fontSize,
    textColor: colorToHex(cssVars.textColor) || fallbacks.textColor,

    // Primary colors for nodes
    primaryColor: colorToHex(cssVars.nodeBackground) || fallbacks.nodeBackground,
    primaryTextColor:
      colorToHex(cssVars.nodeTextColor) || fallbacks.nodeTextColor,
    primaryBorderColor:
      colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,

    // Secondary/tertiary colors for multi-state nodes
    secondaryColor:
      colorToHex(cssVars.nodeBackgroundSecondary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    tertiaryColor:
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,

    // Node & edge colors
    nodeBorder: colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,
    nodeBorderWidth: cssVars.nodeBorderWidth || '1px',
    lineColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,
    edgeWidth: cssVars.edgeWidth || '1px',

    // Sequence diagram variables
    actorBkg: colorToHex(cssVars.nodeBackground) || fallbacks.nodeBackground,
    actorBorder: colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,
    actorTextColor:
      colorToHex(cssVars.nodeTextColor) || fallbacks.nodeTextColor,
    signalColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,
    signalTextColor: colorToHex(cssVars.edgeTextColor) || fallbacks.textColor,
    activationBkgColor:
      colorToHex(cssVars.nodeBackgroundSecondary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    activationBorderColor:
      colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,
    labelBoxBkgColor: 'transparent',
    labelTextColor: colorToHex(cssVars.edgeTextColor) || fallbacks.textColor,

    // State diagram variables
    stateBkg: colorToHex(cssVars.nodeBackground) || fallbacks.nodeBackground,
    stateLabelColor:
      colorToHex(cssVars.nodeTextColor) || fallbacks.nodeTextColor,
    transitionColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,
    transitionLabelColor:
      colorToHex(cssVars.edgeTextColor) || fallbacks.textColor,
    specialStateColor:
      colorToHex(cssVars.nodeBorderColor) || fallbacks.nodeBorderColor,
    arrowheadColor: colorToHex(cssVars.edgeColor) || fallbacks.edgeColor,

    // Additional state fills
    fillType0: colorToHex(cssVars.nodeBackground) || fallbacks.nodeBackground,
    fillType1:
      colorToHex(cssVars.nodeBackgroundSecondary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    fillType2:
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,

    // Gantt chart variables
    taskBkgColor:
      colorToHex(cssVars.inactiveBackground) ||
      colorToHex(cssVars.nodeBackground) ||
      fallbacks.nodeBackground,
    taskBorderColor:
      colorToHex(cssVars.inactiveBorderColor) ||
      colorToHex(cssVars.nodeBorderColor) ||
      fallbacks.nodeBorderColor,
    activeTaskBkgColor:
      colorToHex(cssVars.activeBackground) ||
      colorToHex(cssVars.nodeBackgroundTertiary) ||
      fallbacks.nodeBackground,
    activeTaskBorderColor:
      colorToHex(cssVars.activeBorderColor) ||
      colorToHex(cssVars.nodeBorderColor) ||
      fallbacks.nodeBorderColor,
    doneTaskBkgColor:
      colorToHex(cssVars.completeBackground) ||
      colorToHex(cssVars.nodeBackgroundSecondary) ||
      fallbacks.nodeBackground,
    doneTaskBorderColor:
      colorToHex(cssVars.completeBorderColor) ||
      colorToHex(cssVars.nodeBorderColor) ||
      fallbacks.nodeBorderColor,
    gridColor:
      colorToHex(cssVars.gridColor) ||
      colorToHex(cssVars.edgeColor) ||
      fallbacks.edgeColor,
    sectionBkgColor:
      colorToHex(cssVars.sectionBackground) || fallbacks.background,
    altSectionBkgColor: colorToHex(cssVars.background) || fallbacks.background,

    // Cluster/subgraph styling
    clusterBkg: colorToHex(cssVars.sectionBackground) || fallbacks.background,
    edgeLabelBackground: 'transparent',
  }

  return cachedTheme
}

/**
 * Compile Mermaid diagram to SVG
 */
export async function compileMermaid(
  contentUrl: string,
  element: HTMLElement,
  onSuccess: (svg: string) => void,
  onError: (error: any) => void
): Promise<void> {
  // Import Mermaid dynamically
  // @ts-expect-error - Using ESM min build to avoid parser dependency issues
  const { default: mermaid } = await import('mermaid/dist/mermaid.esm.min.mjs')

  // Initialize Mermaid with theme
  const theme = buildMermaidTheme(element)
  mermaid.initialize({
    theme: 'base',
    themeVariables: theme,
  })

  const container = document.createElement('div')
  document.body.appendChild(container)

  try {
    const id = 'stencila-' + Math.random().toString(36).substring(2)
    const result = await mermaid.render(id, contentUrl, container)
    onSuccess(result.svg)
  } catch (error) {
    // Hide the container so that the Mermaid bomb message does not appear
    container.style.display = 'none'
    onError(error)
  }

  container.remove()
}

/**
 * Render Mermaid SVG
 */
export function renderMermaid(svg: string) {
  /**
   * Reset specific inherited styles on SVG
   *
   * Only reset properties that commonly cause issues with SVGs when inherited
   * from the document, while preserving essential SVG styling.
   *
   * Previously we used `all: initial;` to do this but that was too aggressive
   * and was breaking mermaid rending on Chrome
   */
  const svgStyles = css`
    & {
      /* Horizontally centre the SVG */
      display: flex;
      justify-content: center;
    }

    svg {
      /* Reset text-related inherited properties */
      line-height: 1;
      font-size: inherit;
      font-family: inherit;
      text-align: initial;
      letter-spacing: normal;
      word-spacing: normal;

      /* Reset box model properties that might interfere */
      margin: 0;
      padding: 0;
      border: none;

      /* Ensure SVG displays properly */
      display: block;
      max-width: 100%;
      height: auto;

      /* Fix Mermaid edge label backgrounds */
      /* Based on solution from: https://stephenkernan.com/blog/how-to-style-mermaid-edge-labels */
      foreignObject {
        &:has(.edgeLabel) {
          background-color: transparent;
          .edgeLabel,
          .labelBkg {
            background-color: transparent !important;
            /* Prevent clipping of some words. Seems to be no solved by Stephen Kernan's approach
              but because calculated width of the foreignObject is too small 🤷‍♂️ */
            font-size: 97.5%;
          }
        }
      }
    }
  `

  return html`
    <div slot="content" class=${svgStyles}>${unsafeSVG(svg)}</div>
  `
}
