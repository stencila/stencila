/**
 * Native ResearchObject support for the Stencila Tiptap editor.
 *
 * This module is the single interface for the five editable research-object
 * wrappers: their schema nodes, presentation metadata, authoring commands, and
 * advisory MIRA relation semantics all live here.
 */
import type { ResearchObjectRelationKind } from '@stencila/types'
import { type Attributes, type Editor, Node } from '@tiptap/core'
import { NodeSelection } from '@tiptap/pm/state'

import { passthroughAttrs } from './attributes'

export const RESEARCH_OBJECT_NODE_NAMES = [
  'claim',
  'evidence',
  'question',
  'protocol',
  'request',
] as const

export type ResearchObjectNodeName =
  (typeof RESEARCH_OBJECT_NODE_NAMES)[number]

export interface ResearchObjectDefinition {
  name: ResearchObjectNodeName
  title: string
  icon: string
  shortcut: string
}

/** Definitions in the order used by authoring menus. */
export const RESEARCH_OBJECTS: readonly ResearchObjectDefinition[] = [
  {
    name: 'claim',
    title: 'Claim',
    icon: 'i-lucide:badge-check',
    shortcut: 'Mod-Alt-c',
  },
  {
    name: 'question',
    title: 'Question',
    icon: 'i-lucide:circle-help',
    shortcut: 'Mod-Alt-q',
  },
  {
    name: 'request',
    title: 'Request',
    icon: 'i-lucide:hand',
    shortcut: 'Mod-Alt-r',
  },
  {
    name: 'evidence',
    title: 'Evidence',
    icon: 'i-lucide:scale',
    shortcut: 'Mod-Alt-e',
  },
  {
    name: 'protocol',
    title: 'Protocol',
    icon: 'i-lucide:clipboard-list',
    shortcut: 'Mod-Alt-p',
  },
]

const RESEARCH_OBJECT_DEFINITIONS = new Map(
  RESEARCH_OBJECTS.map((definition) => [definition.name, definition])
)

export function isResearchObjectNodeName(
  name: string
): name is ResearchObjectNodeName {
  return RESEARCH_OBJECT_DEFINITIONS.has(name as ResearchObjectNodeName)
}

export function researchObjectDefinition(
  name: ResearchObjectNodeName
): ResearchObjectDefinition {
  // Every ResearchObjectNodeName is populated by the exhaustive registry.
  return RESEARCH_OBJECT_DEFINITIONS.get(name) as ResearchObjectDefinition
}

function optionalString(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined
}

function createResearchObjectNode(
  definition: ResearchObjectDefinition,
  extraAttributes: Attributes = {}
) {
  const { name, title, icon, shortcut } = definition

  return Node.create({
    name,
    group: 'block',
    content: 'block*',
    defining: true,

    addAttributes() {
      return {
        ...passthroughAttrs(
          'id',
          'relations',
          'metadata',
          'label',
          'title',
          'claimType'
        ),
        ...extraAttributes,
      }
    },

    parseHTML() {
      return [{ tag: `stencila-${name}-wrapper` }]
    },

    renderHTML({ node }) {
      const label = optionalString(node.attrs.label)
      const nodeTitle = optionalString(node.attrs.title)
      const claimType =
        name === 'claim' ? optionalString(node.attrs.claimType) : undefined
      const relationCount = Array.isArray(node.attrs.relations)
        ? node.attrs.relations.length
        : 0

      const header: unknown[] = [
        'div',
        {
          class: 'stencila-tiptap-research-object-header',
          contenteditable: 'false',
        },
        ['span', { class: `stencila-tiptap-research-object-icon ${icon}` }],
        ['span', { class: 'stencila-tiptap-research-object-type' }, title],
      ]

      if (label) {
        header.push([
          'span',
          { class: 'stencila-tiptap-research-object-chip' },
          label,
        ])
      }
      if (claimType) {
        header.push([
          'span',
          { class: 'stencila-tiptap-research-object-chip' },
          claimType,
        ])
      }
      if (relationCount > 0) {
        header.push([
          'span',
          { class: 'stencila-tiptap-research-object-relations' },
          `${relationCount} ${relationCount === 1 ? 'relation' : 'relations'}`,
        ])
      }

      const children: unknown[] = [header]
      if (nodeTitle) {
        children.push([
          'div',
          {
            class: 'stencila-tiptap-research-object-title',
            contenteditable: 'false',
          },
          nodeTitle,
        ])
      }
      children.push([
        'div',
        { class: 'stencila-tiptap-research-object-content' },
        0,
      ])

      return [
        `stencila-${name}-wrapper`,
        {
          class: `stencila-tiptap-research-object stencila-tiptap-research-object-${name}`,
          'data-research-object-type': name,
        },
        ...children,
      ]
    },

    addKeyboardShortcuts() {
      return {
        [shortcut]: () => this.editor.commands.toggleWrap(name),
      }
    },
  })
}

