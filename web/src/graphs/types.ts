import type {
  Graph,
  GraphEdge,
  GraphEdgeKind,
  GraphEvidence,
  GraphNode,
} from '@stencila/types'

export type { Graph, GraphEdge, GraphEdgeKind, GraphEvidence, GraphNode }

export type GraphViewPreset =
  | 'auto'
  | 'full'
  | 'data-flow'
  | 'software-dependencies'
  | 'citations'
  | 'reactivity'

export type ResolvedGraphViewPreset = Exclude<GraphViewPreset, 'auto'>

export type GraphLayout = 'breadthfirst' | 'cose' | 'grid' | 'circle'

export interface GraphProjectionOptions {
  preset: GraphViewPreset
  includeStructureEdges?: boolean
  includeLowConfidenceEdges?: boolean
  collapseCitationNodes?: boolean
}

export interface ResolvedGraphProjectionOptions {
  preset: ResolvedGraphViewPreset
  includeStructureEdges: boolean
  includeLowConfidenceEdges: boolean
  collapseCitationNodes: boolean
}

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

export interface GraphViewNode {
  id: string
  label: string
  kind: GraphViewNodeKind
  node: GraphNode
}

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

export interface GraphView {
  preset: ResolvedGraphViewPreset
  nodes: GraphViewNode[]
  edges: GraphViewEdge[]
}
