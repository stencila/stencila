import type { GraphView } from './types'
import { STRUCTURE_EDGE_KIND } from './vocabulary'

const PRODUCTION_EDGE_KINDS = new Set([
  'Generated',
  'WrittenTo',
  'SentTo',
  'DerivedInto',
  'ConvertedInto',
])

export interface DependencyNeighborhood {
  upstreamNodeIds: Set<string>
  downstreamNodeIds: Set<string>
  overlapNodeIds: Set<string>
  upstreamEdgeIds: Set<string>
  downstreamEdgeIds: Set<string>
  overlapEdgeIds: Set<string>
}

/**
 * Trace visible upstream dependencies and downstream dependants.
 *
 * Containment is deliberately excluded so selecting a document child does not
 * pull an entire workspace into the dependency neighborhood. Both traversals
 * are cycle safe and remain limited to nodes and edges in the current view.
 */
export function dependencyNeighborhood(
  view: GraphView,
  selectedId: string
): DependencyNeighborhood {
  const incoming = new Map<string, GraphView['edges']>()
  const outgoing = new Map<string, GraphView['edges']>()

  for (const edge of view.edges) {
    if (edge.kind === STRUCTURE_EDGE_KIND) {
      continue
    }

    const edges = incoming.get(edge.target) ?? []
    edges.push(edge)
    incoming.set(edge.target, edges)

    const downstreamEdges = outgoing.get(edge.source) ?? []
    downstreamEdges.push(edge)
    outgoing.set(edge.source, downstreamEdges)
  }

  const upstream = traverse(selectedId, incoming, (edge) => edge.source)
  const downstream = traverse(selectedId, outgoing, (edge) => edge.target)
  upstream.nodeIds.delete(selectedId)
  downstream.nodeIds.delete(selectedId)

  return {
    upstreamNodeIds: upstream.nodeIds,
    downstreamNodeIds: downstream.nodeIds,
    overlapNodeIds: intersection(upstream.nodeIds, downstream.nodeIds),
    upstreamEdgeIds: upstream.edgeIds,
    downstreamEdgeIds: downstream.edgeIds,
    overlapEdgeIds: intersection(upstream.edgeIds, downstream.edgeIds),
  }
}

function traverse(
  selectedId: string,
  adjacency: Map<string, GraphView['edges']>,
  adjacentNode: (edge: GraphView['edges'][number]) => string
) {
  const nodeIds = new Set<string>([selectedId])
  const edgeIds = new Set<string>()
  const pending = [selectedId]

  while (pending.length > 0) {
    const current = pending.pop()
    if (!current) {
      continue
    }

    for (const edge of adjacency.get(current) ?? []) {
      edgeIds.add(edge.id)
      const nodeId = adjacentNode(edge)
      if (!nodeIds.has(nodeId)) {
        nodeIds.add(nodeId)
        pending.push(nodeId)
      }
    }
  }

  return { nodeIds, edgeIds }
}

function intersection(left: Set<string>, right: Set<string>): Set<string> {
  return new Set([...left].filter((value) => right.has(value)))
}

/**
 * Classify explicit outputs and terminal targets of visible production edges.
 */
export function producedOutputIds(view: GraphView): Set<string> {
  const outputs = new Set(
    view.nodes.filter((node) => node.kind === 'output').map((node) => node.id)
  )
  const productionSources = new Set(
    view.edges
      .filter((edge) => PRODUCTION_EDGE_KINDS.has(edge.kind))
      .map((edge) => edge.source)
  )

  for (const edge of view.edges) {
    if (
      PRODUCTION_EDGE_KINDS.has(edge.kind) &&
      !productionSources.has(edge.target)
    ) {
      outputs.add(edge.target)
    }
  }

  return outputs
}
