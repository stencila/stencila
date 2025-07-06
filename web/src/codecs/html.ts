/**
 * Encode a Stencila node to DOM HTML
 *
 * This is a browser-based TypeScript implementation of the `codec-dom` codec
 * in `../../rust/codec-dom` which encodes Stencila nodes to rich, mostly lossless
 * HTML for use in the browser by the Web Components in this module.
 *
 * This implementation closely mirrors the Rust codec which uses two approaches:
 *
 * 1. `DomCodec` Derive Macro (`../../rust/codec-dom-derive/src/lib.rs`):
 *    - Automatically generates encoding logic based on `#[dom(...)]` attributes
 *    - Processes fields as attributes or slots based on metadata:
 *       - Uses `#[dom(skip)]` to exclude fields from DOM encoding
 *       - Uses `#[dom(elem = "section")]` to specify HTML wrapper elements
 *       - Uses `#[dom(with = "function")]` for custom encoding functions
 *
 * 2. Manual` DomCodec` Implementations (`../../rust/schema/src/implem/*.rs`):
 *    - Custom logic for complex node types (Figure, Table, Link, etc.)
 *    - Creates semantic HTML structure with proper elements
 *    - Handles both web component slots and static HTML fallbacks
 *
 * This TypeScript version uses:
 *
 * - `EncodeContext` class (mirrors `DomEncodeContext` in `../../rust/codec-dom-trait/src/lib.rs`)
 * - Schema-driven encoding (mimics derive macro behavior)
 * - Manual encoders for specific node types (matches Rust manual implementations)
 *
 * When the tests fail because the Rust implementation has changed:
 *
 * 1. Check `../../rust/codec-dom-derive/src/lib.rs` for derive macro updates
 * 2. Review `../../rust/schema/src/implem/*.rs` for new manual implementations
 * 3. Look for new `#[dom(...)]` attributes in schema definitions
 * 4. Update corresponding TypeScript schemas and encoders
 */

import type { Heading, Link, Node, NodeType, Text } from '@stencila/types'

/**
 * Context for DOM encoding that tracks HTML generation state
 *
 * Mirrors the `DomEncodeContext` struct in `../../rust/codec-dom-trait/src/lib.rs`
 * which manages HTML building, node tracking, and encoding state.
 */
class EncodeContext {
  public html: string = ''
  private nodeStack: string[] = []
  public ancestors: string[]

  constructor(ancestors: string[] = []) {
    this.ancestors = [...ancestors]
  }

  /**
   * Enter a new node, creating the web component element
   *
   * Mirrors `context.enter_node()` in Rust.
   * Generates the opening tag with depth, ancestors, and root attributes.
   */
  enterNode(
    nodeType: NodeType,
    nodeId?: string,
    extraAttrs: Record<string, unknown> = {}
  ): void {
    this.nodeStack.push(nodeType)
    const tagName = this.getTagName(nodeType)

    // Always add a default ID (will be normalized in tests)
    const id = nodeId || 'xxx'
    let attrs =
      this.formatAttribute('id', id) +
      this.formatAttribute('depth', this.ancestors.length) +
      this.formatAttribute('ancestors', this.ancestors.join('.'))
    if (this.ancestors.length === 0) {
      attrs += ' root'
    }

    // Add extra attributes
    for (const [name, value] of Object.entries(extraAttrs)) {
      attrs += this.formatAttribute(name, value)
    }

    this.html += `<${tagName} ${attrs}>`
  }

  /**
   * Exit the current node, closing the web component element
   *
   * Mirrors `context.exit_node()` in Rust.
   */
  exitNode(): void {
    const nodeType = this.nodeStack.pop()
    if (nodeType) {
      const tagName = this.getTagName(nodeType)
      this.html += `</${tagName}>`
    }
  }

  /**
   * Format an attribute name-value pair for HTML
   *
   * Helper method to consistently format attributes with kebab-case names
   * and escaped values. Uses smart quote style: no quotes for simple values,
   * single quotes for empty strings, double quotes for complex values.
   * Used by both enterNode and pushAttribute methods.
   */
  private formatAttribute(name: string, value: unknown): string {
    if (value === null || value === undefined) return ''
    const attrName = this.toKebabCase(name)
    const attrValue = this.escapeAttributeValue(value)
    return ` ${attrName}="${attrValue}"`
  }

