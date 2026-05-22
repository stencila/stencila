/**
 * Graph projection utilities.
 *
 * Stencila graphs can include many node and edge kinds, including structural
 * containment edges that are useful in some views and noisy in others. This
 * module turns the complete schema graph into a smaller view model that matches
 * a reader's intent, so rendering code can stay focused on drawing the graph
 * instead of deciding which relationships matter.
 */
import type {
  Graph,
  GraphEdge,
  GraphNode,
  GraphProjectionOptions,
  GraphView,
  GraphViewEdge,
  GraphViewNode,
  GraphViewPreset,
  ResolvedGraphProjectionOptions,
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

/**
 * Return default projection options for a preset.
 *
 * Defaults encode the expected browsing experience for each preset, allowing
 * callers to override only the controls that are visible in the UI. The auto
 * preset intentionally leaves structural edge inclusion unresolved because the
 * final preset is chosen from the graph's actual relationships.
 */
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

/**
 * Resolve projection options after the preset is known.
 *
 * Auto projection can only choose structure defaults once it has selected a
 * concrete preset. This normalization step gives the rest of the projection
 * pipeline simple booleans, avoiding repeated fallback logic in edge filters and
 * structural ancestor expansion.
 */
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

/**
 * Project a schema graph into the browser graph view model.
 *
 * The raw graph is optimized as a complete interchange format, not as a direct
 * visualization model. Projection filters relationships by preset, optionally
 * folds citations into their document context, aggregates duplicate edges, and
 * sorts output so rendering is deterministic and easier to test.
 */
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

/**
 * Choose the best concrete preset for an auto projection.
 *
 * Auto mode should open on the first meaningful relationship family found in a
 * graph instead of always showing the full structure. The preset order reflects
 * the most common authoring questions first, then falls back to full when the
 * graph has only structural or otherwise unclassified relationships.
 */
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

/**
 * Decide whether an edge belongs to a preset's primary relationships.
 *
 * Structural edges are handled separately because they are context rather than
 * the main subject of most projections. Low-confidence filtering happens before
 * preset matching so all relationship families honor the same UI control.
 */
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

/**
 * Add structural context to a projected graph.
 *
 * Full graphs should include every containment edge, but focused projections
 * only need ancestors of already-visible nodes. This preserves enough document
 * context to understand where data flow, citations, or dependencies occur
 * without overwhelming the view with unrelated document structure.
 */
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

/**
 * Add or aggregate an edge in the view model.
 *
 * Multiple raw graph edges can represent the same visible relationship. Keeping
 * them under one rendered edge reduces clutter while preserving counts,
 * evidence totals, action totals, and confidence summaries for labels and future
 * inspection UI.
 */
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

/**
 * Build a stable visible-edge key.
 *
 * The key must survive graph sorting and include encoded endpoint IDs because
 * Stencila graph IDs commonly contain colons, hashes, and paths. Stable keys
 * make aggregation deterministic and keep Cytoscape element IDs predictable.
 */
function edgeKey(source: string, target: string, kind: string): string {
  return `edge:${kind}:${encodeURIComponent(source)}:${encodeURIComponent(target)}`
}

/**
 * Refresh aggregate edge summary fields.
 *
 * Summary fields are duplicated on the view edge because Cytoscape element data
 * is intentionally flat. Updating them whenever a raw edge is merged keeps the
 * rendered edge accurate without forcing render code to inspect nested arrays.
 */
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

/**
 * Build an internal key for structural edges.
 *
 * Structure lookup is always keyed by exact source and target, so a separator
 * that cannot appear through normal string joining avoids ambiguity while
 * keeping the map cheaper than repeated graph edge scans.
 */
function structureEdgeKey(source: string, target: string): string {
  return `${source}\u0000${target}`
}

/**
 * Check whether an edge carries low-confidence evidence.
 *
 * Confidence is attached to evidence records rather than directly to the edge.
 * Centralizing the check ensures projection filters and aggregate summaries use
 * the same interpretation of low-confidence relationships.
 */
function hasLowConfidence(edge: GraphEdge): boolean {
  return edge.evidence?.some((evidence) => evidence.confidence === 'Low') ?? false
}

/**
 * Build a child-to-parent map from structural edges.
 *
 * Several projection steps need to climb document containment. A map makes that
 * traversal direct and keeps citation collapsing and structural ancestor
 * expansion independent of the raw edge ordering.
 */
function parentMap(graph: Graph): Map<string, string> {
  const parents = new Map<string, string>()
  graph.edges
    .filter((edge) => edge.kind === STRUCTURE_EDGE_KIND)
    .forEach((edge) => parents.set(edge.source, edge.target))
  return parents
}

/**
 * Collapse a citation node to its nearest non-citation parent.
 *
 * Citation projections are usually read as "this reference is cited by this
 * document region" rather than by an intermediate citation marker node. Climbing
 * through citation nodes keeps the graph meaningful while preserving the raw
 * edge under the aggregated view edge.
 */
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

/**
 * Convert a schema graph node to a view node.
 *
 * Rendering needs a compact label and coarse visual category, while downstream
 * interactions may still need the original graph node. Keeping both on the view
 * node avoids recomputing vocabulary decisions in the renderer.
 */
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
