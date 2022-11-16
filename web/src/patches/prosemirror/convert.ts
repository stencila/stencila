import { JsonValue } from '../checks'

/**
 * Convert a ProseMirror JSON representation of a node into Stencila JSON.
 *
 * This transformation is needed because the ProseMirror JSON is not quite the
 * same as Stencila JSON e.g. there are `marks` and `attrs` properties in
 * ProseMirror JSON.
 *
 * Note that derived, non-editable properties such as `errors` and `outputs` are not converted
 * to Stencila JSON.
 */
export function prosemirrorToStencila(value: JsonValue): JsonValue {
  // Primitive values can be just returned
  const type = typeof value
  if (
    type === 'string' ||
    type === 'number' ||
    type === 'boolean' ||
    value === null
  )
    return value

  // Reshape ProseMirror nodes as necessary
  const node = value as {
    type: string
    attrs: Record<string, JsonValue>
    content?: Array<JsonValue>
  }

  // Reshape or rename properties as necessary
  switch (node.type) {
    case 'Article':
      return {
        type: 'Article',
        // TODO: Handle title and description
        // @ts-expect-error Temporarily assuming that only content is `content`
        // (and not `title` or `description`)
        content: node.content[0].content.map(prosemirrorToStencila),
      }

    case 'Button':
      return {
        type: 'Button',
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
        name: node.attrs.name,
        label: node.attrs.label,
      }

    case 'Call':
      return {
        type: 'Call',
        id: node.attrs.id,
        source: node.attrs.source,
        select: node.attrs.select,
        arguments: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'CallArgument':
      return {
        type: 'CallArgument',
        id: node.attrs.id,
        name: node.attrs.name,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
        errors: node.attrs.errors,
      }

    case 'CodeBlock':
    case 'CodeFragment':
      return {
        type: node.type,
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        text: node.attrs.text,
      }

    case 'CodeChunk':
    case 'CodeExpression':
      return {
        type: node.type,
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
      }

    case 'Division':
    case 'Span':
      return {
        type: node.type,
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
      }

    case 'For':
      // Note: the `otherwise` property can not be edited in the prose editor
      // so is not included here
      return {
        type: 'For',
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
        symbol: node.attrs.symbol,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'Form':
      return {
        type: 'Form',
        id: node.attrs.id,
        deriveFrom: node.attrs.deriveFrom,
        deriveAction: node.attrs.deriveAction,
        deriveItem: node.attrs.deriveItem,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'Heading':
      return {
        type: 'Heading',
        depth: node.attrs.depth,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'If':
      return {
        type: 'If',
        id: node.attrs.id,
        clauses: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'IfClause':
      return {
        type: 'IfClause',
        id: node.attrs.id,
        programmingLanguage: node.attrs.programmingLanguage,
        guessLanguage: node.attrs.guessLanguage,
        text: node.attrs.text,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'Include':
      return {
        type: 'Include',
        id: node.attrs.id,
        source: node.attrs.source,
        select: node.attrs.select,
      }

    case 'List':
      return {
        type: 'List',
        order: node.attrs.order,
        items: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'ListItem':
      return {
        type: 'ListItem',
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'MathBlock':
    case 'MathFragment':
      return {
        type: node.type,
        id: node.attrs.id,
        mathLanguage: node.attrs.mathLanguage,
        text: node.attrs.text,
      }

    case 'Paragraph':
    case 'QuoteBlock':
      return {
        type: node.type,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'Parameter': {
      return {
        type: 'Parameter',
        id: node.attrs.id,
        name: node.attrs.name,
        label: node.attrs.label,
        derivedFrom: node.attrs.derivedFrom,
      }
    }

    case 'Table':
      return {
        type: 'Table',
        rows: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'TableRow':
      return {
        type: 'TableRow',
        cells: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'TableHeader':
    case 'TableCell':
      return {
        type: 'TableCell',
        cellType: node.type === 'TableHeader' ? 'Header' : null,
        colspan: node.attrs.colspan,
        rowspan: node.attrs.rowspan,
        content: node.content?.map(prosemirrorToStencila) ?? [],
      }

    case 'ThematicBreak':
      return {
        type: 'ThematicBreak',
      }

    case 'text': {
      // Transform ProseMirror text nodes into a (possibly nested) set of
      // inline nodes e.g. String, Strong, Emphasis.
      // Note that with this algorithm, the first applied mark will be the outer one.
      // This is related to the above merging of inline nodes.
      const text = value as {
        text: string
        marks?: [{ type: string }]
      }
      let node: string | { type: string; content: [JsonValue] } = text.text
      if (text.marks) {
        for (const mark of text.marks) {
          node = {
            type: mark.type,
            content: [node],
          }
        }
      }
      return node
    }

    default:
      return node
  }
}
