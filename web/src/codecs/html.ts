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

import type {
  Admonition,
  AudioObject,
  Block,
  CodeBlock,
  CodeChunk,
  CodeInline,
  Cord,
  Entity,
  Figure,
  Heading,
  ImageObject,
  Inline,
  InstructionMessage,
  Link,
  List,
  ListItem,
  Node,
  NodeType,
  StyledBlock,
  StyledInline,
  Table,
  Text,
  VideoObject,
} from '@stencila/types'

type Attrs = Record<string, unknown>

/**
 * Context for DOM encoding that tracks HTML generation state
 *
 * Mirrors the `DomEncodeContext` struct in `../../rust/codec-dom-trait/src/lib.rs`
 * which manages HTML building, node tracking, and encoding state.
 */
class EncodeContext {
  public html: string = ''
  private nodeStack: string[] = []
  public ancestors: NodeType[]

  constructor(ancestors: NodeType[] = []) {
    this.ancestors = [...ancestors]
  }

  /**
   * Enter a new node, creating the web component element
   *
   * Mirrors `context.enter_node()` in Rust.
   * Generates the opening tag with depth, ancestors, and root attributes.
   */
  enterNode(nodeType: NodeType, extraAttrs: Attrs = {}): void {
    this.nodeStack.push(nodeType)
    const tagName = this.getTagName(nodeType)

    let attrs =
      this.formatAttribute('id', 'xxx') +
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
      const tagName = this.getTagName(nodeType as NodeType)
      this.html += `</${tagName}>`
    }
  }

  /**
   * Format an attribute name-value pair for HTML
   *
   * Helper method to consistently format attributes with kebab-case names and
   * escaped values. Uses smart quote style: no quotes for simple values, single
   * quotes for empty strings, double quotes for complex values.
   *
   * This single quoting and escaping behavior is consistent with the
   * `DomEncodeContext.push_attr_value` method.
   */
  private formatAttribute(name: string, value: unknown): string {
    if (value === null || value === undefined) return ''

    const attrName = this.toKebabCase(name)

    let attrValue: string
    if (typeof value === 'object' && value !== null) {
      // Handle Cord type (extract string value)
      if ('string' in value && typeof value.string === 'string') {
        attrValue = value.string
      } else {
        // Serialize complex objects as JSON
        attrValue = JSON.stringify(value)
      }
    } else {
      attrValue = String(value)
    }

    if (attrValue.length == 0 || /["' \t\n\\/><=]/g.test(attrValue)) {
      // Use single quoting escaping (more terse for JSON attributes because inner double
      // quotes do not need escaping)
      attrValue = attrValue
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/'/g, '&#x27;')
    }

    return ` ${attrName}='${attrValue}'`
  }

  /**
   * Add a slot with wrapper element
   *
   * Mirrors `context.push_slot_fn()` used in Rust manual implementations.
   * Creates semantic HTML structure with proper slot attributes.
   */
  pushSlot(
    tagName: string | null,
    slotName: string,
    content: string,
    attrs: Attrs = {}
  ): void {
    if (tagName === null) {
      this.html += content
    } else {
      const slot = this.toKebabCase(slotName)
      const slotAttr = this.formatAttribute('slot', slot)
      const extraAttrs = Object.entries(attrs)
        .map(([name, value]) => this.formatAttribute(name, value))
        .join('')
      this.html += `<${tagName}${slotAttr}${extraAttrs}>${content}</${tagName}>`
    }
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
   *
   * This is consistent with the `html_escape::encode_safe` function used in Rust.
   */
  escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;')
      .replace(/\//g, '&#x2F;')
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
  /** Skip this field in DOM encoding */
  skip?: boolean

  /** Force this field to be of this type in DOM encoding even if value is undefined or array empty */
  force?: NodeType

  /** Whether this should be treated as a single node */
  singular?: boolean

  /** HTML element to wrap field content ("section", "div", "span", "none", null) */
  element?: string | null

  /** Custom attribute name (defaults to kebab-case field name) */
  attribute?: string

  /** If an attribute its position */
  position?: number

  /** Custom encoder function name */
  encoder?: (value: unknown) => string
}

// Common fields that should be skipped by default
const SKIP_FIELDS = ['type', 'compilationDigest', 'executionDigest', '$schema']

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
  Article: {
    fields: {
      title: { element: 'h1' },
      authors: { element: 'section' },
      provenance: { element: 'div' },
      abstract: { element: 'section' },
      content: { element: 'section' },
      references: { element: 'section' },
    },
  },

  CallBlock: {
    fields: {
      arguments: { element: 'div' },
      content: { element: 'div' },
    },
  },

  CallArgument: {
    fields: {
      value: { attribute: 'value', encoder: (value) => JSON.stringify(value) },
      code: { attribute: 'code', position: 10 },
    },
  },

  Claim: {
    fields: {
      claimType: { attribute: 'claim-type' },
      content: { element: 'aside' },
      authors: { element: 'div' },
      provenance: { element: 'div' },
    },
  },

  CodeExpression: {
    fields: {
      executionMode: { attribute: 'execution-mode', position: -20 },
      code: { attribute: 'code', position: -10 },
      output: { element: 'span', singular: true },
    },
  },

  Emphasis: {
    element: 'em',
    fields: {
      content: { element: 'none' },
    },
  },

  ForBlock: {
    fields: {
      code: { attribute: 'code', position: 10 },
      programmingLanguage: { attribute: 'programming-language', position: 20 },
      variable: { attribute: 'variable', position: 30 },
      content: { element: 'div' },
      otherwise: { element: 'div' },
    },
  },

  IfBlock: {
    fields: {
      clauses: { element: 'div' },
    },
  },

  IfBlockClause: {
    fields: {
      content: { element: 'div' },
    },
  },

  IncludeBlock: {
    fields: {
      executionMode: { attribute: 'execution-mode', position: -10 },
      content: { element: 'div' },
    },
  },

  InstructionBlock: {
    fields: {
      prompt: { element: 'div', force: 'PromptBlock', position: -30 },
      message: { element: 'div', force: 'InstructionMessage', position: -20 },
      modelParameters: {
        element: 'div',
        force: 'ModelParameters',
        position: -10,
      },
      content: { element: 'div' },
      suggestions: { element: 'div' },
    },
  },

  MathBlock: {
    fields: {
      code: { attribute: 'code', position: 10 },
      mathLanguage: { attribute: 'math-language', position: 20 },
      compilationMessages: { element: 'div' },
      authors: { element: 'div' },
      provenance: { element: 'div' },
      mathml: { element: 'div' },
      images: { element: 'div' },
    },
  },

  MathInline: {
    fields: {
      code: { attribute: 'code', position: 10 },
      mathLanguage: { attribute: 'math-language', position: 20 },
      compilationMessages: { element: 'span' },
      authors: { element: 'span' },
      provenance: { element: 'span' },
      mathml: { element: 'span' },
      images: { element: 'span' },
    },
  },

  Note: {
    fields: {
      content: { element: 'aside' },
    },
  },

  Paragraph: {
    fields: {
      content: { element: 'p' },
    },
  },

  Parameter: {
    fields: {
      value: { attribute: 'value', encoder: (value) => JSON.stringify(value) },
      default: {
        attribute: 'default',
        encoder: (value) => JSON.stringify(value),
        position: 10,
      },
      validator: { element: 'span' },
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

  SuggestionBlock: {
    fields: {
      content: { element: 'div' },
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
  Admonition: (node: Admonition, context: EncodeContext) => {
    context.enterNode('Admonition', { 'admonition-type': node.admonitionType, 'is-folded': node.isFolded })

    const ancestors = [...context.ancestors, 'Admonition' as NodeType]

    // Determine if details should be open
    const isOpen = node.isFolded === undefined || node.isFolded === false
    const detailsOpen = isOpen ? ' open' : ''

    context.html += `<details${detailsOpen}>\n`
    context.html += '<summary>\n'

    // Encode title
    if (node.title && Array.isArray(node.title) && node.title.length > 0) {
      context.pushSlot('span', 'title', encodeInlines(node.title, ancestors))
    } else {
      // Use admonitionType as default title
      const defaultTitle = node.admonitionType || 'Note'
      context.html += `<span slot="title">${defaultTitle}</span>\n`
    }

    context.html += '</summary>\n'

    // Encode content
    if (node.content && node.content.length > 0) {
      context.pushSlot('div', 'content', encodeBlocks(node.content, ancestors))
    }

    context.html += '</details>'

    context.exitNode()
  },

  AudioObject: (node: AudioObject, context: EncodeContext) => {
    context.enterNode('AudioObject', { 'content-url': node.contentUrl })

    context.html += `<audio src="${node.contentUrl}" controls></audio>`

    const ancestors = [...context.ancestors, 'AudioObject' as NodeType]

    if (node.title) {
      context.pushSlot('span', 'title', encodeInlines(node.title, ancestors))
    }

    if (node.caption) {
      context.pushSlot(
        'span',
        'caption',
        encodeInlines(node.caption, ancestors)
      )
    }

    context.exitNode()
  },

  CodeBlock: (node: CodeBlock, context: EncodeContext) => {
    context.enterNode('CodeBlock', {
      code: node.code,
      'programming-language': node.programmingLanguage,
    })

    const ancestors = [...context.ancestors, 'CodeBlock' as NodeType]

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    context.html += `<pre><code>${context.escapeHtml(cordToString(node.code))}</code></pre>`

    context.exitNode()
  },

  CodeInline: (node: CodeInline, context: EncodeContext) => {
    context.enterNode('CodeInline', {
      code: node.code,
      'programming-language': node.programmingLanguage,
    })

    const ancestors = [...context.ancestors, 'CodeInline' as NodeType]

    if (node.authors) {
      context.pushSlot('span', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'span',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    context.html += `<code>${context.escapeHtml(String(cordToString(node.code)))}</code>`

    context.exitNode()
  },

  CodeChunk: (node: CodeChunk, context: EncodeContext) => {
    context.enterNode('CodeChunk', {
      'execution-mode': node.executionMode,
      'execution-bounds': node.executionBounds,
      code: node.code,
      'programming-language': node.programmingLanguage,
      'label-type': node.labelType,
      label: node.label,
      'label-automatically': node.labelAutomatically,
      'is-echoed': node.isEchoed,
      'is-hidden': node.isHidden,
      'execution-ended': node.executionEnded,
      'execution-duration': node.executionDuration,
    })

    const ancestors = [...context.ancestors, 'CodeChunk' as NodeType]

    if (node.compilationMessages) {
      context.pushSlot(
        'div',
        'compilation-messages',
        encodeNodes(node.compilationMessages, ancestors)
      )
    }

    if (node.executionMessages) {
      context.pushSlot(
        'div',
        'execution-messages',
        encodeNodes(node.executionMessages, ancestors)
      )
    }

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    if (node.labelType == 'TableLabel') {
      context.pushSlot(
        'div',
        'caption',
        encodeCaption(node.caption, 'Table', node.label, ancestors)
      )
    }

    // Outputs

    if (node.labelType == 'FigureLabel') {
      context.pushSlot(
        'div',
        'caption',
        encodeCaption(node.caption, 'Figure', node.label, ancestors)
      )
    }

    context.exitNode()
  },

  Figure: (node: Figure, context: EncodeContext) => {
    context.enterNode('Figure', {
      label: node.label,
      'label-automatically': node.labelAutomatically,
    })

    const ancestors = [...context.ancestors, 'Figure' as NodeType]

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    // Build figure content with caption inside
    let figureContent = encodeNodes(node.content, ancestors)

    if (node.caption) {
      figureContent += '<figcaption>'
      figureContent += encodeCaption(node.caption, 'Figure', node.label, ancestors)
      figureContent += '</figcaption>'
    }

    context.pushSlot('figure', 'content', figureContent)

    context.exitNode()
  },

  Heading: (node: Heading, context: EncodeContext) => {
    context.enterNode('Heading', { level: node.level })

    if (node.content && node.content.length > 0) {
      const level = Math.max(1, Math.min(6, node.level)) // Clamp to 1-6
      const headingTag = `h${level}`
      const content = node.content
        .map((item: Node) => encode(item, [...context.ancestors, 'Heading']))
        .join('')
      context.pushSlot(headingTag, 'content', content)
    }

    const ancestors = [...context.ancestors, 'Heading' as NodeType]

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    context.exitNode()
  },

  ImageObject: (node: ImageObject, context: EncodeContext) => {
    context.enterNode('ImageObject', {
      'content-url': node.contentUrl,
    })

    context.html += `<img src="${node.contentUrl}" />`

    const ancestors = [...context.ancestors, 'ImageObject' as NodeType]

    if (node.title) {
      context.pushSlot('span', 'title', encodeInlines(node.title, ancestors))
    }

    if (node.caption) {
      context.pushSlot(
        'span',
        'caption',
        encodeInlines(node.caption, ancestors)
      )
    }

    context.exitNode()
  },

  InstructionMessage: (node: InstructionMessage, context: EncodeContext) => {
    context.enterNode('InstructionMessage', { role: node.role })

    if (node.parts && node.parts.length > 0) {
      let parts = ''
      for (const part of node.parts) {
        let type
        let value
        if (part.type == 'Text') {
          type = 'text'
          value = cordToString(part.value)
        } else if (part.type == 'AudioObject') {
          type = 'audio'
          value = part.contentUrl
        } else if (part.type == 'ImageObject') {
          type = 'image'
          value = part.contentUrl
        } else if (part.type == 'VideoObject') {
          type = 'video'
          value = part.contentUrl
        }

        if (type && value) {
          parts += `<stencila-message-part type=${type} value="${context.escapeHtml(value)}"></stencila-message-part>`
        }
      }
      context.pushSlot('div', 'parts', parts)
    }

    context.exitNode()
  },

  Link: (node: Link, context: EncodeContext) => {
    context.enterNode('Link', { target: node.target })

    // Create anchor element with href and optional title
    let anchorAttrs = `href="${node.target || ''}"`
    if (node.title) {
      anchorAttrs += ` title="${node.title}"`
    }

    // Content wrapped in span with slot
    if (node.content && node.content.length > 0) {
      const content = node.content
        .map((item) => encode(item, [...context.ancestors, 'Link']))
        .join('')
      context.html += `<a ${anchorAttrs}><span slot=content>${content}</span></a>`
    } else {
      context.html += `<a ${anchorAttrs}></a>`
    }

    context.exitNode()
  },

  List: (node: List, context: EncodeContext) => {
    context.enterNode('List', { order: node.order })

    // Generate ul or ol based on order
    const items = node.items
    if (items && items.length > 0) {
      const tag =
        node.order === 'Ascending' || node.order === 'Descending' ? 'ol' : 'ul'
      const content = items
        .map((item) => encode(item, [...context.ancestors, 'List']))
        .join('')
      context.pushSlot(tag, 'items', content)
    }

    context.exitNode()
  },

  ListItem: (node: ListItem, context: EncodeContext) => {
    context.enterNode('ListItem', { 'is-checked': node.isChecked })

    if (node.content) {
      context.pushSlot(
        'li',
        'content',
        encodeBlocks(node.content, [...context.ancestors, 'ListItem'])
      )
    }

    context.exitNode()
  },

  StyledBlock: (node: StyledBlock, context: EncodeContext) => {
    context.enterNode('StyledBlock', {
      code: node.code,
      'style-language': node.styleLanguage,
      css: node.css,
    })

    const ancestors = [...context.ancestors, 'StyledBlock' as NodeType]

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    context.pushSlot('div', 'content', encodeNodes(node.content, ancestors), {
      class: node.classList,
    })

    context.exitNode()
  },

  StyledInline: (node: StyledInline, context: EncodeContext) => {
    context.enterNode('StyledInline', {
      code: node.code,
      'style-language': node.styleLanguage,
      css: node.css,
    })

    const ancestors = [...context.ancestors, 'StyledInline' as NodeType]

    if (node.authors) {
      context.pushSlot('span', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'span',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    context.pushSlot('span', 'content', encodeNodes(node.content, ancestors), {
      class: node.classList,
    })

    context.exitNode()
  },

  Table: (node: Table, context: EncodeContext) => {
    context.enterNode('Table', {
      label: node.label,
      'label-automatically': node.labelAutomatically,
    })

    const ancestors = [...context.ancestors, 'Table' as NodeType]

    if (node.authors) {
      context.pushSlot('div', 'authors', encodeNodes(node.authors, ancestors))
    }

    if (node.provenance) {
      context.pushSlot(
        'div',
        'provenance',
        encodeNodes(node.provenance, ancestors)
      )
    }

    if (node.rows && node.rows.length > 0) {
      let tableContent = ''

      // Add caption inside table if it exists
      if (node.caption) {
        tableContent += '<caption>'
        tableContent += encodeCaption(node.caption, 'Table', node.label, ancestors)
        tableContent += '</caption>'
      }

      // Generate rows
      const rowsContent = node.rows
        .map((row) => {
          const rowContent = row.cells
            .map((cell) => {
              let content: Block[] | Inline[] = cell.content || []
              if (content.length == 1 && content[0].type == 'Paragraph') {
                content = content[0].content
              }

              const cellContent = content
                .map((item: Node) =>
                  encode(item, [
                    ...context.ancestors,
                    'Table',
                    'TableRow',
                    'TableCell',
                  ])
                )
                .join('')

              let style = ''
              if (cell.horizontalAlignment == 'AlignLeft') {
                style = ' style="text-align: left"'
              } else if (cell.horizontalAlignment == 'AlignCenter') {
                style = ' style="text-align: center"'
              } else if (cell.horizontalAlignment == 'AlignRight') {
                style = ' style="text-align: right"'
              }

              // Use th for header cells, td otherwise
              const tag = cell.cellType === 'HeaderCell' ? 'th' : 'td'
              return `<${tag} id="${cell.id || 'xxx'}" depth="${context.ancestors.length + 2}" ancestors="${[...ancestors, 'TableRow'].join('.')}"${style}>${cellContent}</${tag}>`
            })
            .join('')
          return `<tr id="${row.id || 'xxx'}" depth="${context.ancestors.length + 1}" ancestors="${ancestors.join('.')}">${rowContent}</tr>`
        })
        .join('')

      tableContent += rowsContent
      context.pushSlot('table', 'rows', tableContent)
    }

    if (node.notes) {
      context.pushSlot('aside', 'notes', encodeBlocks(node.notes, ancestors))
    }

    context.exitNode()
  },

  Text: (node: Text, context: EncodeContext) => {
    context.enterNode('Text')

    let text = node.value || ''

    // Handle Cord type - extract string value
    if (typeof text === 'object' && text !== null && 'string' in text) {
      text = text.string
    }

    // Ensure we have a string value and trim whitespace to match Rust implementation
    const textValue =
      typeof text === 'string' ? text.trim() : String(text).trim()
    context.html += context.escapeHtml(textValue)

    context.exitNode()
  },

  ThematicBreak: (_node: Node, context: EncodeContext) => {
    context.enterNode('ThematicBreak')
    context.html += '<hr>'
    context.exitNode()
  },

  VideoObject: (node: VideoObject, context: EncodeContext) => {
    context.enterNode('VideoObject', { 'content-url': node.contentUrl })

    context.html += `<video src="${node.contentUrl}" controls></video>`

    const ancestors = [...context.ancestors, 'VideoObject' as NodeType]

    if (node.title) {
      context.pushSlot('span', 'title', encodeInlines(node.title, ancestors))
    }

    if (node.caption) {
      context.pushSlot(
        'span',
        'caption',
        encodeInlines(node.caption, ancestors)
      )
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
function encodePrimitive(
  node: Node,
  ancestors: NodeType[] = []
): string | null {
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

  // Process all fields that exist on the node, not just those in the schema
  const allFields = new Set([
    ...Object.keys(node),
    ...Object.keys(schema.fields),
  ])

  // Sort fields according to any `position` corresponding field schema (if any)
  const sortedFields = Array.from(allFields).sort((a, b) => {
    const aSchema = schema.fields[a] || {}
    const bSchema = schema.fields[b] || {}
    const aPosition = aSchema.position ?? 0
    const bPosition = bSchema.position ?? 0
    return aPosition - bPosition
  })

  // Collect attributes to pass to enterNode
  const attrs: Attrs = {}
  for (const fieldName of sortedFields) {
    if (SKIP_FIELDS.includes(fieldName)) continue

    const value = node[fieldName]
    if (value === undefined || value === null) {
      continue
    }

    const fieldSchema = schema.fields[fieldName] || {}
    if (fieldSchema.skip || fieldSchema.element !== undefined) {
      continue
    }

    if (
      fieldSchema.attribute !== undefined ||
      (fieldSchema.element === undefined && fieldSchema.attribute === undefined)
    ) {
      // Field becomes an attribute
      const attrName = fieldSchema.attribute || context.toKebabCase(fieldName)
      attrs[attrName] = fieldSchema.encoder ? fieldSchema.encoder(value) : value
    }
  }

  context.enterNode(nodeType, attrs)

  // If schema has a top-level element (e.g., 'em', 'strong'), create that semantic HTML element
  if (schema.element) {
    let elementContent = ''

    // Process fields to build content for the semantic element
    for (const fieldName of sortedFields) {
      if (SKIP_FIELDS.includes(fieldName)) {
        continue
      }

      const value = node[fieldName]
      if (value === undefined) {
        continue
      }

      const fieldSchema = schema.fields[fieldName] || {}
      if (fieldSchema.skip) {
        continue
      }

      if (fieldSchema.element === 'none') {
        // Direct content without wrapper - goes directly into the semantic element
        const content = Array.isArray(value)
          ? encodeNodes(value, [...context.ancestors, nodeType])
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
    for (const fieldName of sortedFields) {
      if (SKIP_FIELDS.includes(fieldName)) {
        continue
      }

      const fieldSchema = schema.fields[fieldName] || {}
      if (fieldSchema.skip || fieldSchema.element == undefined) {
        continue
      }

      let value = node[fieldName]
      if (value == undefined) {
        if (fieldSchema.force) {
          value = { type: fieldSchema.force }
        } else {
          continue
        }
      }
      if (Array.isArray(value) && value.length == 0) {
        continue
      }

      const content =
        Array.isArray(value) && !fieldSchema.singular
          ? encodeNodes(value, [...context.ancestors, nodeType])
          : encode(value as Node, [...context.ancestors, nodeType])

      // Field becomes a slot
      if (fieldSchema.element === 'none') {
        // Direct content without wrapper
        context.pushSlot(null, fieldName, content)
      } else {
        // Content with wrapper element
        context.pushSlot(fieldSchema.element, fieldName, content)
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
function encodeFallback(node: Entity, context: EncodeContext): void {
  const attrs: Attrs = {}

  for (const [key, value] of Object.entries(node)) {
    if (SKIP_FIELDS.includes(key) || value === undefined || value === null) {
      continue
    }
    attrs[key] = value
  }

  context.enterNode(node.type as NodeType, attrs)
  context.exitNode()
}

/**
 * Encode a `Cord` as a string
 *
 * This is necessary because cords may be serialized to JSON as
 * a plain string or a Cord object (with `string` and possibly `authorship` properties)
 */
function cordToString(cord: Cord | string): string {
  if (typeof cord == 'string') {
    return cord
  }

  if (typeof cord === 'object' && cord !== null) {
    if ('string' in cord && typeof cord.string === 'string') {
      return cord.string
    }
  }

  return String(cord)
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
export function encode(node: Node, ancestors: NodeType[] = []): string {
  // Try primitive encoding first - early return if it's a primitive type
  const primitiveResult = encodePrimitive(node, ancestors)
  if (primitiveResult !== null) {
    return primitiveResult
  }

  const entity = node as unknown as Entity

  // Handle typed nodes
  const nodeType = entity.type as NodeType
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
  encodeFallback(entity, context)
  return context.getHtml()
}

function encodeNodes(nodes: Node[], ancestors: NodeType[] = []): string {
  return nodes.map((block) => encode(block, ancestors)).join('')
}

function encodeBlocks(blocks: Block[], ancestors: NodeType[] = []): string {
  return encodeNodes(blocks, ancestors)
}

function encodeInlines(inlines: Inline[], ancestors: NodeType[] = []): string {
  return encodeNodes(inlines, ancestors)
}

function encodeCaption(
  blocks: Block[],
  type: 'Table' | 'Figure',
  label: string | undefined,
  ancestors: NodeType[] = []
): string {
  return blocks
    .map((block, index) => {
      if (index == 0 && block.type === 'Paragraph') {
        return (
          `<stencila-paragraph id=xxx depth="${ancestors.length}" ancestors="${ancestors.join('.')}">` +
          `<p slot="content">` +
          `<span class="${type.toLowerCase()}-label">${type}${label ? ` ${label}` : ''}</span>:` +
          encodeInlines(block.content, [...ancestors, 'Paragraph']) +
          `</p></stencila-paragraph>`
        )
      } else {
        return encode(block, ancestors)
      }
    })
    .join('')
}