  /**
   * Add an attribute to the current node
   *
   * Mirrors attribute handling in the Rust derive macro.
   * Converts camelCase to kebab-case and escapes HTML entities.
   */
  pushAttribute(name: string, value: unknown): void {
    if (value === null || value === undefined) return

    const attrString = this.formatAttribute(name, value)

    // Insert attribute before the closing >
    const lastTagStart = this.html.lastIndexOf('<')
    const lastTagEnd = this.html.lastIndexOf('>')
    if (lastTagStart > lastTagEnd) {
      this.html += attrString
    }
  }

  /**
   * Add a slot with wrapper element
   *
   * Mirrors `context.push_slot_fn()` used in Rust manual implementations.
   * Creates semantic HTML structure with proper slot attributes.
   */
  pushSlot(tagName: string | null, slotName: string, content: string): void {
    if (tagName === null) {
      this.html += content
    } else {
      const slot = this.toKebabCase(slotName)
      const slotAttr = this.formatAttribute('slot', slot)
      this.html += `<${tagName}${slotAttr}>${content}</${tagName}>`
    }
  }

  /**
   * Add plain text content, escaping HTML entities
   *
   * Mirrors `context.push_text()` in Rust implementations.
   */
  pushText(text: string): void {
    this.html += this.escapeHtml(text)
  }

  /**
   * Get the generated HTML
   */
  getHtml(): string {
    return this.html
  }

  /**
   * Generate web component tag name from node type
   *
   * Mirrors the tag name generation in Rust derive macro.
   * Converts PascalCase to kebab-case with stencila- prefix.
   */
  private getTagName(nodeType: NodeType): string {
    return `stencila-${this.toKebabCase(nodeType)}`
  }

  /**
   * Convert camelCase/PascalCase to kebab-case
   */
  toKebabCase(str: string): string {
    return str.replace(/[A-Z]/g, (letter, index) => {
      return index === 0 ? letter.toLowerCase() : `-${letter.toLowerCase()}`
    })
  }

  /**
   * Escape HTML entities in text content
   */
  escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
  }

  /**
   * Escape HTML entities in attribute values
   *
   * Mirrors attribute value escaping in Rust implementation.
   */
  private escapeAttributeValue(value: unknown): string {
    let str: string

    if (typeof value === 'object' && value !== null) {
      // Handle Cord type (extract string value)
      if ('string' in value && typeof value.string === 'string') {
        str = value.string
      } else {
        // Serialize complex objects as JSON
        str = JSON.stringify(value)
      }
    } else {
      str = String(value)
    }

    return str
      .replace(/&/g, '&amp;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
  }
}

/**
 * Schema definition for a node type's DOM encoding
 *
 * Based on the `#[dom(...)]` attributes used in Rust schema definitions.
 * Defines the element to use for the node itself (defaults to custom element)
 * and how each field should be encoded (as attribute, slot, or skipped).
 */
interface NodeSchema {
  /** Primary HTML element */
  element?: string

  /** Field encoding specifications */
  fields: Record<string, FieldSchema>
}

/**
 * Schema definition for a field's DOM encoding
 *
 * Mirrors the dom attribute options in Rust schema:
 * - `#[dom(elem = "section")]` -> `element: "section"`
 * - `#[dom(attr = "custom-name")]` -> `attribute: "custom-name"`
 * - `#[dom(skip)]` -> `skip: true`
 * - `#[dom(with = "function")]` -> `encoder: "function"`
 */
interface FieldSchema {
  /** HTML element to wrap field content ("section", "div", "span", "none", null) */
  element?: string | null

  /** Custom attribute name (defaults to kebab-case field name) */
  attribute?: string

  /** Skip this field in DOM encoding */
  skip?: boolean

  /** Custom encoder function name */
  encoder?: string
}

