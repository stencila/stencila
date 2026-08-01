import type {
  GraphAction,
  GraphEvidence,
  GraphEvidenceConfidence,
  GraphEvidenceKind,
} from '@stencila/types'

import type {
  GraphEdge,
  GraphViewEdge,
  GraphViewNode,
  GraphViewNodeKind,
} from './types'

export type EdgeRole =
  | 'input'
  | 'derivation'
  | 'output'
  | 'software'
  | 'structure'
  | 'discourse'

export type EvidenceMarker = 'none' | 'recorded' | 'attested'

export type BadgeTone = 'blue' | 'teal' | 'violet' | 'amber' | 'gray'

export type PresentedDetail = [string, unknown]

export interface PresentedEvidence {
  evidence: GraphEvidence
  graphEdge: GraphEdge
  confidence: GraphEvidenceConfidence
  location?: string
  source?: GraphEvidence['source']
  details: PresentedDetail[]
  contributor: number
}

export interface PresentedAction {
  action: GraphAction
  graphEdge: GraphEdge
  type: string
  details: PresentedDetail[]
  contributor: number
}

export interface ActionBadge {
  label: string
  tone: BadgeTone
}

export interface MetadataIdentity {
  key: string
  label: string
}

const EDGE_ROLES: Record<GraphViewEdge['kind'], EdgeRole> = {
  UsedBy: 'input',
  ReadBy: 'input',
  ReceivedBy: 'input',
  IncludedBy: 'input',
  LinkedBy: 'input',
  CitedBy: 'input',
  Generated: 'output',
  WrittenTo: 'output',
  SentTo: 'output',
  DerivedInto: 'derivation',
  ConvertedInto: 'derivation',
  CalledBy: 'software',
  ImportedBy: 'software',
  Declares: 'software',
  Configures: 'software',
  RequiredBy: 'software',
  Pins: 'software',
  PartOf: 'structure',
  Supports: 'discourse',
  SupportedBy: 'discourse',
  Opposes: 'discourse',
  OpposedBy: 'discourse',
  Addresses: 'discourse',
  AddressedBy: 'discourse',
  Follows: 'discourse',
  Grounds: 'discourse',
  IsGroundedIn: 'discourse',
  RequestFor: 'discourse',
  RequestTarget: 'discourse',
}

const ATTESTED_KINDS = new Set<GraphEvidenceKind>(['Attested'])
const MAX_EVIDENCE_GLYPHS = 5

export function edgeRole(kind: GraphViewEdge['kind']): EdgeRole {
  return EDGE_ROLES[kind]
}

export function relationshipBadgeTone(
  kind: GraphViewEdge['kind']
): BadgeTone {
  return {
    input: 'blue',
    derivation: 'teal',
    output: 'violet',
    software: 'amber',
    structure: 'gray',
    discourse: 'gray',
  }[edgeRole(kind)] as BadgeTone
}

export function nodeBadgeTone(kind: GraphViewNodeKind): BadgeTone {
  switch (kind) {
    case 'document':
    case 'resource':
      return 'blue'
    case 'code':
    case 'symbol':
    case 'function':
    case 'datatable':
      return 'teal'
    case 'environment':
    case 'package':
      return 'amber'
    case 'reference':
    case 'citation':
    case 'output':
      return 'violet'
    case 'workspace':
    case 'content':
    case 'other':
      return 'gray'
  }
}

export function evidenceBadgeTone(kind: GraphEvidenceKind): BadgeTone {
  switch (kind) {
    case 'Declared':
    case 'Resolved':
    case 'Imported':
      return 'blue'
    case 'Observed':
    case 'Computed':
    case 'Recorded':
    case 'StaticAnalysis':
    case 'RuntimeAnalysis':
      return 'teal'
    case 'UserAssertion':
    case 'Attested':
      return 'violet'
    case 'Inferred':
      return 'amber'
  }
}

export function confidenceBadgeTone(
  confidence: GraphEvidenceConfidence
): BadgeTone {
  return {
    Low: 'amber',
    Medium: 'blue',
    High: 'teal',
    Certain: 'violet',
  }[confidence] as BadgeTone
}

export function actionBadge(type: GraphAction['type']): ActionBadge {
  switch (type) {
    case 'ExecuteAction':
      return { label: 'Execute', tone: 'amber' }
    case 'CreateAction':
      return { label: 'Create', tone: 'violet' }
    case 'ConvertAction':
      return { label: 'Convert', tone: 'teal' }
    case 'Action':
      return { label: 'Action', tone: 'gray' }
  }
}

export function evidenceMarker(edge: GraphViewEdge): EvidenceMarker {
  if (edge.evidenceCount === 0) {
    return 'none'
  }

  return edge.edges.some((rawEdge) =>
    rawEdge.evidence?.some((evidence) => ATTESTED_KINDS.has(evidence.kind))
  )
    ? 'attested'
    : 'recorded'
}

/**
 * Format evidence as compact glyphs for an edge label.
 *
 * Ordinary evidence uses circles while attested evidence uses diamonds. Small
 * groups remain directly countable; larger groups switch to multiplication so
 * labels do not grow without bound.
 */
export function evidenceGlyphs(edge: GraphViewEdge): string {
  let recorded = 0
  let attested = 0

  for (const rawEdge of edge.edges) {
    for (const evidence of rawEdge.evidence ?? []) {
      if (ATTESTED_KINDS.has(evidence.kind)) {
        attested += 1
      } else {
        recorded += 1
      }
    }
  }

  return [formatGlyphCount('●', recorded), formatGlyphCount('◆', attested)]
    .filter(Boolean)
    .join(' ')
}

