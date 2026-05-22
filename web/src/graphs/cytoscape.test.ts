/**
 * Cytoscape adapter tests.
 *
 * These tests pin the boundary between the projected graph view model and the
 * Cytoscape element format. That boundary matters because rendering styles and
 * future graph interactions depend on stable element data fields and classes.
 */
import { describe, expect, it } from 'vitest'

import { toElements } from './cytoscape'
import type { GraphView } from './types'

describe('toElements', () => {
  it('converts a graph view to Cytoscape elements', () => {
    const elements = toElements({
      preset: 'data-flow',
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
    })
    expect(elements[1].data).toMatchObject({
      id: 'ReadBy:0',
      source: 'file:data.csv',
      target: 'code:analysis.py',
      count: 1,
      evidenceCount: 0,
      actionCount: 0,
      lowConfidence: false,
    })
  })
})