/**
 * Node schema definitions based on Rust schema `#[dom(...)]` attributes
 *
 * These schemas are primarily derived from the Rust struct definitions in
 * `../../rust/schema/src/types/*.rs`.
 * When updating, check for new `#[dom(...)]` attributes there.
 *
 * PREFERRED APPROACH: Most node types should use schema-driven encoding here.
 * This mimics the Rust derive macro behavior and is easier to maintain.
 */
const NODE_SCHEMAS: Partial<Record<NodeType, NodeSchema>> = {
  Admonition: {
    fields: {
      title: { element: 'p' },
      content: { element: 'aside' },
    },
  },

  Article: {
    fields: {
      title: { element: 'h1' },
      authors: { element: 'span' },
      abstract: { element: 'section' },
      content: { element: 'section' },
      references: { element: 'section' },
    },
  },

  Claim: {
    fields: {
      claimType: { attribute: 'claim-type' },
      content: { element: 'div' },
    },
  },

  Emphasis: {
    element: 'em',
    fields: {
      content: { element: 'none' },
    },
  },

  MathBlock: {
    fields: {
      mathLanguage: { attribute: 'math-language' },
      code: { element: 'pre' },
    },
  },

  MathInline: {
    element: 'code',
    fields: {
      mathLanguage: { attribute: 'math-language' },
      code: { element: 'none' },
    },
  },

  Note: {
    fields: {
      content: { element: 'div' },
    },
  },

  Paragraph: {
    fields: {
      content: { element: 'p' },
    },
  },

  QuoteBlock: {
    fields: {
      content: { element: 'blockquote' },
    },
  },

  QuoteInline: {
    element: 'q',
    fields: {
      content: { element: 'none' },
    },
  },

  Section: {
    fields: {
      title: { element: 'h1' },
      depth: { skip: true },
      content: { element: 'section' },
    },
  },

  Strikeout: {
    element: 's',
    fields: {
      content: { element: 'none' },
    },
  },

  Strong: {
    element: 'strong',
    fields: {
      content: { element: 'none' },
    },
  },

  StyledBlock: {
    fields: {
      content: { element: 'div' },
      styleLanguage: { attribute: 'style-language' },
      code: { element: 'style' },
    },
  },

  StyledInline: {
    element: 'span',
    fields: {
      content: { element: 'none' },
      styleLanguage: { attribute: 'style-language' },
      code: { skip: true }, // Inline styles are applied differently
    },
  },

  Subscript: {
    element: 'sub',
    fields: {
      content: { element: 'none' },
    },
  },

  Superscript: {
    element: 'sup',
    fields: {
      content: { element: 'none' },
    },
  },

  ThematicBreak: {
    fields: {},
  },

  Underline: {
    element: 'u',
    fields: {
      content: { element: 'none' },
    },
  },
}

/**
 * Manual encoders for node types that need custom logic
 *
 * ONLY USE WHEN NECESSARY: These should only exist if there is a corresponding
 * manual `impl DomCodec for <NodeType>` in `../../rust/schema/src/implem/*.rs`.
 * Most node types should use schema-driven encoding in NODE_SCHEMAS instead.
 */
const MANUAL_ENCODERS: Partial<
  Record<NodeType, (node: Node, context: EncodeContext) => void>
