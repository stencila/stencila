/**
 * Graph projection tests.
 *
 * These tests document the behavior expected from focused graph presets:
 * automatic preset selection, structural context, citation collapsing,
 * confidence filtering, vocabulary classification, and edge aggregation.
 * Keeping those examples close to projection code makes future preset changes
 * explicit instead of accidental.
 */
import { describe, expect, it } from 'vitest'

import { defaultProjectionOptions, projectGraph } from './project'
import type { Graph } from './types'

describe('projectGraph', () => {
  it('selects a data-flow projection automatically', () => {
    const view = projectGraph(graph(), defaultProjectionOptions())

    expect(view.preset).toBe('data-flow')
    expect(view.edges.map((edge) => edge.kind)).toEqual(['ReadBy'])
    expect(view.nodes.map((node) => node.id)).toEqual([
      'code:analysis.py',
      'file:data.csv',
    ])
  })

  it('collapses citations to their document parent', () => {
    const view = projectGraph(graph(), {
      ...defaultProjectionOptions('citations'),
      collapseCitationNodes: true,
    })

    expect(view.edges).toHaveLength(1)
    expect(view.edges[0].source).toBe('reference:paper')
    expect(view.edges[0].target).toBe('node:document#article')
    expect(view.edges[0].edges).toHaveLength(1)
    expect(view.nodes.map((node) => node.kind)).toEqual(['document', 'reference'])
  })

  it('can filter low-confidence edges', () => {
    const view = projectGraph(graph(), {
      ...defaultProjectionOptions('data-flow'),
      includeLowConfidenceEdges: false,
    })

    expect(view.edges).toHaveLength(1)
    expect(view.edges[0].kind).toBe('ReadBy')
  })

  it('adds structural ancestors for projected nodes only', () => {
    const view = projectGraph(graph(), {
      ...defaultProjectionOptions('data-flow'),
      includeStructureEdges: true,
    })

    expect(view.nodes.map((node) => node.id)).toEqual([
      'code:analysis.py',
      'file:data.csv',
      'node:document#article',
    ])
    expect(
      view.edges.map((edge) => `${edge.kind}:${edge.source}->${edge.target}`)
    ).toEqual([
      'PartOf:code:analysis.py->node:document#article',
      'ReadBy:file:data.csv->code:analysis.py',
    ])
  })

  it('uses full preset structure defaults after auto resolution', () => {
    const view = projectGraph(structureOnlyGraph(), defaultProjectionOptions())

    expect(view.preset).toBe('full')
    expect(view.nodes.map((node) => node.id)).toEqual([
      'node:document#article',
      'node:document#figure',
    ])
    expect(view.edges.map((edge) => edge.kind)).toEqual(['PartOf'])
  })

  it('classifies graph vocabulary node namespaces', () => {
    const view = projectGraph(vocabularyGraph(), defaultProjectionOptions('full'))

    expect(
      Object.fromEntries(view.nodes.map((node) => [node.id, node.kind]))
    ).toMatchObject({
      'code:analysis.py': 'code',
      'column:analysis.py:data.csv:species': 'datatable',
      'environment:python:pyproject.toml': 'environment',
      'function:analysis.py:python:plot': 'function',
      'output:document#chunk:0': 'output',
      'package:python/numpy': 'package',
      'symbol:analysis.py:python:data': 'symbol',
      'workflow-rule:Snakefile:align': 'function',
    })
  })

  it('uses stable aggregate edge ids and summaries', () => {
    const view = projectGraph(duplicateEdgeGraph(), defaultProjectionOptions('data-flow'))

    expect(view.edges).toHaveLength(1)
    expect(view.edges[0]).toMatchObject({
      id: 'edge:ReadBy:file%3Adata.csv:code%3Aanalysis.py',
      count: 2,
      evidenceCount: 1,
      lowConfidence: true,
    })
  })

  it('defaults flow detail to medium without local symbols', () => {
    const view = projectGraph(detailGraph(), defaultProjectionOptions('data-flow'))

    expect(view.nodes.some((node) => node.kind === 'symbol')).toBe(false)
    expect(
      view.nodes.some((node) => node.id === 'function:analysis.py:python:read_csv')
    ).toBe(false)
    expect(
      view.nodes.some((node) => node.id === 'column:analysis.py:data.csv:count')
    ).toBe(true)
    expect(
      view.edges.map((edge) => `${edge.kind}:${edge.source}->${edge.target}`)
    ).toEqual([
      'DerivedInto:column:analysis.py:data.csv:count->file:plot.png',
      'Generated:code:analysis.py->file:plot.png',
      'ReadBy:file:data.csv->code:analysis.py',
    ])
  })

  it('hides datatable columns at low flow detail', () => {
    const view = projectGraph(detailGraph(), {
      ...defaultProjectionOptions('data-flow'),
      detail: 'low',
    })

    expect(view.nodes.some((node) => node.kind === 'datatable')).toBe(false)
    expect(view.edges.map((edge) => edge.kind)).toEqual(['Generated', 'ReadBy'])
  })

  it('includes local symbols and functions at high flow detail', () => {
    const view = projectGraph(detailGraph(), {
      ...defaultProjectionOptions('data-flow'),
      detail: 'high',
    })

    expect(view.nodes.some((node) => node.id === 'symbol:analysis.py:python:df')).toBe(
      true
    )
    expect(
      view.nodes.some((node) => node.id === 'function:analysis.py:python:read_csv')
    ).toBe(true)
    expect(view.edges.some((edge) => edge.kind === 'CalledBy')).toBe(true)
  })
})

