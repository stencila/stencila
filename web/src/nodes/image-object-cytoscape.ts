import { html } from 'lit'

import { buildCytoscapeTheme } from '../utilities/cytoscapeTheme'
export type { CytoscapeTheme } from '../utilities/cytoscapeTheme'

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
 *
 * Uses `.cytoscape-container` class from shared styles (image-object-styles.ts)
 */
export function renderCytoscapeContainer() {
  return html`<div class="cytoscape-container" id="stencila-cytoscape-container"></div>`
}
