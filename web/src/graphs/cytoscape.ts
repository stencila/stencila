/**
 * Cytoscape graph rendering adapters.
 *
 * This module converts the graph view model into Cytoscape configuration rather
 * than letting the web component assemble rendering details inline. Keeping this
 * adapter separate makes the view component responsible for interaction state
 * only, while graph styling, element metadata, and layout mapping remain easy to
 * test and reuse.
 */
import type {
  CytoscapeOptions,
  ElementDefinition,
  LayoutOptions,
} from 'cytoscape'

import type { CytoscapeTheme } from '../utilities/cytoscapeTheme'

import type { GraphLayout, GraphView } from './types'

/**
 * Build Cytoscape options for a graph view.
 *
 * Cytoscape expects its data, stylesheet, layout, and interaction limits in one
 * object. This function keeps those choices centralized so graph views use the
 * same zoom behavior, theme tokens, element data fields, and preset layouts no
 * matter where the Cytoscape instance is created.
 */
export function toCytoscapeOptions(
  view: GraphView,
  container: HTMLElement,
  layout: GraphLayout,
  theme: CytoscapeTheme
): CytoscapeOptions {
  return {
    container,
    elements: toElements(view),
    style: stylesheet(theme),
    layout: layoutOptions(layout),
    minZoom: 0.1,
    maxZoom: 4,
    wheelSensitivity: 0.2,
  }
}

/**
 * Convert a graph view to Cytoscape elements.
 *
 * The projected view model carries Stencila-specific metadata, while Cytoscape
 * renders nodes and edges using flat element definitions. This conversion keeps
 * stable IDs and summary fields on each element so styles, labels, selection,
 * and later inspection features can all work from the same canonical data.
 */
export function toElements(view: GraphView): ElementDefinition[] {
  return [
    ...view.nodes.map((node) => ({
      group: 'nodes' as const,
      data: {
        id: node.id,
        label: node.label,
        kind: node.kind,
      },
      classes: `node-${node.kind}`,
    })),
    ...view.edges.map((edge) => ({
      group: 'edges' as const,
      data: {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label,
        kind: edge.kind,
        count: edge.count,
        evidenceCount: edge.evidenceCount,
        actionCount: edge.actionCount,
        lowConfidence: edge.lowConfidence,
      },
      classes: [
        `edge-${edge.kind}`,
        edge.count > 1 ? 'edge-aggregate' : undefined,
        edge.lowConfidence ? 'edge-low-confidence' : undefined,
      ]
        .filter(Boolean)
        .join(' '),
    })),
  ]
}

/**
 * Build the Cytoscape stylesheet.
 *
 * Styles live here because node and edge classes are produced by this adapter,
 * not by the Lit view. The class palette distinguishes graph concepts at a
 * glance, while theme-derived base colors keep embedded and full-page graphs
 * aligned with the active document theme.
 */
