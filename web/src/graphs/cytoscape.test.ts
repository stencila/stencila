/**
 * Cytoscape adapter tests.
 *
 * These tests pin the boundary between the projected graph view model and the
 * Cytoscape element format. That boundary matters because rendering styles and
 * future graph interactions depend on stable element data fields and classes.
 */
import { describe, expect, it } from 'vitest'

import {
  layoutOptions,
  toCytoscapeOptions,
  toElements,
} from './cytoscape'
import type { GraphView } from './types'
import type { CytoscapeTheme } from '../utilities/cytoscapeTheme'

describe('toElements', () => {
  it('converts a graph view to Cytoscape elements', () => {
    const elements = toElements({
      preset: 'data-flow',
      detail: 'medium',
      nodes: [
        {
          id: 'file:data.csv',
          label: 'data.csv',
          kind: 'resource',
          node: {
            type: 'GraphNode',
            id: 'file:data.csv',
            node: { type: 'File', name: 'data.csv', path: 'data.csv' },
          },
        },
      ],
      edges: [
        {
          id: 'ReadBy:0',
          source: 'file:data.csv',
          target: 'code:analysis.py',
          label: 'Read By',
          kind: 'ReadBy',
          edge: {
            type: 'GraphEdge',
            source: 'file:data.csv',
            target: 'code:analysis.py',
            kind: 'ReadBy',
          },
          edges: [
            {
              type: 'GraphEdge',
              source: 'file:data.csv',
              target: 'code:analysis.py',
              kind: 'ReadBy',
            },
          ],
          count: 1,
          evidenceCount: 0,
          actionCount: 0,
          lowConfidence: false,
        },
      ],
    } as GraphView)

    expect(elements).toHaveLength(2)
    expect(elements[0].data).toMatchObject({
      id: 'file:data.csv',
      label: 'data.csv',
      kind: 'resource',
      producedOutput: false,
    })
    expect(elements[1].data).toMatchObject({
      id: 'ReadBy:0',
      source: 'file:data.csv',
      target: 'code:analysis.py',
      count: 1,
      evidenceCount: 0,
      actionCount: 0,
      lowConfidence: false,
      role: 'input',
      evidenceMarker: 'none',
      attested: false,
      multiplicity: 0,
      displayLabel: 'Read By',
    })
    expect(elements[1].classes).toContain('edge-role-input')
  })
})

describe('layoutOptions', () => {
  it('uses a denser default lineage layout', () => {
    expect(layoutOptions('breadthfirst', 'cozy')).toMatchObject({
      name: 'breadthfirst',
      spacingFactor: 1,
    })
  })

  it('increases shaped and force layout spacing across the control range', () => {
    expect(layoutOptions('grid', 'compact')).toMatchObject({
      spacingFactor: 0.85,
    })
    expect(layoutOptions('grid', 'spacious')).toMatchObject({
      spacingFactor: 1.65,
    })
    expect(layoutOptions('cose', 'compact')).toMatchObject({
      componentSpacing: 24,
      idealEdgeLength: 28,
      nodeRepulsion: 1000,
    })
    expect(layoutOptions('cose', 'spacious')).toMatchObject({
      componentSpacing: 68,
      idealEdgeLength: 84,
      nodeRepulsion: 5200,
    })
  })
})

describe('node appearance', () => {
  it('uses a restrained shape and color vocabulary for node families', () => {
    const theme: CytoscapeTheme = {
      background: '#ffffff',
      nodeBackground: '#e0e0e0',
      nodeBackgroundSecondary: '#eeeeee',
      nodeBackgroundTertiary: '#f5f5f5',
      nodeBorderColor: '#999999',
      nodeBorderWidth: '1px',
      nodeTextColor: '#000000',
      edgeColor: '#666666',
      edgeWidth: '1px',
      textColor: '#000000',
      activeBackground: '#dddddd',
      activeBorderColor: '#333333',
      fontFamily: 'Inter',
      fontSize: '12px',
    }
    const styles = toCytoscapeOptions(
      {
        preset: 'full',
        detail: 'medium',
        nodes: [],
        edges: [],
      },
      {} as HTMLElement,
      'breadthfirst',
      'cozy',
      theme
    ).style as unknown as {
      selector: string
      style: Record<string, unknown>
    }[]
    const style = (selector: string) =>
      styles.find((rule) => rule.selector === selector)?.style

    expect(style('.node-code')).toMatchObject({
      'background-color': '#e6f5f2',
      shape: 'hexagon',
    })
    expect(style('.node-symbol')).toMatchObject({
      'background-color': '#e6f5f2',
      shape: 'ellipse',
    })
    expect(style('.node-package')).toMatchObject({
      'background-color': '#fbf1d8',
      shape: 'cut-rectangle',
    })
    expect(style('.node-reference')).toMatchObject({
      'background-color': '#f0ecfa',
      shape: 'tag',
    })
    expect(style('.node-document, .node-resource')).toMatchObject({
      'background-color': '#eaf2fb',
    })
    expect(style('.node-workspace')).toMatchObject({
      'border-style': 'dashed',
    })
  })
})
