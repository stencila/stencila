import type {
  GraphEdgeKind,
  GraphNode,
  GraphViewNodeKind,
  ResolvedGraphViewPreset,
} from './types'

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

export function edgeKindInPreset(
  kind: GraphEdgeKind,
  preset: ResolvedGraphViewPreset
): boolean {
  return preset === 'full' || EDGE_PRESETS_BY_KIND[kind].includes(preset)
}

export function edgeLabel(kind: GraphEdgeKind): string {
  return kind.replace(/([a-z])([A-Z])/g, '$1 $2')
}

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

function graphIdNamespace(id: string): string {
  const index = id.indexOf(':')
  return index === -1 ? id : id.slice(0, index)
}

function schemaNodeType(node: GraphNode): string | undefined {
  return stringValue(asRecord(node.node).type)
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object'
    ? (value as Record<string, unknown>)
    : {}
}

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

function compactLabel(label: string): string {
  const value = label.replace(/^([a-z]+):/, '').replace(/^.*[/#]/, '')
  return value.length > 42 ? `${value.slice(0, 39)}...` : value
}
