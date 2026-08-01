import { describe, expect, it } from 'vitest'

import {
  actionBadge,
  actionLabel,
  confidenceBadgeTone,
  edgeDisplayLabel,
  edgeRole,
  edgeSummary,
  evidenceBadgeTone,
  evidenceGlyphs,
  evidenceLabel,
  evidenceMarker,
  formatCodeLocation,
  humanizeLabel,
  isLongMetadataValue,
  metadataIdentity,
  metadataTypeLabel,
  nodeBadgeTone,
  nodeProperties,
  presentActions,
  presentEvidence,
  relationshipBadgeTone,
  sortedDetails,
  webUrlLabel,
} from './evidence'
import type { GraphViewEdge } from './types'

function viewEdge(overrides: Partial<GraphViewEdge> = {}): GraphViewEdge {
  const edge = {
    type: 'GraphEdge' as const,
    source: 'file:data.csv',
    target: 'code:analysis.py',
    kind: 'ReadBy' as const,
  }

  return {
    id: 'edge:ReadBy:data:analysis',
    source: edge.source,
    target: edge.target,
    label: 'Read By',
    kind: edge.kind,
    edge,
    edges: [edge],
    count: 1,
    evidenceCount: 0,
    actionCount: 0,
    lowConfidence: false,
    ...overrides,
  }
}