> = {
  AudioObject: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.contentUrl) {
      attrs['content-url'] = node.contentUrl
    }

    context.enterNode('AudioObject' as NodeType, undefined, attrs)

    // Generate audio element
    if (node.contentUrl) {
      context.html += `<audio src="${node.contentUrl}" controls></audio>`
    }

    // Add title and caption similar to ImageObject
    if (node.title) {
      const title = Array.isArray(node.title)
        ? (node.title as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'AudioObject'])
            )
            .join('')
        : encode(node.title as Node, [...context.ancestors, 'AudioObject'])
      context.pushSlot('span', 'title', title)
    }

    if (node.caption) {
      const caption = Array.isArray(node.caption)
        ? (node.caption as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'AudioObject'])
            )
            .join('')
        : encode(node.caption as Node, [...context.ancestors, 'AudioObject'])
      context.pushSlot('span', 'caption', caption)
    }

    context.exitNode()
  },

  CodeBlock: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.code !== undefined) {
      attrs.code = node.code
    }
    if (node.programmingLanguage) {
      attrs['programming-language'] = node.programmingLanguage
    }

    context.enterNode('CodeBlock' as NodeType, undefined, attrs)

    // Generate pre > code element structure
    const code = String(node.code || '')
    if (code.trim() === '') {
      // Empty or whitespace-only code
      context.html += `<pre><code></code></pre>`
    } else {
      // Non-empty code
      context.html += `<pre><code>${context.escapeHtml(code)}</code></pre>`
    }

    context.exitNode()
  },

  CodeInline: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    attrs.code = node.code
    if (node.programmingLanguage) {
      attrs['programming-language'] = node.programmingLanguage
    }

    context.enterNode('CodeInline' as NodeType, undefined, attrs)

    // Generate code element directly
    context.html += `<code>${context.escapeHtml(String(node.code || ''))}</code>`

    context.exitNode()
  },

  Heading: (node: Heading, context: EncodeContext) => {
    // Prepare attributes
    const attrs: Record<string, unknown> = {}
    if (node.level !== undefined) {
      attrs.level = node.level
    }

    context.enterNode('Heading' as NodeType, undefined, attrs)

    // Content in appropriate heading element
    if (node.content && node.content.length > 0) {
      const level = Math.max(1, Math.min(6, node.level || 1)) // Clamp to 1-6
      const headingTag = `h${level}`
      const content = node.content
        .map((item: Node) => encode(item, [...context.ancestors, 'Heading']))
        .join('')
      context.pushSlot(headingTag, 'content', content)
    }

    context.exitNode()
  },

  ImageObject: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.contentUrl) {
      attrs['content-url'] = node.contentUrl
    }

    context.enterNode('ImageObject' as NodeType, undefined, attrs)

    // Generate img element
    if (node.contentUrl) {
      context.html += `<img src="${node.contentUrl}" />`
    }

    // Add title if present
    if (node.title) {
      const title = Array.isArray(node.title)
        ? (node.title as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'ImageObject'])
            )
            .join('')
        : encode(node.title as Node, [...context.ancestors, 'ImageObject'])
      context.pushSlot('span', 'title', title)
    }

    // Add caption if present
    if (node.caption) {
      const caption = Array.isArray(node.caption)
        ? (node.caption as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'ImageObject'])
            )
            .join('')
        : encode(node.caption as Node, [...context.ancestors, 'ImageObject'])
      context.pushSlot('span', 'caption', caption)
    }

    context.exitNode()
  },

  Link: (node: Link, context: EncodeContext) => {
    // Prepare attributes
    const attrs: Record<string, unknown> = {}
    if (node.target) {
      attrs.target = node.target
    }

    context.enterNode('Link' as NodeType, undefined, attrs)

    // Create anchor element with href and optional title
    let anchorAttrs = `href="${node.target || ''}"`
    if (node.title) {
      anchorAttrs += ` title="${node.title}"`
    }

    // Content wrapped in span with slot
    if (node.content && node.content.length > 0) {
      const content = node.content
        .map((item: Node) => encode(item, [...context.ancestors, 'Link']))
        .join('')
      context.html += `<a ${anchorAttrs}><span slot=content>${content}</span></a>`
    } else {
      context.html += `<a ${anchorAttrs}></a>`
    }

    context.exitNode()
  },

  List: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.order) {
      attrs.order = node.order
    }

    context.enterNode('List' as NodeType, undefined, attrs)

    // Generate ul or ol based on order
    const items = node.items as Node[]
    if (items && items.length > 0) {
      const tag =
        node.order === 'Ascending' || node.order === 'Descending' ? 'ol' : 'ul'
      const content = items
        .map((item: Node) => encode(item, [...context.ancestors, 'List']))
        .join('')
      context.pushSlot(tag, 'items', content)
    }

    context.exitNode()
  },

  ListItem: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.isChecked !== undefined) {
      attrs['is-checked'] = node.isChecked
    }

    context.enterNode('ListItem' as NodeType, undefined, attrs)

    // Generate li element for content
    const content = node.content as Node[]
    if (content && content.length > 0) {
      const encoded = content
        .map((item: Node) => encode(item, [...context.ancestors, 'ListItem']))
        .join('')
      context.pushSlot('li', 'content', encoded)
    }

    context.exitNode()
  },

  Table: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.label !== undefined) {
      attrs.label = node.label
    }
    if (node.labelAutomatically !== undefined) {
      attrs['label-automatically'] = node.labelAutomatically
    }

    context.enterNode('Table' as NodeType, undefined, attrs)

    // Add caption if present
    if (node.caption) {
      const captionArray = Array.isArray(node.caption)
        ? (node.caption as Node[])
        : [node.caption as Node]
      const captionContent = captionArray
        .map((item: Node) => {
          // Add label span before first paragraph if label exists
          if (node.label && item.type === 'Paragraph') {
            return `<stencila-paragraph id="${item.id || 'xxx'}" depth="${context.ancestors.length + 1}" ancestors="${[...context.ancestors, 'Table'].join('.')}"><p slot="content"><span class="table-label">Table ${node.label}</span>${(item.content as Node[]).map((c: Node) => encode(c, [...context.ancestors, 'Table', 'Paragraph'])).join('')}</p></stencila-paragraph>`
          }
          return encode(item, [...context.ancestors, 'Table'])
        })
        .join('')
      context.pushSlot('div', 'caption', captionContent)
    }

    // Add rows
    const rows = node.rows as Record<string, unknown>[]
    if (rows && rows.length > 0) {
      const rowsContent = rows
        .map((row: Record<string, unknown>) => {
          // Encode TableRow
          const cells = row.cells as Record<string, unknown>[]
          const cellsContent = cells
            .map((cell: Record<string, unknown>) => {
              // Encode TableCell
              const cellContent = ((cell.content as Node[]) || [])
                .map((item: Node) =>
                  encode(item, [
                    ...context.ancestors,
                    'Table',
                    'TableRow',
                    'TableCell',
                  ])
                )
                .join('')
              return `<td id="${cell.id || 'xxx'}" depth="${context.ancestors.length + 2}" ancestors="${[...context.ancestors, 'Table', 'TableRow'].join('.')}">${cellContent}</td>`
            })
            .join('')
          return `<tr id="${row.id || 'xxx'}" depth="${context.ancestors.length + 1}" ancestors="${[...context.ancestors, 'Table'].join('.')}">${cellsContent}</tr>`
        })
        .join('')
      context.pushSlot('table', 'rows', rowsContent)
    }

    // Add notes if present
    if (node.notes) {
      const notesArray = Array.isArray(node.notes)
        ? (node.notes as Node[])
        : [node.notes as Node]
      const notesContent = notesArray
        .map((item: Node) => encode(item, [...context.ancestors, 'Table']))
        .join('')
      context.pushSlot('aside', 'notes', notesContent)
    }

    context.exitNode()
  },

  Text: (node: Text, context: EncodeContext) => {
    context.enterNode('Text' as NodeType)

    let text = node.value || ''

    // Handle Cord type - extract string value
    if (typeof text === 'object' && text !== null && 'string' in text) {
      text = text.string
    }

    // Ensure we have a string value and trim whitespace to match Rust implementation
    const textValue =
      typeof text === 'string' ? text.trim() : String(text).trim()
    context.pushText(textValue)
    context.exitNode()
  },

  VideoObject: (node: Record<string, unknown>, context: EncodeContext) => {
    const attrs: Record<string, unknown> = {}
    if (node.contentUrl) {
      attrs['content-url'] = node.contentUrl
    }

    context.enterNode('VideoObject' as NodeType, undefined, attrs)

    // Generate video element
    if (node.contentUrl) {
      context.html += `<video src="${node.contentUrl}" controls></video>`
    }

    // Add title and caption similar to ImageObject
    if (node.title) {
      const title = Array.isArray(node.title)
        ? (node.title as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'VideoObject'])
            )
            .join('')
        : encode(node.title as Node, [...context.ancestors, 'VideoObject'])
      context.pushSlot('span', 'title', title)
    }

    if (node.caption) {
      const caption = Array.isArray(node.caption)
        ? (node.caption as Node[])
            .map((item: Node) =>
              encode(item, [...context.ancestors, 'VideoObject'])
            )
            .join('')
        : encode(node.caption as Node, [...context.ancestors, 'VideoObject'])
      context.pushSlot('span', 'caption', caption)
    }

    context.exitNode()
  },
}

