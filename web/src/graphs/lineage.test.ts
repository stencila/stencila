import { describe, expect, it } from 'vitest'

import { dependencyNeighborhood, producedOutputIds } from './lineage'
import type { GraphView, GraphViewEdge } from './types'

function edge(
  id: string,
  source: string,
  target: string,
  kind: GraphViewEdge['kind'] = 'Generated'
): GraphViewEdge {
  const raw = { type: 'GraphEdge' as const, source, target, kind }
  return {
    id,
    source,
    target,
    kind,
    label: kind,
    edge: raw,
    edges: [raw],
    count: 1,
    evidenceCount: 0,
    actionCount: 0,
    lowConfidence: false,
  }
}

function view(edges: GraphViewEdge[]): GraphView {
  const ids = new Set(edges.flatMap((item) => [item.source, item.target]))
  return {
    preset: 'data-flow',
    detail: 'medium',
    nodes: [...ids].map((id) => ({
      id,
      label: id,
      kind: id === 'explicit' ? 'output' : 'resource',
      node: { type: 'GraphNode', id, node: { type: 'Thing' } },
    })),
    edges,
  } as GraphView
}

describe('dependencyNeighborhood', () => {
  it('traces upstream dependencies and downstream dependants', () => {
    const graph = view([
      edge('ab', 'a', 'b', 'ReadBy'),
      edge('bc', 'b', 'c'),
      edge('ce', 'c', 'e'),
      edge('ef', 'e', 'f'),
      edge('dc', 'd', 'c', 'PartOf'),
      edge('xy', 'x', 'y'),
    ])

    const neighborhood = dependencyNeighborhood(graph, 'c')
    expect([...neighborhood.upstreamNodeIds].sort()).toEqual(['a', 'b'])
    expect([...neighborhood.upstreamEdgeIds].sort()).toEqual(['ab', 'bc'])
    expect([...neighborhood.downstreamNodeIds].sort()).toEqual(['e', 'f'])
    expect([...neighborhood.downstreamEdgeIds].sort()).toEqual(['ce', 'ef'])
  })

  it('is cycle safe and identifies bidirectional overlap', () => {
    const neighborhood = dependencyNeighborhood(
      view([edge('ab', 'a', 'b'), edge('ba', 'b', 'a')]),
      'a'
    )
    expect([...neighborhood.upstreamNodeIds]).toEqual(['b'])
    expect([...neighborhood.downstreamNodeIds]).toEqual(['b'])
    expect([...neighborhood.overlapNodeIds]).toEqual(['b'])
    expect([...neighborhood.overlapEdgeIds].sort()).toEqual(['ab', 'ba'])
  })
})

describe('producedOutputIds', () => {
  it('includes explicit outputs and terminal production targets', () => {
    const graph = view([
      edge('ab', 'a', 'b'),
      edge('bc', 'b', 'c', 'ConvertedInto'),
      edge('cd', 'c', 'd', 'ReadBy'),
      edge('de', 'd', 'explicit', 'PartOf'),
    ])

    expect([...producedOutputIds(graph)].sort()).toEqual(['c', 'explicit'])
  })
})
