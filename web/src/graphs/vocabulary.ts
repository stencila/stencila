/**
 * Graph display vocabulary.
 *
 * Schema graphs contain detailed node and edge names that are useful for data
 * exchange but too granular for a compact visual interface. This module maps
 * those schema concepts into the smaller vocabulary used by graph presets,
 * labels, and visual categories so projection and rendering stay consistent.
 */
import type {
  GraphEdgeKind,
  GraphNode,
  GraphViewNodeKind,
  ResolvedGraphViewPreset,
} from './types'

/**
 * The edge kind used for containment relationships.
 *
 * Structure edges are treated as contextual scaffolding by projections rather
 * than primary relationships. Naming the kind once avoids scattering special
 * cases across projection and vocabulary logic.
 */
export const STRUCTURE_EDGE_KIND: GraphEdgeKind = 'PartOf'

const EDGE_PRESETS_BY_KIND: Record<
  GraphEdgeKind,
  readonly ResolvedGraphViewPreset[]
> = {
  UsedBy: ['data-flow', 'software-dependencies'],
  ReadBy: ['data-flow'],
  Generated: ['data-flow'],
  CalledBy: ['software-dependencies'],
  DerivedInto: ['data-flow'],
  ConvertedInto: ['data-flow'],
  Parameterizes: ['reactivity'],
  DependsOn: ['reactivity'],
  PartOf: [],
  TranscludedBy: ['data-flow'],
  ImportedBy: ['software-dependencies'],
  CitedBy: ['citations'],
  ReferencedBy: ['citations'],
}

/**
 * Check whether an edge kind belongs to a view preset.
 *
 * Presets are intentionally user-facing categories rather than schema concepts.
 * Keeping the mapping here makes it clear which schema relationships are shown
 * for each browsing task and lets the projection code avoid duplicating that
 * policy.
 */
export function edgeKindInPreset(
  kind: GraphEdgeKind,
  preset: ResolvedGraphViewPreset
): boolean {
  return preset === 'full' || EDGE_PRESETS_BY_KIND[kind].includes(preset)
}

/**
 * Format an edge kind for display.
 *
 * Schema edge kinds are PascalCase identifiers. Splitting them only at word
 * boundaries preserves the canonical wording while making labels readable in
 * compact graph edges.
 */
export function edgeLabel(kind: GraphEdgeKind): string {
  return kind.replace(/([a-z])([A-Z])/g, '$1 $2')
}

/**
 * Classify a graph node for display.
 *
 * The renderer needs a small set of visual categories, not every schema node
 * type. This function prefers graph ID namespaces when available because they
 * encode graph-construction intent, then falls back to schema node types for
 * document-derived nodes that do not have specialized namespaces.
 */
export function nodeKind(node: GraphNode | undefined): GraphViewNodeKind {
  if (!node) {
    return 'other'
  }

  const namespace = graphIdNamespace(node.id)
  switch (namespace) {
    case 'dir':
      return 'workspace'
    case 'environment':
      return 'environment'
    case 'file':
    case 'symlink':
    case 'resource':
    case 'code-file':
      return 'resource'
    case 'code':
      return 'code'
    case 'symbol':
      return 'symbol'
    case 'function':
    case 'workflow-rule':
      return 'function'
    case 'package':
      return 'package'
    case 'column':
      return 'datatable'
    case 'reference':
      return 'reference'
    case 'output':
      return 'output'
  }

  const nodeType = schemaNodeType(node)

  if (nodeType === 'Citation' || node.id.includes('#citation')) {
    return 'citation'
  }

  if (nodeType === 'Reference') {
    return 'reference'
  }

  if (nodeType === 'CreativeWork') {
    return namespace === 'resource' ? 'resource' : 'reference'
  }

  if (
    nodeType === 'CodeBlock' ||
    nodeType === 'CodeChunk' ||
    nodeType === 'CodeExpression' ||
    nodeType === 'CodeInline' ||
    nodeType === 'SoftwareSourceCode'
  ) {
    return 'code'
  }

  if (nodeType === 'Variable') {
    return 'symbol'
  }

  if (nodeType === 'Function') {
    return 'function'
  }

  if (nodeType === 'Datatable' || nodeType === 'DatatableColumn') {
    return 'datatable'
  }

  if (nodeType === 'Directory') {
    return 'workspace'
  }

  if (nodeType === 'File' || nodeType === 'SymbolicLink') {
    return 'resource'
  }

  if (
    nodeType === 'Article' ||
    nodeType === 'Collection' ||
    nodeType === 'Prompt'
  ) {
    return 'document'
  }

  if (
    nodeType === 'AudioObject' ||
    nodeType === 'CitationGroup' ||
    nodeType === 'Datatable' ||
    nodeType === 'Figure' ||
    nodeType === 'Heading' ||
    nodeType === 'ImageObject' ||
    nodeType === 'IncludeBlock' ||
    nodeType === 'Link' ||
    nodeType === 'MediaObject' ||
    nodeType === 'Table' ||
    nodeType === 'VideoObject'
  ) {
    return 'content'
  }

  return 'other'
}

