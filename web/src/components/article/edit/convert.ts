import { Node } from 'prosemirror-model'
import {
  isArray,
  isObject,
  JsonObject,
  JsonValue,
} from '../../../patches/checks'
import { articleMarks } from './schema'

/**
 * Convert a ProseMirror node to Stencila JSON.
 *
 * @why Used to generate Stencila patch `Operations` from more complicated
 * ProseMirror transactions which are difficult to transform into operations directly.
 */
export function prosemirrorToStencila(node: Node) {
  const json = node.toJSON()
  // console.log('Prosemirror: ', JSON.stringify(json, null, '  '))
  const stencila = transformProsemirror(json)
  // console.log('Stencila: ', JSON.stringify(stencila, null, '  '))
  return stencila
}

/**
 * Transform a ProseMirror JSON representation of a node into Stencila JSON.
 *
 * @why Because the ProseMirror JSON is not quite the same as Stencila JSON
 * e.g. `marks` and `attrs` properties in ProseMirror JSON
 *
 * @how Performance is important given that this function is recursively
 * called over potentially large documents. Given that, it favours mutation
 * and loops over restructuring and mapping etc.
 */
export function transformProsemirror(value: JsonValue): JsonValue {
  if (typeof value === 'string') return value
  if (typeof value === 'number') return value
  if (typeof value === 'boolean') return value
  if (value === null) return value

  if (Array.isArray(value)) {
    // Transform items of array and then merge adjacent inlines that mary have
    // arisen from how ProseMirror marks are handled (see below)
    let index = 0
    let prev: JsonValue | undefined
    while (index < value.length) {
      const curr = transformProsemirror(value[index] as JsonValue)
      if (
        isObject(prev) &&
        isArray(prev.content) &&
        isObject(curr) &&
        isArray(curr.content) &&
        prev.type == curr.type &&
        articleMarks.includes(curr.type as string)
      ) {
        value.splice(index, 1)
        prev.content.push(...curr.content)
      } else {
        value[index] = curr
        prev = curr
        index++
      }
    }
    return value
  }

  const object = value as JsonObject & {
    type: string
  }

  // Transform properties of objects
  for (const key in object) {
    object[key] = transformProsemirror(object[key] as JsonValue)
  }

  switch (object.type) {
    case 'Article':
      // Reshape the top-level article
      return {
        type: 'Article',
        // @ts-ignore
        title: object.title[0].content,
        // @ts-ignore
        abstract: object.abstract[0].content,
        // @ts-ignore
        content: object.content[0].content,
      }

    case 'Paragraph':
      // Ensure that the `content` property is defined
      // (won't be for empty paragraphs etc).
      // Important for diffing.
      object.content = object.content ?? []
      return object

    case 'Heading':
      // Ensure `content `and get properties
      object.content = object.content ?? []
      object.depth = (object.attrs as JsonObject).depth as number
      delete object.attrs
      return object

    case 'List':
      // Ensure `items` and get properties
      object.items = object.content ?? []
      delete object.content
      object.order = (object.attrs as JsonObject).order as string
      delete object.attrs
      return object

    case 'CodeBlock':
      // Ensure `text` and get properties
      object.text = (object.content as [string] | undefined)?.[0] ?? ''
      delete object.content
      object.programmingLanguage = (object.attrs as JsonObject)
        .programmingLanguage as string
      return object

    case 'QuoteBlock':
      // Ensure `content`
      object.content = object.content ?? []
      return object

    case 'Table':
      // Ensure `rows`
      object.rows = object.content ?? []
      delete object.content
      return object

    case 'TableRow':
      // Ensure `cells`
      object.cells = object.content ?? []
      delete object.content
      return object

    case 'TableHeader':
      // Convert to a `TableCell`
      // Note: intentionally falls through to `TableCell` case
      object.type = 'TableCell'
      object.cellType = 'Header'

    case 'TableCell':
      // Get properties
      const attrs = object.attrs as JsonObject
      object.colspan = attrs.colspan ?? 1
      object.rowspan = attrs.rowspan ?? 1
      delete object.attrs
      return object

    case 'text':
      // Transform ProseMirror text nodes into a (possibly nested) set of
      // inline nodes e.g. String, Strong, Emphasis.
      // Note that with this algorithm, the first applied mark will be the outer one.
      // This is related to the above merging of inline nodes.
      const text = object as JsonObject as {
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

    default:
      return object
  }
}