/**
 * Build a mixed graph fixture.
 *
 * The fixture combines data-flow, citation, low-confidence, and structural
 * relationships so projection tests can verify that each preset selects only
 * the relevant subset while sharing the same source graph.
 */
function graph(): Graph {
  return {
    type: 'Graph',
    subject: 'test:graph',
    nodes: [
      {
        type: 'GraphNode',
        id: 'file:data.csv',
        node: { type: 'File', name: 'data.csv', path: 'data.csv' },
      },
      {
        type: 'GraphNode',
        id: 'code:analysis.py',
        node: { type: 'SoftwareSourceCode', name: 'analysis.py' },
      },
      {
        type: 'GraphNode',
        id: 'reference:paper',
        node: { type: 'Reference', title: 'Paper' },
      },
      {
        type: 'GraphNode',
        id: 'node:document#citation-1',
        node: { type: 'Citation', target: 'paper' },
      },
      {
        type: 'GraphNode',
        id: 'node:document#article',
        node: { type: 'Article', title: 'Report' },
      },
    ],
    edges: [
      {
        type: 'GraphEdge',
        source: 'file:data.csv',
        target: 'code:analysis.py',
        kind: 'ReadBy',
      },
      {
        type: 'GraphEdge',
        source: 'code:analysis.py',
        target: 'file:plot.png',
        kind: 'Generated',
        evidence: [{ type: 'GraphEvidence', kind: 'Inferred', confidence: 'Low' }],
      },
      {
        type: 'GraphEdge',
        source: 'reference:paper',
        target: 'node:document#citation-1',
        kind: 'CitedBy',
      },
      {
        type: 'GraphEdge',
        source: 'node:document#citation-1',
        target: 'node:document#article',
        kind: 'PartOf',
      },
      {
        type: 'GraphEdge',
        source: 'code:analysis.py',
        target: 'node:document#article',
        kind: 'PartOf',
      },
    ],
  } as Graph
}

/**
 * Build a graph with only structural relationships.
 *
 * Auto projection should still produce a useful view when no focused
 * relationship family exists. This fixture ensures that fallback path remains
 * covered without mixing in primary edges.
 */
function structureOnlyGraph(): Graph {
  return {
    type: 'Graph',
    subject: 'test:structure',
    nodes: [
      {
        type: 'GraphNode',
        id: 'node:document#article',
        node: { type: 'Article', title: 'Report' },
      },
      {
        type: 'GraphNode',
        id: 'node:document#figure',
        node: { type: 'Figure', label: 'Figure 1' },
      },
    ],
    edges: [
      {
        type: 'GraphEdge',
        source: 'node:document#figure',
        target: 'node:document#article',
        kind: 'PartOf',
      },
    ],
  } as Graph
}

/**
 * Build a graph covering specialized node namespaces.
 *
 * Node classification uses graph ID namespaces before schema type fallbacks.
 * This fixture keeps those namespace mappings visible and protects the visual
 * vocabulary used by the renderer.
 */
