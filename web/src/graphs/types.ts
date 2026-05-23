/**
 * Graph view types.
 *
 * These types define the boundary between schema-level graph data and the
 * browser graph renderer. Keeping a narrow view model lets projection code
 * preserve the original Stencila graph objects while giving Cytoscape a stable,
 * display-oriented shape with resolved presets, labels, categories, and
 * aggregate edge summaries.
 */
import type {
  Graph,
  GraphEdge,
  GraphEdgeKind,
  GraphEvidence,
  GraphNode,
} from '@stencila/types'

export type { Graph, GraphEdge, GraphEdgeKind, GraphEvidence, GraphNode }

/**
 * User-selectable graph projection preset.
 *
 * Presets describe the question a reader is asking of the graph, while `auto`
 * lets the projection layer choose the first useful preset based on available
 * relationships. This keeps the UI small without hiding focused views.
 */
export type GraphViewPreset =
  | 'auto'
  | 'full'
  | 'data-flow'
  | 'software-dependencies'
  | 'citations'
  | 'reactivity'

/**
 * Projection preset after auto resolution.
 *
 * Rendered views must always know their concrete preset so labels, structure
 * defaults, and stats can describe what is actually being shown.
 */
export type ResolvedGraphViewPreset = Exclude<GraphViewPreset, 'auto'>

/**
 * Amount of detail to include in focused graph projections.
 */
export type GraphProjectionDetail = 'low' | 'medium' | 'high'

/**
 * User-selectable Cytoscape layout.
 *
 * The view exposes a constrained layout vocabulary so controls remain stable
 * even if the underlying Cytoscape layout options are tuned over time.
 */
export type GraphLayout = 'breadthfirst' | 'cose' | 'grid' | 'circle'

/**
 * Options used to project a schema graph.
 *
 * These are intentionally close to the UI controls. Optional booleans allow
 * callers to request defaults for a preset while still overriding specific
 * display choices when a user changes them.
 */
export interface GraphProjectionOptions {
  preset: GraphViewPreset
  detail?: GraphProjectionDetail
  includeStructureEdges?: boolean
  includeLowConfidenceEdges?: boolean
  collapseCitationNodes?: boolean
}

/**
 * Projection options after defaults have been resolved.
 *
 * Projection internals use this shape so filtering and aggregation logic can
 * rely on concrete booleans instead of repeating fallback checks.
 */
export interface ResolvedGraphProjectionOptions {
  preset: ResolvedGraphViewPreset
  detail: GraphProjectionDetail
  includeStructureEdges: boolean
  includeLowConfidenceEdges: boolean
  collapseCitationNodes: boolean
}

/**
 * Coarse node category used by the renderer.
 *
 * The schema has many more node types than a graph can usefully distinguish
 * visually. These categories group nodes by role so styling communicates the
 * important differences without creating a noisy legend.
 */
export type GraphViewNodeKind =
  | 'document'
  | 'workspace'
  | 'environment'
  | 'resource'
  | 'content'
  | 'code'
  | 'symbol'
  | 'function'
  | 'package'
  | 'datatable'
  | 'reference'
  | 'citation'
  | 'output'
  | 'other'

/**
 * Node shape consumed by the graph renderer.
 *
 * The label and kind are precomputed for display, while the original graph node
 * is retained so future interactions can inspect source data without having to
 * reverse-map from rendered elements.
 */
export interface GraphViewNode {
  id: string
  label: string
  kind: GraphViewNodeKind
  node: GraphNode
}

/**
 * Edge shape consumed by the graph renderer.
 *
 * A rendered edge may represent several raw graph edges between the same visible
 * nodes. Keeping both the aggregate summary and original edges allows the graph
 * to stay readable now while preserving details for labels and later drilldown.
 */
export interface GraphViewEdge {
  id: string
  source: string
  target: string
  label: string
  kind: GraphEdgeKind
  edges: GraphEdge[]
  edge: GraphEdge
  count: number
  evidenceCount: number
  actionCount: number
  lowConfidence: boolean
}

/**
 * Complete projected graph for rendering.
 *
 * This is the deterministic output of projection: a resolved preset plus sorted
 * node and edge arrays. The stable shape makes tests, rendering, and incremental
 * UI features easier to reason about.
 */
export interface GraphView {
  preset: ResolvedGraphViewPreset
  detail: GraphProjectionDetail
  nodes: GraphViewNode[]
  edges: GraphViewEdge[]
}
