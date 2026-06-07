/**
 * Pure model logic for editing properties of Tiptap nodes in the edit view.
 *
 * The toolbar and popover components are kept thin and declarative by delegating
 * all ProseMirror reads and writes to the framework-agnostic functions here.
 * This keeps the editing rules (which nodes are editable, how persistent ids are
 * validated, how patches map onto node attributes) testable without a DOM and
 * decoupled from Lit, and concentrates the awkward differences between native
 * Tiptap nodes (`codeBlock`, `table`) and opaque Stencila placeholders
 * (`stencilaBlock`) in one place.
 */
import type { Node as ProseMirrorNode } from '@tiptap/pm/model'
import { type EditorState, NodeSelection, type Transaction } from '@tiptap/pm/state'

/**
 * Node types whose properties can be edited from the toolbar.
 *
 * Restricting editing to a known set keeps the toolbar from appearing on prose
 * nodes (paragraphs, headings) that have no Stencila-level properties to manage.
 */
export const EDIT_NODE_PROPERTY_NODE_TYPES = [
  'codeBlock',
  'table',
  'stencilaBlock',
] as const

export type EditNodePropertyNodeType =
  (typeof EDIT_NODE_PROPERTY_NODE_TYPES)[number]

const PROPERTY_NODE_TYPES = new Set<string>(EDIT_NODE_PROPERTY_NODE_TYPES)

/**
 * The editable node currently targeted by the toolbar, with its read-only
 * property snapshot used to seed the popover form and render the summary bar.
 */
export interface EditNodePropertyTarget {
  pos: number
  typeName: EditNodePropertyNodeType
  displayName: string
  typeIcon: string
  summaryLabel: string
  persistentId?: string
  programmingLanguage?: string
  isDemo?: boolean
}

/**
 * A set of property changes to apply to a target node.
 *
 * Each field is tri-state: `undefined` leaves the property untouched, `null`
 * clears it, and a value sets it. This lets a single patch both set and remove
 * properties in one transaction.
 */
export interface EditNodePropertyPatch {
  persistentId?: string | null
  programmingLanguage?: string | null
  isDemo?: boolean | null
}

/**
 * Result of validating user input for a persistent id.
 */
export type PersistentIdValidation =
  | {
      ok: true
      value: string
    }
  | {
      ok: false
      message: string
    }

/**
 * Stable identity for a target, used to detect when the toolbar has moved to a
 * different node.
 *
 * Position alone is ambiguous (a position can host different nodes after edits),
 * so the node type is included to avoid carrying popover state across a node
 * change.
 */
export function editNodePropertyTargetKey(target: {
  pos: number
  typeName: EditNodePropertyNodeType
}): string {
  return `${target.pos}:${target.typeName}`
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function optionalString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined
}

function optionalBoolean(value: unknown): boolean | undefined {
  return typeof value === 'boolean' ? value : undefined
}

/**
 * Read the serialized Stencila node carried by an opaque Stencila placeholder.
 *
 * Opaque nodes store the original Stencila node as a plain object under the
 * `node` attribute so unsupported nodes round-trip losslessly; properties are
 * read from and written to that payload rather than to Tiptap attributes.
 */
function opaqueNodePayload(node: ProseMirrorNode): Record<string, unknown> | undefined {
  const payload = node.attrs.node
  return isRecord(payload) ? payload : undefined
}

function isOpaqueStencilaNode(node: ProseMirrorNode): boolean {
  return node.type.name === 'stencilaBlock' || node.type.name === 'stencilaInline'
}

function opaqueNodeType(node: ProseMirrorNode): string | undefined {
  return optionalString(node.attrs.nodeType) ?? optionalString(opaqueNodePayload(node)?.type)
}

/**
 * Turn a camelCase node type into a spaced, capitalized display name, e.g.
 * `codeBlock` -> `Code Block`.
 */
function formatNodeName(value: string): string {
  return value
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/^./, (letter) => letter.toUpperCase())
}

export function isEditNodePropertyNodeType(
  typeName: string
): typeName is EditNodePropertyNodeType {
  return PROPERTY_NODE_TYPES.has(typeName)
}

function propertyNodeType(
  node: ProseMirrorNode
): EditNodePropertyNodeType | undefined {
  const typeName = node.type.name
  return isEditNodePropertyNodeType(typeName) ? typeName : undefined
}

/**
 * Read a node's persistent id from the location appropriate to its type.
 *
 * Opaque Stencila nodes keep the id inside their serialized payload; native
 * nodes keep it as a Tiptap attribute.
 */
function readPersistentId(node: ProseMirrorNode): string | undefined {
  if (isOpaqueStencilaNode(node)) {
    return optionalString(opaqueNodePayload(node)?.id)
  }

  return optionalString(node.attrs.id)
}