describe('evidence presentation', () => {
  it('defaults confidence, converts source locations to one-based, and sorts details', () => {
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'file:data.csv',
      target: 'code:analysis.py',
      kind: 'ReadBy' as const,
      evidence: [
        {
          type: 'GraphEvidence' as const,
          kind: 'StaticAnalysis' as const,
          codeLocation: {
            type: 'CodeLocation' as const,
            source: 'analysis.py',
            startLine: 2,
            startColumn: 4,
            endLine: 2,
            endColumn: 9,
          },
          details: { zeta: 2, alpha: 'parser' },
        },
      ],
    }
    const [presented] = presentEvidence(
      viewEdge({
        edge: rawEdge,
        edges: [rawEdge],
        evidenceCount: 1,
      })
    )

    expect(presented.confidence).toBe('Certain')
    expect(presented.location).toBe('analysis.py:3:5–3:10')
    expect(presented.details).toEqual([
      ['alpha', 'parser'],
      ['zeta', 2],
    ])
  })

  it('preserves structured evidence sources for metadata inspection', () => {
    const source = {
      type: 'Person' as const,
      name: 'Ada Lovelace',
    }
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'file:data.csv',
      target: 'code:analysis.py',
      kind: 'ReadBy' as const,
      evidence: [
        {
          type: 'GraphEvidence' as const,
          kind: 'Attested' as const,
          source,
        },
      ],
    }
    const [presented] = presentEvidence(
      viewEdge({
        edge: rawEdge,
        edges: [rawEdge],
        evidenceCount: 1,
      })
    )

    expect(presented.source).toBe(source)
    expect(evidenceLabel(presented)).toBe('Ada Lovelace')
  })

  it('keeps duplicate evidence and actions from aggregated edges', () => {
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'source',
      target: 'target',
      kind: 'Generated' as const,
      evidence: [
        { type: 'GraphEvidence' as const, kind: 'Recorded' as const },
      ],
      actions: [{ type: 'ExecuteAction' as const }],
    }
    const aggregate = viewEdge({
      kind: 'Generated',
      edge: rawEdge,
      edges: [rawEdge, rawEdge],
      count: 2,
      evidenceCount: 2,
      actionCount: 2,
    })

    expect(presentEvidence(aggregate)).toHaveLength(2)
    expect(presentActions(aggregate)).toHaveLength(2)
    expect(presentEvidence(aggregate).map((item) => item.contributor)).toEqual([
      1, 2,
    ])
    expect(presentActions(aggregate).map((item) => item.contributor)).toEqual([
      1, 2,
    ])
  })

  it('only mentions contributing relationships for aggregates', () => {
    expect(
      edgeSummary(viewEdge({ evidenceCount: 1, actionCount: 1 }))
    ).toBe('1 evidence item · 1 activity')
    expect(
      edgeSummary(
        viewEdge({
          count: 2,
          evidenceCount: 2,
          actionCount: 1,
        })
      )
    ).toBe(
      '2 evidence items · 1 activity · 2 contributing relationships'
    )
  })

  it('detects evidence markers and edge semantic roles', () => {
    const attested = {
      type: 'GraphEdge' as const,
      source: 'source',
      target: 'target',
      kind: 'Generated' as const,
      evidence: [
        { type: 'GraphEvidence' as const, kind: 'Attested' as const },
      ],
    }

    expect(
      evidenceMarker(
        viewEdge({
          edge: attested,
          edges: [attested],
          evidenceCount: 1,
        })
      )
    ).toBe('attested')
    expect(evidenceMarker(viewEdge())).toBe('none')
    expect(edgeRole('ReadBy')).toBe('input')
    expect(edgeRole('ConvertedInto')).toBe('derivation')
    expect(edgeRole('Generated')).toBe('output')
    expect(edgeRole('RequiredBy')).toBe('software')
    expect(edgeRole('PartOf')).toBe('structure')
    expect(edgeRole('Supports')).toBe('discourse')
  })

  it('formats ordinary and attested evidence as compact label glyphs', () => {
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'source',
      target: 'target',
      kind: 'Generated' as const,
      evidence: [
        { type: 'GraphEvidence' as const, kind: 'Recorded' as const },
        { type: 'GraphEvidence' as const, kind: 'StaticAnalysis' as const },
        { type: 'GraphEvidence' as const, kind: 'Attested' as const },
      ],
    }
    const edge = viewEdge({
      label: 'Generated',
      kind: 'Generated',
      edge: rawEdge,
      edges: [rawEdge],
      evidenceCount: 3,
    })

    expect(evidenceGlyphs(edge)).toBe('●● ◆')
    expect(edgeDisplayLabel(edge)).toBe('Generated · ●● ◆')
    expect(edgeDisplayLabel(viewEdge())).toBe('Read By')
  })

  it('compacts large evidence groups', () => {
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'source',
      target: 'target',
      kind: 'Generated' as const,
      evidence: Array.from({ length: 7 }, () => ({
        type: 'GraphEvidence' as const,
        kind: 'Recorded' as const,
      })),
    }

    expect(
      evidenceGlyphs(
        viewEdge({
          edge: rawEdge,
          edges: [rawEdge],
          evidenceCount: 7,
        })
      )
    ).toBe('●×7')
  })

  it('formats partial locations and stable machine details', () => {
    expect(
      formatCodeLocation({
        type: 'CodeLocation',
        source: 'workflow.yml',
        startLine: 0,
      })
    ).toBe('workflow.yml:1')
    expect(sortedDetails({ beta: true, type: 'Action', alpha: null })).toEqual([
      ['alpha', null],
      ['beta', true],
    ])
  })

  it('preserves structured metadata while retaining stable key order', () => {
    const environment = {
      architecture: 'x86_64',
      runtimes: [{ version: '2.15.0', name: 'stencila' }],
      lockfiles: [{ path: 'uv.lock', digest: 'sha256:1234' }],
    }

    expect(
      sortedDetails({ environment, aiDisclosure: null, description: 'Export' })
    ).toEqual([
      ['aiDisclosure', null],
      ['description', 'Export'],
      ['environment', environment],
    ])
  })

  it('derives compact identities, web links, and long-value treatment', () => {
    expect(
      metadataIdentity({ digest: 'sha256:1234', path: 'uv.lock' })
    ).toEqual({ key: 'path', label: 'uv.lock' })
    expect(
      metadataIdentity({ version: '2.15.0', name: 'stencila' })
    ).toEqual({ key: 'name', label: 'stencila' })
    expect(metadataIdentity({ value: true })).toBeUndefined()
    expect(metadataTypeLabel({ type: 'ArrayValidator' })).toBe(
      'Array validator'
    )
    expect(metadataTypeLabel({ name: 'stencila' })).toBeUndefined()
    expect(webUrlLabel('https://github.com/stencila/stencila')).toBe(
      'github.com/stencila/stencila'
    )
    expect(webUrlLabel('javascript:alert(1)')).toBeUndefined()
    expect(isLongMetadataValue(`sha256:${'a'.repeat(64)}`)).toBe(true)
    expect(isLongMetadataValue('A long sentence that should wrap normally')).toBe(
      false
    )
  })

  it('maps every evidence kind and confidence to its semantic tone', () => {
    expect(
      Object.fromEntries(
        [
          'Declared',
          'Resolved',
          'Observed',
          'Computed',
          'Recorded',
          'StaticAnalysis',
          'RuntimeAnalysis',
          'Imported',
          'UserAssertion',
          'Attested',
          'Inferred',
        ].map((kind) => [kind, evidenceBadgeTone(kind as never)])
      )
    ).toEqual({
      Declared: 'blue',
      Resolved: 'blue',
      Observed: 'teal',
      Computed: 'teal',
      Recorded: 'teal',
      StaticAnalysis: 'teal',
      RuntimeAnalysis: 'teal',
      Imported: 'blue',
      UserAssertion: 'violet',
      Attested: 'violet',
      Inferred: 'amber',
    })
    expect({
      Low: confidenceBadgeTone('Low'),
      Medium: confidenceBadgeTone('Medium'),
      High: confidenceBadgeTone('High'),
      Certain: confidenceBadgeTone('Certain'),
    }).toEqual({
      Low: 'amber',
      Medium: 'blue',
      High: 'teal',
      Certain: 'violet',
    })
  })

  it('maps every action and relationship role to graph semantic tones', () => {
    expect([
      actionBadge('ExecuteAction'),
      actionBadge('CreateAction'),
      actionBadge('ConvertAction'),
      actionBadge('Action'),
    ]).toEqual([
      { label: 'Execute', tone: 'amber' },
      { label: 'Create', tone: 'violet' },
      { label: 'Convert', tone: 'teal' },
      { label: 'Action', tone: 'gray' },
    ])

    for (const kind of [
      'UsedBy',
      'ReadBy',
      'ReceivedBy',
      'IncludedBy',
      'LinkedBy',
      'CitedBy',
    ] as const) {
      expect(relationshipBadgeTone(kind)).toBe('blue')
    }
    for (const kind of ['DerivedInto', 'ConvertedInto'] as const) {
      expect(relationshipBadgeTone(kind)).toBe('teal')
    }
    for (const kind of ['Generated', 'WrittenTo', 'SentTo'] as const) {
      expect(relationshipBadgeTone(kind)).toBe('violet')
    }
    for (const kind of [
      'CalledBy',
      'ImportedBy',
      'Declares',
      'Configures',
      'RequiredBy',
      'Pins',
    ] as const) {
      expect(relationshipBadgeTone(kind)).toBe('amber')
    }
    for (const kind of [
      'PartOf',
      'Supports',
      'SupportedBy',
      'Opposes',
      'OpposedBy',
      'Addresses',
      'AddressedBy',
      'Follows',
      'Grounds',
      'IsGroundedIn',
      'RequestFor',
      'RequestTarget',
    ] as const) {
      expect(relationshipBadgeTone(kind)).toBe('gray')
    }
  })

  it('maps every node category to the graph palette', () => {
    expect(
      Object.fromEntries(
        [
          'document',
          'workspace',
          'environment',
          'resource',
          'content',
          'code',
          'symbol',
          'function',
          'package',
          'datatable',
          'reference',
          'citation',
          'output',
          'other',
        ].map((kind) => [kind, nodeBadgeTone(kind as never)])
      )
    ).toEqual({
      document: 'blue',
      workspace: 'gray',
      environment: 'amber',
      resource: 'blue',
      content: 'gray',
      code: 'teal',
      symbol: 'teal',
      function: 'teal',
      package: 'amber',
      datatable: 'teal',
      reference: 'violet',
      citation: 'violet',
      output: 'violet',
      other: 'gray',
    })
  })

  it('humanizes labels and formats node properties without schema type', () => {
    expect(humanizeLabel('recordedAt')).toBe('Recorded at')
    expect(humanizeLabel('runtime_analysis-rule')).toBe(
      'Runtime analysis rule'
    )
    expect(
      nodeProperties({
        id: 'file:data.csv',
        label: 'data.csv',
        kind: 'resource',
        node: {
          type: 'GraphNode',
          id: 'file:data.csv',
          node: { type: 'File', path: 'data.csv', name: 'Data' },
        },
      })
    ).toEqual([
      ['Name', 'Data'],
      ['Path', 'data.csv'],
    ])
  })

  it('chooses useful collapsed labels for evidence and activities', () => {
    const rawEdge = {
      type: 'GraphEdge' as const,
      source: 'source',
      target: 'target',
      kind: 'ReadBy' as const,
      evidence: [
        {
          type: 'GraphEvidence' as const,
          kind: 'Observed' as const,
          description: 'Read during inspection',
        },
      ],
      actions: [{ type: 'Action' as const, name: 'Inspect input' }],
    }
    const edge = viewEdge({ edge: rawEdge, edges: [rawEdge] })

    expect(evidenceLabel(presentEvidence(edge)[0])).toBe(
      'Read during inspection'
    )
    expect(actionLabel(presentActions(edge)[0])).toBe('Inspect input')
  })
})
