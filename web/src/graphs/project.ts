import type {
  Graph,
  GraphEdge,
  GraphNode,
  GraphProjectionOptions,
  ResolvedGraphProjectionOptions,
  GraphView,
  GraphViewEdge,
  GraphViewNode,
  GraphViewPreset,
  ResolvedGraphViewPreset,
} from './types'

import {
  STRUCTURE_EDGE_KIND,
  edgeKindInPreset,
  edgeLabel,
  nodeKind,
  nodeLabel,
} from './vocabulary'

const AUTO_PRESETS: ResolvedGraphViewPreset[] = [
  'data-flow',
  'software-dependencies',
  'citations',
  'reactivity',
]

interface ProjectedEdge {
  edge: GraphEdge
  source: string
  target: string
}

export function defaultProjectionOptions(
  preset: GraphViewPreset = 'auto'
): GraphProjectionOptions {
  const options: GraphProjectionOptions = {
    preset,
    includeLowConfidenceEdges: true,
    collapseCitationNodes: true,
  }

  if (preset !== 'auto') {
    options.includeStructureEdges = preset === 'full'
  }

  return options
}

function resolveProjectionOptions(
  preset: ResolvedGraphViewPreset,
  options: GraphProjectionOptions
): ResolvedGraphProjectionOptions {
  return {
    ...defaultProjectionOptions(preset),
    ...options,
    preset,
    includeStructureEdges:
      options.includeStructureEdges ??
      defaultProjectionOptions(preset).includeStructureEdges ??
      false,
    includeLowConfidenceEdges: options.includeLowConfidenceEdges ?? true,
    collapseCitationNodes: options.collapseCitationNodes ?? true,
  }
}

export function projectGraph(
  graph: Graph,
  options: GraphProjectionOptions
): GraphView {
  const preset = resolvePreset(graph, options)
  const resolvedOptions = resolveProjectionOptions(preset, options)
  const nodesById = new Map(graph.nodes.map((node) => [node.id, node]))
  const parentById = parentMap(graph)
  const nodeIds = new Set<string>()
  const edges = new Map<string, GraphViewEdge>()

  graph.edges.forEach((edge) => {
    if (!includePrimaryEdge(edge, preset, resolvedOptions)) {
      return
    }

    const source = edge.source
    let target = edge.target

    if (
      preset === 'citations' &&
      resolvedOptions.collapseCitationNodes &&
      edge.kind === 'CitedBy'
    ) {
      target = collapseCitationTarget(target, nodesById, parentById)
    }

    if (!nodesById.has(source) || !nodesById.has(target)) {
      return
    }

    nodeIds.add(source)
    nodeIds.add(target)

    addViewEdge(edges, { edge, source, target })
  })

  if (preset === 'full') {
    graph.nodes.forEach((node) => nodeIds.add(node.id))
  }

  if (resolvedOptions.includeStructureEdges) {
    addStructureEdges(
      graph,
      nodesById,
      parentById,
      nodeIds,
      edges,
      resolvedOptions,
      preset
    )
  }

  const nodes = Array.from(nodeIds)
    .sort()
    .map((id) => viewNode(nodesById.get(id)))
    .filter((node): node is GraphViewNode => node !== undefined)

  return {
    preset,
    nodes,
    edges: Array.from(edges.values()).sort((left, right) =>
      left.id.localeCompare(right.id)
    ),
  }
}

function resolvePreset(
  graph: Graph,
  options: GraphProjectionOptions
): ResolvedGraphViewPreset {
  if (options.preset !== 'auto') {
    return options.preset
  }

  return (
    AUTO_PRESETS.find((preset) =>
      graph.edges.some((edge) =>
        includePrimaryEdge(edge, preset, {
          includeLowConfidenceEdges: options.includeLowConfidenceEdges,
        })
      )
    ) ?? 'full'
  )
}

function includePrimaryEdge(
  edge: GraphEdge,
  preset: ResolvedGraphViewPreset,
  options: Pick<GraphProjectionOptions, 'includeLowConfidenceEdges'>
): boolean {
  if (options.includeLowConfidenceEdges === false && hasLowConfidence(edge)) {
    return false
  }

  if (edge.kind === STRUCTURE_EDGE_KIND) {
    return false
  }

  return edgeKindInPreset(edge.kind, preset)
}