/**
 * Choose a compact display label for a graph node.
 *
 * Graph nodes may come from files, document nodes, references, symbols, or
 * generated resources, each with different naming fields. Trying common
 * human-readable fields first keeps labels useful, while falling back to the
 * graph ID guarantees every visible node has a stable label.
 */
export function nodeLabel(node: GraphNode): string {
  const schemaNode = asRecord(node.node)

  for (const key of [
    'name',
    'title',
    'path',
    'url',
    'target',
    'id',
  ]) {
    const value = schemaNode[key]
    const label = stringValue(value)
    if (label) {
      return compactLabel(label)
    }
  }

  return compactLabel(node.id)
}

/**
 * Extract the namespace from a graph ID.
 *
 * Graph builders use namespace prefixes to carry origin information such as
 * file, package, code, or reference. Reading the namespace lets classification
 * use that high-signal context without parsing the rest of the ID.
 */
function graphIdNamespace(id: string): string {
  const index = id.indexOf(':')
  return index === -1 ? id : id.slice(0, index)
}

/**
 * Read the schema node type from a graph node.
 *
 * The graph node payload is typed broadly after crossing the schema boundary.
 * Isolating this lookup keeps the rest of the vocabulary code defensive without
 * losing the useful schema type signal.
 */
function schemaNodeType(node: GraphNode): string | undefined {
  return stringValue(asRecord(node.node).type)
}

/**
 * Treat an unknown value as an object record when possible.
 *
 * Graph payload fields can be absent or have richer schema values. Returning an
 * empty record for non-objects lets callers probe optional fields without
 * repeated type guards.
 */
function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object'
    ? (value as Record<string, unknown>)
    : {}
}

/**
 * Extract text from common schema value shapes.
 *
 * Labels may be plain strings, arrays of inline-like values, or objects carrying
 * text under value, text, or content fields. Recursing through those shapes gives
 * the graph a readable label without depending on a specific node subtype.
 */
function stringValue(value: unknown): string | undefined {
  if (typeof value === 'string' && value.trim()) {
    return value
  }

  if (Array.isArray(value)) {
    const text = value
      .map((item) => stringValue(item))
      .filter((item): item is string => item !== undefined)
      .join(' ')
      .trim()

    return text || undefined
  }

  if (value && typeof value === 'object') {
    const record = value as Record<string, unknown>
    return stringValue(record.value ?? record.text ?? record.content)
  }

  return undefined
}

/**
 * Shorten a label for graph rendering.
 *
 * Cytoscape nodes have limited space, and graph IDs often include namespaces,
 * paths, or fragment prefixes. Compacting keeps the most distinguishing suffix
 * visible while preventing long labels from dominating the layout.
 */
function compactLabel(label: string): string {
  const value = label.replace(/^([a-z]+):/, '').replace(/^.*[/#]/, '')
  return value.length > 42 ? `${value.slice(0, 39)}...` : value
}