function stylesheet(theme: CytoscapeTheme): CytoscapeOptions['style'] {
  return [
    {
      selector: 'core',
      style: {
        'active-bg-color': theme.activeBackground,
        'active-bg-opacity': 0.14,
        'selection-box-color': theme.activeBackground,
        'selection-box-opacity': 0.18,
        'selection-box-border-color': theme.activeBorderColor,
      },
    },
    {
      selector: 'node',
      style: {
        label: 'data(label)',
        'background-color': theme.nodeBackground,
        'border-color': theme.nodeBorderColor,
        'border-width': theme.nodeBorderWidth,
        color: theme.nodeTextColor,
        'font-family': theme.fontFamily,
        'font-size': theme.fontSize,
        'text-max-width': 130,
        'text-wrap': 'wrap',
        'text-valign': 'center',
        'text-halign': 'center',
        height: 44,
        padding: 10,
        shape: 'round-rectangle',
        width: 'label',
      },
    },
    {
      selector: 'edge',
      style: {
        label: 'data(label)',
        color: theme.textColor,
        'curve-style': 'bezier',
        'font-family': theme.fontFamily,
        'font-size': '10px',
        'line-color': theme.edgeColor,
        'target-arrow-color': theme.edgeColor,
        'target-arrow-shape': 'triangle',
        'text-background-color': theme.background,
        'text-background-opacity': 0.85,
        'text-background-padding': 2,
        'text-rotation': 'autorotate',
        width: theme.edgeWidth,
      },
    },
    {
      selector: '.node-code',
      style: {
        'background-color': '#eef6f4',
        'border-color': '#187b6b',
      },
    },
    {
      selector: '.node-symbol',
      style: {
        'background-color': '#eef6f4',
        'border-color': '#187b6b',
        shape: 'ellipse',
      },
    },
    {
      selector: '.node-function',
      style: {
        'background-color': '#eaf4fb',
        'border-color': '#2870a6',
        shape: 'diamond',
      },
    },
    {
      selector: '.node-datatable',
      style: {
        'background-color': '#edf7ee',
        'border-color': '#3f7f48',
        shape: 'rectangle',
      },
    },
    {
      selector: '.node-package',
      style: {
        'background-color': '#f8f0dd',
        'border-color': '#a06d15',
        shape: 'hexagon',
      },
    },
    {
      selector: '.node-reference',
      style: {
        'background-color': '#f1eef8',
        'border-color': '#6f4fb0',
        shape: 'tag',
      },
    },
    {
      selector: '.node-citation',
      style: {
        'background-color': '#fff4f1',
        'border-color': '#b64d38',
        shape: 'ellipse',
      },
    },
    {
      selector: '.node-content',
      style: {
        'background-color': '#f7f4ea',
        'border-color': '#8a6f2a',
      },
    },
    {
      selector: '.node-workspace',
      style: {
        'background-color': theme.nodeBackgroundSecondary,
        'border-style': 'dashed',
      },
    },
    {
      selector: '.node-environment',
      style: {
        'background-color': '#eef7e8',
        'border-color': '#4d7c2f',
        'border-style': 'dashed',
      },
    },
    {
      selector: '.node-resource',
      style: {
        'background-color': '#eef2f7',
        'border-color': '#516981',
      },
    },
    {
      selector: '.node-output',
      style: {
        'background-color': '#f2f5f8',
        'border-color': '#647486',
        shape: 'cut-rectangle',
      },
    },
    {
      selector: '.edge-PartOf',
      style: {
        'line-style': 'dotted',
        opacity: 0.45,
      },
    },
    {
      selector: '.edge-low-confidence',
      style: {
        'line-style': 'dashed',
        opacity: 0.58,
      },
    },
    {
      selector: '.edge-aggregate',
      style: {
        'text-background-opacity': 0.95,
      },
    },
    {
      selector: ':selected',
      style: {
        'background-color': theme.activeBackground,
        'border-color': theme.activeBorderColor,
        'line-color': theme.activeBorderColor,
        'target-arrow-color': theme.activeBorderColor,
      },
    },
  ] as CytoscapeOptions['style']
}

/**
 * Map a graph layout name to Cytoscape layout options.
 *
 * The UI exposes a small stable vocabulary rather than raw Cytoscape settings.
 * Translating that vocabulary here lets the controls stay compact while still
 * tuning each layout for readable document graphs with labels included.
 */
function layoutOptions(layout: GraphLayout): LayoutOptions {
  switch (layout) {
    case 'breadthfirst':
      return {
        name: 'breadthfirst',
        directed: true,
        fit: true,
        grid: true,
        padding: 40,
        spacingFactor: 1.45,
      } as LayoutOptions
    case 'cose':
      return {
        name: 'cose',
        animate: false,
        fit: true,
        nodeDimensionsIncludeLabels: true,
        padding: 40,
      } as LayoutOptions
    case 'grid':
      return {
        name: 'grid',
        fit: true,
        padding: 40,
      }
    case 'circle':
      return {
        name: 'circle',
        fit: true,
        padding: 40,
      }
  }
}