/** Native Tiptap extensions for all concrete ResearchObject block types. */
export const ResearchObjectExtensions = RESEARCH_OBJECTS.map((definition) =>
  createResearchObjectNode(definition)
)

export function wrapInResearchObject(
  editor: Editor,
  name: ResearchObjectNodeName
): boolean {
  const wrapped = editor.commands.wrapIn(name)
  if (wrapped) {
    editor.commands.focus()
  }
  return wrapped
}

export function canWrapInResearchObject(editor: Editor): boolean {
  return editor.can().wrapIn(RESEARCH_OBJECT_NODE_NAMES[0])
}

/** Return the innermost ResearchObject containing the current selection. */
export function activeResearchObject(
  editor: Editor
): ResearchObjectNodeName | null {
  const { selection } = editor.state

  if (
    selection instanceof NodeSelection &&
    isResearchObjectNodeName(selection.node.type.name)
  ) {
    return selection.node.type.name
  }

  const depth = selection.$from.sharedDepth(selection.to)
  for (let index = depth; index > 0; index -= 1) {
    const name = selection.$from.node(index).type.name
    if (isResearchObjectNodeName(name)) {
      return name
    }
  }

  return null
}

export function unwrapResearchObject(editor: Editor): boolean {
  const name = activeResearchObject(editor)
  const lifted = name !== null && editor.commands.lift(name)
  if (lifted) {
    editor.commands.focus()
  }
  return lifted
}

export const RESEARCH_OBJECT_RELATION_KINDS = [
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
] as const satisfies readonly ResearchObjectRelationKind[]

interface RelationRecommendation {
  sources: readonly ResearchObjectNodeName[]
  targets: readonly ResearchObjectNodeName[]
}

/**
 * MIRA-derived authoring guidance. These are deliberately recommendations:
 * the inspector offers every kind and target and only uses this table to sort.
 */
const RELATION_RECOMMENDATIONS: Record<
  ResearchObjectRelationKind,
  RelationRecommendation
> = {
  Supports: { sources: ['claim', 'evidence'], targets: ['claim'] },
  SupportedBy: { sources: ['claim'], targets: ['claim', 'evidence'] },
  Opposes: { sources: ['claim', 'evidence'], targets: ['claim'] },
  OpposedBy: { sources: ['claim'], targets: ['claim', 'evidence'] },
  Addresses: { sources: ['claim'], targets: ['question'] },
  AddressedBy: { sources: ['question'], targets: ['claim'] },
  Follows: { sources: [], targets: ['protocol'] },
  Grounds: { sources: [], targets: ['evidence'] },
  IsGroundedIn: { sources: ['evidence'], targets: [] },
  RequestFor: { sources: ['request'], targets: [] },
  RequestTarget: { sources: ['request'], targets: ['claim'] },
}

export function recommendedRelationKinds(
  source: ResearchObjectNodeName
): readonly ResearchObjectRelationKind[] {
  return RESEARCH_OBJECT_RELATION_KINDS.filter((kind) =>
    RELATION_RECOMMENDATIONS[kind].sources.includes(source)
  )
}

export function recommendedRelationTargets(
  kind: ResearchObjectRelationKind
): readonly ResearchObjectNodeName[] {
  return RELATION_RECOMMENDATIONS[kind].targets
}

export function relationKindLabel(kind: string): string {
  return kind
    .split(/(?=[A-Z])/)
    .map((word, index) => (index === 0 ? word : word.toLowerCase()))
    .join(' ')
}
