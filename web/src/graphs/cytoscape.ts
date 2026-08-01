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

import { edgeDisplayLabel, edgeRole, evidenceMarker } from './evidence'
import { producedOutputIds } from './lineage'
import type { GraphLayout, GraphLayoutSpacing, GraphView } from './types'

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
  spacing: GraphLayoutSpacing,
  theme: CytoscapeTheme
): CytoscapeOptions {
  return {
    container,
    elements: toElements(view),
    style: stylesheet(theme),
    layout: layoutOptions(layout, spacing),
    autounselectify: true,
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
  const outputIds = producedOutputIds(view)

  return [
    ...view.nodes.map((node) => ({
      group: 'nodes' as const,
      data: {
        id: node.id,
        label: node.label,
        kind: node.kind,
        producedOutput: outputIds.has(node.id),
      },
      classes: [
        `node-${node.kind}`,
        outputIds.has(node.id) ? 'node-produced-output' : undefined,
      ]
        .filter(Boolean)
        .join(' '),
    })),
    ...view.edges.map((edge) => {
      const marker = evidenceMarker(edge)
      const role = edgeRole(edge.kind)

      return {
        group: 'edges' as const,
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          displayLabel: edgeDisplayLabel(edge),
          kind: edge.kind,
          role,
          count: edge.count,
          evidenceCount: edge.evidenceCount,
          evidenceMarker: marker,
          attested: marker === 'attested',
          multiplicity: edge.evidenceCount > 1 ? edge.evidenceCount : 0,
          actionCount: edge.actionCount,
          lowConfidence: edge.lowConfidence,
        },
        classes: [
          `edge-${edge.kind}`,
          `edge-role-${role}`,
          edge.count > 1 ? 'edge-aggregate' : undefined,
          edge.lowConfidence ? 'edge-low-confidence' : undefined,
        ]
          .filter(Boolean)
          .join(' '),
      }
    }),
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
  const colors = semanticColors(theme)

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
        'shadow-blur': 8,
        'shadow-color': colors.shadow,
        'shadow-offset-y': 2,
        'shadow-opacity': 0.18,
        width: 'label',
      },
    },
    {
      selector: 'edge',
      style: {
        label: 'data(displayLabel)',
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
        'text-margin-y': -9,
        'text-rotation': 'autorotate',
        width: theme.edgeWidth,
      },
    },
    {
      selector: '.node-code',
      style: {
        'background-color': colors.tealFill,
        'border-color': colors.teal,
        padding: 15,
        shape: 'hexagon',
      },
    },
    {
      selector: '.node-symbol',
      style: {
        'background-color': colors.tealFill,
        'border-color': colors.teal,
        padding: 14,
        shape: 'ellipse',
      },
    },
    {
      selector: '.node-function',
      style: {
        'background-color': colors.tealFill,
        'border-color': colors.teal,
        padding: 15,
        shape: 'hexagon',
      },
    },
    {
      selector: '.node-datatable',
      style: {
        'background-color': colors.tealFill,
        'border-color': colors.teal,
        padding: 14,
        shape: 'ellipse',
      },
    },
    {
      selector: '.node-package',
      style: {
        'background-color': colors.amberFill,
        'border-color': colors.amber,
        padding: 12,
        shape: 'cut-rectangle',
      },
    },
    {
      selector: '.node-reference',
      style: {
        'background-color': colors.violetFill,
        'border-color': colors.violet,
        padding: 13,
        shape: 'tag',
      },
    },
    {
      selector: '.node-citation',
      style: {
        'background-color': colors.violetFill,
        'border-color': colors.violet,
        padding: 13,
        shape: 'tag',
      },
    },
    {
      selector: '.node-document, .node-resource',
      style: {
        'background-color': colors.blueFill,
        'border-color': colors.blue,
      },
    },
    {
      selector: '.node-content',
      style: {
        'background-color': colors.grayFill,
        'border-color': colors.gray,
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
        'background-color': colors.amberFill,
        'border-color': colors.amber,
        padding: 12,
        shape: 'cut-rectangle',
      },
    },
    {
      selector: '.node-output',
      style: {
        'background-color': colors.violetFill,
        'border-color': colors.violet,
      },
    },
    {
      selector: '.node-produced-output',
      style: {
        'background-color': colors.violetStrongFill,
        'border-color': colors.violet,
        'border-width': 3,
        'shadow-blur': 14,
        'shadow-color': colors.violet,
        'shadow-opacity': 0.28,
      },
    },
    {
      selector: '.edge-role-input',
      style: {
        'line-color': colors.blue,
        'target-arrow-color': colors.blue,
      },
    },
    {
      selector: '.edge-role-derivation',
      style: {
        'line-color': colors.teal,
        'target-arrow-color': colors.teal,
      },
    },
    {
      selector: '.edge-role-output',
      style: {
        'line-color': colors.violet,
        'target-arrow-color': colors.violet,
        width: 2,
      },
    },
    {
      selector: '.edge-role-software',
      style: {
        'line-color': colors.amber,
        'target-arrow-color': colors.amber,
      },
    },
    {
      selector: '.edge-role-structure, .edge-role-discourse',
      style: {
        'line-color': colors.gray,
        'target-arrow-color': colors.gray,
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
      selector: '.lineage-dim',
      style: {
        opacity: 0.16,
      },
    },
    {
      selector:
        '.lineage-upstream-node, .lineage-downstream-node, .lineage-overlap-node, .lineage-upstream-edge, .lineage-downstream-edge, .lineage-overlap-edge, .lineage-selected, .inspector-selected-node, .inspector-selected-edge',
      style: {
        opacity: 1,
      },
    },
    {
      selector:
        '.lineage-upstream-node, .lineage-downstream-node, .lineage-overlap-node, .lineage-selected',
      style: {
        'border-color': colors.teal,
        'border-width': 3,
        'shadow-blur': 13,
        'shadow-color': colors.teal,
        'shadow-opacity': 0.3,
      },
    },
    {
      selector:
        '.lineage-upstream-edge, .lineage-downstream-edge, .lineage-overlap-edge',
      style: {
        opacity: 1,
        width: 3,
      },
    },
    {
      selector: '.inspector-selected-node',
      style: {
        'underlay-color': theme.activeBorderColor,
        'underlay-opacity': 0.9,
        'underlay-padding': 5,
        'underlay-shape': 'round-rectangle',
        'z-index': 10,
      },
    },
    {
      selector: '.inspector-selected-edge',
      style: {
        opacity: 1,
        width: 4,
        'underlay-color': theme.activeBorderColor,
        'underlay-opacity': 0.62,
        'underlay-padding': 4,
        'z-index': 10,
      },
    },
  ] as CytoscapeOptions['style']
}