function vocabularyGraph(): Graph {
  return {
    type: 'Graph',
    subject: 'test:vocabulary',
    nodes: [
      {
        type: 'GraphNode',
        id: 'code:analysis.py',
        node: { type: 'SoftwareSourceCode', name: 'analysis.py' },
      },
      {
        type: 'GraphNode',
        id: 'column:analysis.py:data.csv:species',
        node: { type: 'DatatableColumn', name: 'species' },
      },
      {
        type: 'GraphNode',
        id: 'environment:python:pyproject.toml',
        node: { type: 'SoftwareSourceCode', name: 'Python environment' },
      },
      {
        type: 'GraphNode',
        id: 'function:analysis.py:python:plot',
        node: { type: 'Function', name: 'plot' },
      },
      {
        type: 'GraphNode',
        id: 'output:document#chunk:0',
        node: { type: 'ImageObject', contentUrl: 'plot.png' },
      },
      {
        type: 'GraphNode',
        id: 'package:python/numpy',
        node: { type: 'SoftwareSourceCode', name: 'numpy' },
      },
      {
        type: 'GraphNode',
        id: 'symbol:analysis.py:python:data',
        node: { type: 'Variable', name: 'data' },
      },
      {
        type: 'GraphNode',
        id: 'workflow-rule:Snakefile:align',
        node: { type: 'Function', name: 'align' },
      },
    ],
    edges: [],
  } as Graph
}

/**
 * Build a graph with duplicate visible relationships.
 *
 * The projection layer aggregates duplicate rendered edges while preserving
 * counts and confidence summaries. This fixture makes that aggregation behavior
 * deterministic and easy to assert.
 */
function duplicateEdgeGraph(): Graph {
  return {
    type: 'Graph',
    subject: 'test:duplicate',
    nodes: [
      {
        type: 'GraphNode',
        id: 'file:data.csv',
        node: { type: 'File', name: 'data.csv', path: 'data.csv' },
      },
      {
        type: 'GraphNode',
        id: 'code:analysis.py',
        node: { type: 'SoftwareSourceCode', name: 'analysis.py' },
      },
    ],
    edges: [
      {
        type: 'GraphEdge',
        source: 'file:data.csv',
        target: 'code:analysis.py',
        kind: 'ReadBy',
      },
      {
        type: 'GraphEdge',
        source: 'file:data.csv',
        target: 'code:analysis.py',
        kind: 'ReadBy',
        evidence: [{ type: 'GraphEvidence', kind: 'Inferred', confidence: 'Low' }],
      },
    ],
  } as Graph
}

function detailGraph(): Graph {
  return {
    type: 'Graph',
    subject: 'test:detail',
    nodes: [
      {
        type: 'GraphNode',
        id: 'file:data.csv',
        node: { type: 'File', name: 'data.csv', path: 'data.csv' },
      },
      {
        type: 'GraphNode',
        id: 'file:plot.png',
        node: { type: 'File', name: 'plot.png', path: 'plot.png' },
      },
      {
        type: 'GraphNode',
        id: 'code:analysis.py',
        node: { type: 'SoftwareSourceCode', name: 'analysis.py' },
      },
      {
        type: 'GraphNode',
        id: 'symbol:analysis.py:python:df',
        node: { type: 'Variable', name: 'df' },
      },
      {
        type: 'GraphNode',
        id: 'function:analysis.py:python:read_csv',
        node: { type: 'Function', name: 'read_csv' },
      },
      {
        type: 'GraphNode',
        id: 'column:analysis.py:data.csv:count',
        node: { type: 'DatatableColumn', name: 'count' },
      },
    ],
    edges: [
      {
        type: 'GraphEdge',
        source: 'file:data.csv',
        target: 'code:analysis.py',
        kind: 'ReadBy',
      },
      {
        type: 'GraphEdge',
        source: 'code:analysis.py',
        target: 'file:plot.png',
        kind: 'Generated',
      },
      {
        type: 'GraphEdge',
        source: 'code:analysis.py',
        target: 'symbol:analysis.py:python:df',
        kind: 'Generated',
      },
      {
        type: 'GraphEdge',
        source: 'symbol:analysis.py:python:df',
        target: 'code:analysis.py',
        kind: 'UsedBy',
      },
      {
        type: 'GraphEdge',
        source: 'function:analysis.py:python:read_csv',
        target: 'code:analysis.py',
        kind: 'CalledBy',
      },
      {
        type: 'GraphEdge',
        source: 'column:analysis.py:data.csv:count',
        target: 'code:analysis.py',
        kind: 'UsedBy',
      },
      {
        type: 'GraphEdge',
        source: 'column:analysis.py:data.csv:count',
        target: 'file:plot.png',
        kind: 'DerivedInto',
      },
    ],
  } as Graph
}