/**
 * Primitive encoder for basic data types
 *
 * Handles null, boolean, number, string, bigint, arrays, and plain objects.
 * Returns HTML string for primitives, or null if not a primitive type.
 * Mirrors the primitive handling patterns in Rust implementation.
 */
function encodePrimitive(node: Node, ancestors: string[] = []): string | null {
  // Handle `Null` nodes - primitive case
  if (node === null) {
    return '<stencila-null>null</stencila-null>'
  }

  // Handle `Primitive` nodes - matches Rust primitive handling
  switch (typeof node) {
    case 'boolean':
      return `<stencila-boolean>${node}</stencila-boolean>`
    case 'number': {
      // Check if it's an integer or float
      const isInteger = Number.isInteger(node)
      const tag = isInteger ? 'integer' : 'number'
      return `<stencila-${tag}>${node}</stencila-${tag}>`
    }
    case 'string':
      return `<stencila-string>${node}</stencila-string>`
    case 'bigint':
      return `<stencila-integer>${node}</stencila-integer>`
  }

  // Handle arrays - encode with array-item wrappers
  if (Array.isArray(node)) {
    let html = '<stencila-array'
    if (ancestors.length === 0) {
      html += ' root'
    }
    html += '>'

    node.forEach((item, index) => {
      html += `<stencila-array-item index=${index}>${encode(item, [...ancestors, 'Array'])}</stencila-array-item>`
    })

    html += '</stencila-array>'
    return html
  }

  // Handle objects without type - encode with object-item wrappers
  if (!Object.prototype.hasOwnProperty.call(node, 'type')) {
    let html = '<stencila-object'
    if (ancestors.length === 0) {
      html += ' root'
    }
    html += '>'

    for (const [key, value] of Object.entries(node)) {
      html += `<stencila-object-item key=${key}>${encode(value, [...ancestors, 'Object'])}</stencila-object-item>`
    }

    html += '</stencila-object>'
    return html
  }

  // Not a primitive type
  return null
}