/** Format a relationship label with its evidence glyphs, when present. */
export function edgeDisplayLabel(edge: GraphViewEdge): string {
  const glyphs = evidenceGlyphs(edge)
  return glyphs ? `${edge.label} · ${glyphs}` : edge.label
}

export function presentEvidence(edge: GraphViewEdge): PresentedEvidence[] {
  return edge.edges.flatMap((graphEdge, index) =>
    (graphEdge.evidence ?? []).map((evidence) => ({
      evidence,
      graphEdge,
      confidence: evidence.confidence ?? 'Certain',
      location: formatCodeLocation(evidence.codeLocation),
      source: evidence.source,
      details: sortedDetails(evidence.details),
      contributor: index + 1,
    }))
  )
}

export function presentActions(edge: GraphViewEdge): PresentedAction[] {
  return edge.edges.flatMap((graphEdge, index) =>
    (graphEdge.actions ?? []).map((action) => ({
      action,
      graphEdge,
      type: action.type,
      details: sortedDetails(action),
      contributor: index + 1,
    }))
  )
}

export function evidenceLabel(item: PresentedEvidence): string {
  return (
    item.evidence.description ??
    item.location ??
    evidenceSourceLabel(item.source) ??
    humanizeLabel(item.evidence.kind)
  )
}

function evidenceSourceLabel(
  source: GraphEvidence['source']
): string | undefined {
  if (typeof source === 'string') {
    return source
  }

  return metadataIdentity(source)?.label ?? metadataTypeLabel(source)
}

export function actionLabel(item: PresentedAction): string {
  const name = item.details.find(([key]) => key === 'name')?.[1]
  return typeof name === 'string' && name
    ? name
    : actionBadge(item.action.type).label
}

export function nodeProperties(node: GraphViewNode): PresentedDetail[] {
  return sortedDetails(node.node.node).map(([key, value]) => [
    humanizeLabel(key),
    value,
  ])
}

export function humanizeLabel(value: string): string {
  const words = value
    .replace(/([a-z\d])([A-Z])/g, '$1 $2')
    .replace(/[_-]+/g, ' ')
    .trim()
    .toLowerCase()

  return words ? `${words[0].toUpperCase()}${words.slice(1)}` : value
}

export function metadataIdentity(value: unknown): MetadataIdentity | undefined {
  if (!isRecord(value)) {
    return undefined
  }

  for (const key of ['path', 'name', 'propertyId', 'id', 'kind', 'type']) {
    const label = value[key]
    if (typeof label === 'string' && label) {
      return { key, label }
    }
    if (typeof label === 'number') {
      return { key, label: String(label) }
    }
  }

  return undefined
}

export function metadataTypeLabel(value: unknown): string | undefined {
  if (!isRecord(value) || typeof value.type !== 'string' || !value.type) {
    return undefined
  }

  return humanizeLabel(value.type)
}

export function webUrlLabel(value: string): string | undefined {
  try {
    const url = new URL(value)
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
      return undefined
    }

    return `${url.host}${url.pathname === '/' ? '' : url.pathname}${url.search}${url.hash}`
  } catch {
    return undefined
  }
}

export function isLongMetadataValue(value: string): boolean {
  return value.length > 48 && !/\s/.test(value)
}

/**
 * Format the compact evidence and activity summary shown for an inspected edge.
 *
 * Most rendered edges represent exactly one graph relationship, so aggregation
 * is only mentioned when the projection has actually combined relationships.
 */
export function edgeSummary(edge: GraphViewEdge): string {
  const parts = [
    countLabel(edge.evidenceCount, 'evidence item'),
    countLabel(edge.actionCount, 'activity', 'activities'),
  ]

  if (edge.count > 1) {
    parts.push(countLabel(edge.count, 'contributing relationship'))
  }

  return parts.join(' · ')
}

export function formatCodeLocation(
  location: GraphEvidence['codeLocation']
): string | undefined {
  if (!location) {
    return undefined
  }

  const start =
    location.startLine === undefined
      ? undefined
      : `${location.startLine + 1}${
          location.startColumn === undefined ? '' : `:${location.startColumn + 1}`
        }`
  const end =
    location.endLine === undefined
      ? undefined
      : `${location.endLine + 1}${
          location.endColumn === undefined ? '' : `:${location.endColumn + 1}`
        }`
  const range = start ? `${start}${end && end !== start ? `–${end}` : ''}` : undefined

  if (location.source && range) {
    return `${location.source}:${range}`
  }

  return location.source ?? range
}

export function sortedDetails(
  details: unknown,
  omittedKeys: readonly string[] = ['type']
): PresentedDetail[] {
  if (!isRecord(details)) {
    return []
  }

  return Object.entries(details)
    .filter(([key, value]) => !omittedKeys.includes(key) && value !== undefined)
    .sort(([left], [right]) => left.localeCompare(right))
}

export function formatValue(value: unknown): string {
  if (typeof value === 'string') {
    return value
  }

  if (
    typeof value === 'number' ||
    typeof value === 'boolean' ||
    value === null
  ) {
    return String(value)
  }

  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

function formatGlyphCount(glyph: string, count: number): string {
  if (count === 0) {
    return ''
  }

  return count <= MAX_EVIDENCE_GLYPHS
    ? glyph.repeat(count)
    : `${glyph}×${count}`
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function countLabel(count: number, singular: string, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`
}