function addStructureEdges(
  graph: Graph,
  nodesById: Map<string, GraphNode>,
  parentById: Map<string, string>,
  nodeIds: Set<string>,
  edges: Map<string, GraphViewEdge>,
  options: ResolvedGraphProjectionOptions,
  preset: ResolvedGraphViewPreset
) {
  const structureEdges = new Map<string, ProjectedEdge>()

  graph.edges.forEach((edge) => {
    if (
      edge.kind !== STRUCTURE_EDGE_KIND ||
      (options.includeLowConfidenceEdges === false && hasLowConfidence(edge)) ||
      !nodesById.has(edge.source) ||
      !nodesById.has(edge.target)
    ) {
      return
    }

    structureEdges.set(structureEdgeKey(edge.source, edge.target), {
      edge,
      source: edge.source,
      target: edge.target,
    })
  })

  if (preset === 'full') {
    structureEdges.forEach((edge) => addViewEdge(edges, edge))
    return
  }

  const seeds = Array.from(nodeIds)
  for (const seed of seeds) {
    let child = seed
    const visited = new Set<string>()

    while (!visited.has(child)) {
      visited.add(child)

      const parent = parentById.get(child)
      if (!parent || !nodesById.has(parent)) {
        break
      }

      const edge = structureEdges.get(structureEdgeKey(child, parent))
      if (!edge) {
        break
      }

      nodeIds.add(parent)
      addViewEdge(edges, edge)
      child = parent
    }
  }
}

function addViewEdge(
  edges: Map<string, GraphViewEdge>,
  projected: ProjectedEdge
) {
  const key = edgeKey(projected.source, projected.target, projected.edge.kind)
  const existing = edges.get(key)

  if (existing) {
    if (!existing.edges.includes(projected.edge)) {
      existing.edges.push(projected.edge)
      updateEdgeSummary(existing)
    }
    return
  }

  edges.set(key, {
    id: key,
    source: projected.source,
    target: projected.target,
    label: edgeLabel(projected.edge.kind),
    kind: projected.edge.kind,
    edges: [projected.edge],
    edge: projected.edge,
    count: 1,
    evidenceCount: projected.edge.evidence?.length ?? 0,
    actionCount: projected.edge.actions?.length ?? 0,
    lowConfidence: hasLowConfidence(projected.edge),
  })
}

function edgeKey(source: string, target: string, kind: string): string {
  return `edge:${kind}:${encodeURIComponent(source)}:${encodeURIComponent(target)}`
}

function updateEdgeSummary(edge: GraphViewEdge) {
  edge.count = edge.edges.length
  edge.evidenceCount = edge.edges.reduce(
    (total, graphEdge) => total + (graphEdge.evidence?.length ?? 0),
    0
  )
  edge.actionCount = edge.edges.reduce(
    (total, graphEdge) => total + (graphEdge.actions?.length ?? 0),
    0
  )
  edge.lowConfidence = edge.edges.some(hasLowConfidence)
}

function structureEdgeKey(source: string, target: string): string {
  return `${source}\u0000${target}`
}

function hasLowConfidence(edge: GraphEdge): boolean {
  return edge.evidence?.some((evidence) => evidence.confidence === 'Low') ?? false
}

function parentMap(graph: Graph): Map<string, string> {
  const parents = new Map<string, string>()
  graph.edges
    .filter((edge) => edge.kind === STRUCTURE_EDGE_KIND)
    .forEach((edge) => parents.set(edge.source, edge.target))
  return parents
}

function collapseCitationTarget(
  target: string,
  nodesById: Map<string, GraphNode>,
  parentById: Map<string, string>
): string {
  let current = target
  const visited = new Set<string>()

  while (nodeKind(nodesById.get(current)) === 'citation') {
    if (visited.has(current)) {
      break
    }
    visited.add(current)

    const parent = parentById.get(current)
    if (!parent) {
      break
    }
    current = parent
  }

  return current
}

function viewNode(node: GraphNode | undefined): GraphViewNode | undefined {
  if (!node) {
    return undefined
  }

  return {
    id: node.id,
    label: nodeLabel(node),
    kind: nodeKind(node),
    node,
  }
}