/**
 * Generic derive-like encoder that mimics the Rust DomCodec derive macro
 *
 * This function implements the logic equivalent to what the Rust derive macro
 * generates in `../../rust/codec-dom-derive/src/lib.rs`. It processes fields
 * based on schema definitions to determine whether they should become attributes,
 * slots, or be skipped entirely.
 *
 * Fields default to being encoded as kebab-case attributes unless explicitly
 * configured otherwise in the schema.
 */
function encodeDerived(
  node: Record<string, unknown>,
  schema: NodeSchema,
  context: EncodeContext
): void {
  const nodeType = node.type as NodeType

  // Common fields that should be skipped by default
  const skipFields = ['type', 'compilationDigest', 'executionDigest', '$schema']

  // Process all fields that exist on the node, not just those in the schema
  const allFields = new Set([
    ...Object.keys(node),
    ...Object.keys(schema.fields),
  ])

  // Collect attributes to pass to enterNode
  const extraAttrs: Record<string, unknown> = {}
  for (const fieldName of allFields) {
    if (skipFields.includes(fieldName)) continue

    const value = node[fieldName]
    if (value === undefined || value === null) {
      continue
    }

    const fieldSchema = schema.fields[fieldName] || {}
    if (fieldSchema.skip) {
      continue
    }

    if (
      fieldSchema.attribute !== undefined ||
      (fieldSchema.element === undefined && fieldSchema.attribute === undefined)
    ) {
      // Field becomes an attribute
      const attrName = fieldSchema.attribute || context.toKebabCase(fieldName)
      extraAttrs[attrName] = value
    }
  }

  context.enterNode(nodeType, undefined, extraAttrs)

  // If schema has a top-level element (e.g., 'em', 'strong'), create that semantic HTML element
  if (schema.element) {
    let elementContent = ''

    // Process fields to build content for the semantic element
    for (const fieldName of allFields) {
      if (skipFields.includes(fieldName)) continue

      const value = node[fieldName]
      if (value === undefined || value === null) {
        continue
      }

      const fieldSchema = schema.fields[fieldName] || {}
      if (fieldSchema.skip) {
        continue
      }

      if (fieldSchema.element === 'none') {
        // Direct content without wrapper - goes directly into the semantic element
        const content = Array.isArray(value)
          ? value
              .map((item: Node) =>
                encode(item, [...context.ancestors, nodeType])
              )
              .join('')
          : encode(value as Node, [...context.ancestors, nodeType])
        elementContent += content
      }
      // Note: For inline marks, we typically only have content fields with element: 'none'
      // Other field types would need additional handling here if needed
    }

    // Create the semantic HTML element with the content
    context.html += `<${schema.element}>${elementContent}</${schema.element}>`
  } else {
    // No top-level element - process fields as slots (attributes already handled)
    for (const fieldName of allFields) {
      if (skipFields.includes(fieldName)) continue

      const value = node[fieldName]
      if (value === undefined || value === null) {
        continue
      }

      const fieldSchema = schema.fields[fieldName] || {}
      if (fieldSchema.skip) {
        continue
      }

      // Only process slots (attributes already handled above)
      if (fieldSchema.element !== undefined) {
        // Field becomes a slot
        if (fieldSchema.element === 'none') {
          // Direct content without wrapper
          const content = Array.isArray(value)
            ? value
                .map((item: Node) =>
                  encode(item, [...context.ancestors, nodeType])
                )
                .join('')
            : encode(value as Node, [...context.ancestors, nodeType])
          if (content) {
            context.pushSlot(null, fieldName, content)
          }
        } else {
          // Content with wrapper element
          const content = Array.isArray(value)
            ? value
                .map((item: Node) =>
                  encode(item, [...context.ancestors, nodeType])
                )
                .join('')
            : encode(value as Node, [...context.ancestors, nodeType])
          if (content) {
            context.pushSlot(fieldSchema.element, fieldName, content)
          }
        }
      }
    }
  }

  context.exitNode()
}