function readPropertyString(node: ProseMirrorNode, attr: string): string | undefined {
  if (node.type.name === 'stencilaBlock') {
    return optionalString(opaqueNodePayload(node)?.[attr])
  }

  return optionalString(node.attrs[attr])
}

function readPropertyBoolean(node: ProseMirrorNode, attr: string): boolean | undefined {
  if (node.type.name === 'stencilaBlock') {
    return optionalBoolean(opaqueNodePayload(node)?.[attr])
  }

  return optionalBoolean(node.attrs[attr])
}

/**
 * Read a node's programming language.
 *
 * `codeBlock` reuses Tiptap's built-in `language` attribute, whereas other
 * nodes use the Stencila `programmingLanguage` name; centralizing this mapping
 * keeps the read and write sides from drifting apart.
 */
function readProgrammingLanguage(node: ProseMirrorNode): string | undefined {
  return node.type.name === 'codeBlock'
    ? readPropertyString(node, 'language')
    : readPropertyString(node, 'programmingLanguage')
}

/**
 * Icons representing a node's type, shown in the compact inspector bar.
 *
 * Keyed by the same `displayType` used for `displayName` (the native Tiptap type
 * for `codeBlock`/`table`, or the opaque Stencila node type for `stencilaBlock`),
 * with a generic fallback so unmapped types still get an icon. Extend as more
 * Stencila block types gain dedicated icons.
 */
const NODE_TYPE_ICONS: Record<string, string> = {
  codeBlock: 'i-lucide:square-code',
  table: 'i-lucide:table',
  CodeChunk: 'i-lucide:square-terminal',
  MathBlock: 'i-lucide:sigma',
  Figure: 'i-lucide:image',
}

const DEFAULT_NODE_TYPE_ICON = 'i-lucide:box'

function nodeTypeIcon(displayType: string): string {
  return NODE_TYPE_ICONS[displayType] ?? DEFAULT_NODE_TYPE_ICON
}

/**
 * Proper-cased display names for common programming languages.
 *
 * The summary bar shows a code node's language as its primary label, so the
 * lowercase stored value (`python`) is mapped to a conventionally cased name
 * (`Python`); anything unmapped falls back to capitalizing the first letter.
 */
const LANGUAGE_DISPLAY_NAMES: Record<string, string> = {
  javascript: 'JavaScript',
  typescript: 'TypeScript',
  python: 'Python',
  r: 'R',
  sql: 'SQL',
  bash: 'Bash',
  shell: 'Shell',
  json: 'JSON',
  html: 'HTML',
  css: 'CSS',
}

function formatLanguageName(language: string): string {
  const key = language.toLowerCase()
  return (
    LANGUAGE_DISPLAY_NAMES[key] ??
    language.replace(/^./, (letter) => letter.toUpperCase())
  )
}

/**
 * Pick the single most useful descriptor to show beside the type icon in the
 * compact summary bar: the programming language for code nodes, the captioned
 * label for tables and figures (e.g. `Table 1`, combining the type name with the
 * stored numeric label), and the type name on its own as a final fallback.
 */
function summaryLabel(node: ProseMirrorNode, displayName: string): string {
  const language = readProgrammingLanguage(node)
  if (language) {
    return formatLanguageName(language)
  }

  const label = readPropertyString(node, 'label')
  return label ? `${displayName} ${label}` : displayName
}

function targetFromNode(
  node: ProseMirrorNode,
  pos: number
): EditNodePropertyTarget | undefined {
  const typeName = propertyNodeType(node)
  if (!typeName) {
    return undefined
  }

  const displayType =
    typeName === 'stencilaBlock' ? opaqueNodeType(node) : typeName
  const displayName = formatNodeName(displayType ?? typeName)

  return {
    pos,
    typeName,
    displayName,
    typeIcon: nodeTypeIcon(displayType ?? typeName),
    summaryLabel: summaryLabel(node, displayName),
    persistentId: readPersistentId(node),
    programmingLanguage: readProgrammingLanguage(node),
    isDemo: readPropertyBoolean(node, 'isDemo'),
  }
}

/**
 * Find the editable node that contains a document position.
 *
 * Used for pointer inspection: the position may be inside a text node or table
 * cell, so the nearest property-bearing ancestor is used.
 */
export function findEditNodePropertyTargetAtPosition(
  state: EditorState,
  pos: number
): EditNodePropertyTarget | undefined {
  const clampedPos = Math.max(0, Math.min(pos, state.doc.content.size))
  const directNode = state.doc.nodeAt(clampedPos)
  const directTarget = directNode
    ? targetFromNode(directNode, clampedPos)
    : undefined
  if (directTarget) {
    return directTarget
  }

  if (clampedPos > 0) {
    const previousNode = state.doc.nodeAt(clampedPos - 1)
    const previousTarget = previousNode
      ? targetFromNode(previousNode, clampedPos - 1)
      : undefined
    if (previousTarget) {
      return previousTarget
    }
  }

  const $pos = state.doc.resolve(clampedPos)
  for (let depth = $pos.depth; depth > 0; depth -= 1) {
    const target = targetFromNode($pos.node(depth), $pos.before(depth))
    if (target) {
      return target
    }
  }

  return undefined
}