function semanticColors(theme: CytoscapeTheme) {
  const dark = relativeLuminance(theme.background) < 0.35

  return dark
    ? {
        blue: '#60a5fa',
        blueFill: '#172b45',
        teal: '#2dd4bf',
        tealFill: '#143b39',
        violet: '#c4b5fd',
        violetFill: '#302653',
        violetStrongFill: '#44346f',
        amber: '#fbbf24',
        amberFill: '#463716',
        gray: '#8291a6',
        grayFill: '#27313e',
        shadow: '#000000',
      }
    : {
        blue: '#2563a9',
        blueFill: '#eaf2fb',
        teal: '#0f766e',
        tealFill: '#e6f5f2',
        violet: '#7053b6',
        violetFill: '#f0ecfa',
        violetStrongFill: '#e2d9f7',
        amber: '#a16207',
        amberFill: '#fbf1d8',
        gray: '#718096',
        grayFill: '#f1f4f7',
        shadow: '#334155',
      }
}

function relativeLuminance(hex: string): number {
  const match = /^#([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i.exec(hex)
  if (!match) {
    return 1
  }

  const [, red, green, blue] = match
  return (
    0.2126 * parseInt(red, 16) +
    0.7152 * parseInt(green, 16) +
    0.0722 * parseInt(blue, 16)
  ) / 255
}

/**
 * Map a graph layout name to Cytoscape layout options.
 *
 * The UI exposes a small stable vocabulary rather than raw Cytoscape settings.
 * Translating that vocabulary here lets the controls stay compact while still
 * tuning each layout for readable document graphs with labels included.
 */
export function layoutOptions(
  layout: GraphLayout,
  spacing: GraphLayoutSpacing
): LayoutOptions {
  const spacingFactor = {
    compact: 0.85,
    cozy: 1,
    balanced: 1.2,
    open: 1.4,
    spacious: 1.65,
  }[spacing]

  switch (layout) {
    case 'breadthfirst':
      return {
        name: 'breadthfirst',
        directed: true,
        fit: true,
        grid: true,
        circle: false,
        padding: 40,
        roots: undefined,
        spacingFactor,
        transform: (_node, position) => ({ x: position.y, y: position.x }),
      } as LayoutOptions
    case 'cose':
      {
        const forceSpacing = {
          compact: {
            componentSpacing: 24,
            idealEdgeLength: 28,
            nodeRepulsion: 1000,
          },
          cozy: {
            componentSpacing: 32,
            idealEdgeLength: 36,
            nodeRepulsion: 1600,
          },
          balanced: {
            componentSpacing: 40,
            idealEdgeLength: 48,
            nodeRepulsion: 2400,
          },
          open: {
            componentSpacing: 52,
            idealEdgeLength: 64,
            nodeRepulsion: 3600,
          },
          spacious: {
            componentSpacing: 68,
            idealEdgeLength: 84,
            nodeRepulsion: 5200,
          },
        }[spacing]

        return {
          name: 'cose',
          animate: false,
          fit: true,
          nodeDimensionsIncludeLabels: true,
          padding: 40,
          ...forceSpacing,
        } as LayoutOptions
      }
    case 'grid':
      return {
        name: 'grid',
        fit: true,
        padding: 40,
        spacingFactor,
      }
    case 'circle':
      return {
        name: 'circle',
        fit: true,
        padding: 40,
        spacingFactor,
      }
  }
}