/**
 * Fallback encoder for unknown node types
 *
 * Provides minimal encoding by adding node attributes and skipping metadata fields.
 * Used when no manual encoder or schema is available for a node type.
 */
function encodeFallback(
  node: Record<string, unknown>,
  context: EncodeContext
): void {
  const nodeType = node.type as NodeType
  context.enterNode(nodeType)

  // Add common attributes, skip known metadata fields
  const skipFields = ['type', 'compilationDigest', 'executionDigest']
  for (const [key, value] of Object.entries(node)) {
    if (skipFields.includes(key) || value === undefined || value === null) {
      continue
    }
    context.pushAttribute(key, value)
  }

  context.exitNode()
}

/**
 * Main encoding function - entry point for DOM HTML generation
 *
 * This function determines whether to use manual encoding (for node types with
 * custom implementations) or the generic derive-like encoding (for standard cases).
 *
 * Mirrors the dispatch logic in the Rust implementation where some types have
 * manual `impl DomCodec` and others use the derive macro.
 */
export function encode(node: Node, ancestors: string[] = []): string {
  // Try primitive encoding first - early return if it's a primitive type
  const primitiveResult = encodePrimitive(node, ancestors)
  if (primitiveResult !== null) {
    return primitiveResult
  }

  // Handle typed nodes
  const nodeType = node.type as NodeType
  const context = new EncodeContext(ancestors)

  // Prefer schema-driven encoding first (mirrors Rust derive macro behavior)
  const schema = NODE_SCHEMAS[nodeType]
  if (schema) {
    encodeDerived(node as Record<string, unknown>, schema, context)
    return context.getHtml()
  }

  // Fall back to manual encoders only when necessary (mirrors Rust manual implementations)
  if (MANUAL_ENCODERS[nodeType]) {
    MANUAL_ENCODERS[nodeType](node, context)
    return context.getHtml()
  }

  // Final fallback for unknown node types - use minimal encoding
  encodeFallback(node as Record<string, unknown>, context)
  return context.getHtml()
}