/**
 * Find the editable node the toolbar should target for the current selection.
 *
 * A directly selected property node is preferred; otherwise the nearest
 * property-bearing ancestor is used so the toolbar still appears when, for
 * example, the cursor is inside a table cell rather than on the table itself.
 */
export function findEditNodePropertyTarget(
  state: EditorState
): EditNodePropertyTarget | undefined {
  const { selection } = state

  if (selection instanceof NodeSelection) {
    const selectedTarget = targetFromNode(selection.node, selection.from)
    if (selectedTarget) {
      return selectedTarget
    }
  }

  for (let depth = selection.$from.depth; depth > 0; depth -= 1) {
    const node = selection.$from.node(depth)
    const target = targetFromNode(node, selection.$from.before(depth))
    if (target) {
      return target
    }
  }

  return undefined
}

/**
 * Strip a leading `#` and surrounding whitespace from a persistent id.
 *
 * Ids are displayed and often pasted as `#id`, but stored without the hash, so
 * input is normalized before validation and storage.
 */
export function normalizePersistentIdInput(value: string): string {
  const trimmed = value.trim()
  return trimmed.startsWith('#') ? trimmed.slice(1).trim() : trimmed
}

/**
 * Validate persistent id input against the document.
 *
 * Persistent ids must be non-empty, space-free, and unique across the document
 * so they can act as stable cross-references; the node at `currentPos` is
 * excluded so re-saving an unchanged id is not treated as a duplicate.
 */
export function validatePersistentIdInput(
  value: string,
  state: EditorState,
  currentPos: number
): PersistentIdValidation {
  const normalized = normalizePersistentIdInput(value)

  if (!normalized) {
    return {
      ok: false,
      message: 'Persistent id is empty',
    }
  }

  if (/\s/.test(normalized)) {
    return {
      ok: false,
      message: 'Persistent id cannot contain spaces',
    }
  }

  let duplicate = false
  state.doc.descendants((node, pos) => {
    if (pos !== currentPos && readPersistentId(node) === normalized) {
      duplicate = true
      return false
    }

    return true
  })

  if (duplicate) {
    return {
      ok: false,
      message: 'Persistent id already exists',
    }
  }

  return {
    ok: true,
    value: normalized,
  }
}

/**
 * Compute the new attributes for a node after applying a property patch, or
 * `undefined` if the node is not editable.
 *
 * Native nodes are patched via their Tiptap attributes (mapping
 * `programmingLanguage` onto `codeBlock`'s `language`), while opaque blocks are
 * patched inside their serialized payload, preserving the original `type` so the
 * node still round-trips after editing.
 */
function attrsWithPropertyPatch(
  node: ProseMirrorNode,
  patch: EditNodePropertyPatch
): Record<string, unknown> | undefined {
  const typeName = propertyNodeType(node)
  if (!typeName) {
    return undefined
  }

  if (typeName !== 'stencilaBlock') {
    const attrs: Record<string, unknown> = {
      ...node.attrs,
    }

    if (patch.persistentId !== undefined) {
      attrs.id = patch.persistentId
    }

    if (typeName === 'codeBlock') {
      if (patch.programmingLanguage !== undefined) {
        attrs.language = patch.programmingLanguage
      }
      if (patch.isDemo !== undefined) {
        attrs.isDemo = patch.isDemo
      }
    }

    return attrs
  }

  const payload = {
    ...(opaqueNodePayload(node) ?? {}),
  }
  const nodeType = opaqueNodeType(node)

  if (!optionalString(payload.type) && nodeType) {
    payload.type = nodeType
  }

  if (patch.persistentId !== undefined) {
    if (patch.persistentId) {
      payload.id = patch.persistentId
    } else {
      delete payload.id
    }
  }

  return {
    ...node.attrs,
    node: payload,
  }
}

/**
 * Build a transaction that applies a property patch to the target node, or
 * `undefined` if there is nothing valid to change.
 */
export function setEditNodePropertiesTransaction(
  state: EditorState,
  target: EditNodePropertyTarget,
  patch: EditNodePropertyPatch
): Transaction | undefined {
  const node = state.doc.nodeAt(target.pos)
  if (!node) {
    return undefined
  }

  const attrs = attrsWithPropertyPatch(node, patch)
  if (!attrs) {
    return undefined
  }

  return state.tr.setNodeMarkup(target.pos, undefined, attrs, node.marks)
}
