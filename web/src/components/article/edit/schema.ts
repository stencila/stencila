import {
  AttributeSpec,
  MarkSpec,
  Node,
  NodeSpec,
  ParseRule,
  Schema,
} from 'prosemirror-model'

/**
 * ProseMirror schema for a Stencila `Article`.
 *
 * This schema uses the following conventions:
 *
 * - Properties of Stencila nodes are represented as ProseMirror `NodeSpec`s with
 *   a lowercase name (e.g. `title`, `abstract`) and `toDOM` and `parseDOM` rules
 *   which use the corresponding `data-prop` selector (e.g. [data-prop=title]).
 *   The `prop` function is a shortcut for creating these node specs.
 *
 * - Stencila node types are represented as ProseMirror node types with title
 *   case name (e.g. `Paragraph`), a `toDOM` rule that includes `itemtype` (and `itemscope`)
 *   for application of semantic themes, and `parseDOM` rules that are as simple as
 *   possible (for copy-paste) compatibility.
 *
 * - Stencila node types can define a `contentProp` which is the name of the node property
 *   that will be used when generating an address e.g. `contentProp: 'cells'`
 *
 * These conventions make it possible to convert a ProseMirror offset position e.g. `83`
 * into a Stencila address e.g. `["content", 1, "caption", 4]`.
 *
 * Note: When adding types here, please ensure transformations are handled in the
 * `transformProsemirror` function.
 *
 * For docs and examples see:
 *  - https://prosemirror.net/docs/guide/#schema
 *  - https://prosemirror.net/examples/schema/
 *  - https://github.com/ProseMirror/prosemirror-schema-basic/blob/master/src/schema-basic.js
 */
export const articleSchema = new Schema({
  topNode: 'Article',
  nodes: {
    // Article type and its properties
    Article: { content: 'title? abstract? content' },
    title: prop('title', 'div', 'InlineContent*'),
    abstract: prop('abstract', 'div', 'BlockContent+'),
    content: prop('content', 'div', 'BlockContent+'),

    // Block content types. Note that order is important as the
    // first is the default block content
    Paragraph: block('Paragraph', 'p', 'InlineContent*'),
    Heading: heading(),
    List: list(),
    ListItem: listItem(),
    CodeBlock: codeBlock(),
    QuoteBlock: block('QuoteBlock', 'blockquote', 'BlockContent+'),
    Table: table(),
    TableRow: tableRow(),
    TableCell: tableCell(),
    TableHeader: tableHeader(),
    ThematicBreak: thematicBreak(),

    // Inline content types, starting with `text` (equivalent of Stencila `String`),
    // the default inline node type
    text: {
      group: 'InlineContent',
    },
    CodeFragment: codeFragment(),
  },

  marks: {
    Emphasis: mark('Emphasis', 'em', [
      { tag: 'em' },
      { tag: 'i' },
      { style: 'font-style=italic' },
    ]),
    Strong: mark('Strong', 'strong', [
      { tag: 'strong' },
      { tag: 'b' },
      {
        style: 'font-weight',
        getAttrs: (value) =>
          /^(bold(er)?|[5-9]\d{2,})$/.test(value as string) && null,
      },
    ]),
    NontextualAnnotation: mark('NontextualAnnotation', 'u'),
    Delete: mark('Delete', 'del'),
    Subscript: mark('Subscript', 'sub'),
    Superscript: mark('Superscript', 'sup'),
  },
})

export const articleMarks = Object.keys(articleSchema.marks)

/**
 * Generate a `NodeSpec` to represent the property of a Stencila node type.
 *
 * @param name The name of the property
 * @param tag The tag name of the HTML element for `toDOM` and `parseDOM`)
 * @param content The expression specifying valid content for the property e.g `InlineContent+`
 * @param marks The expression specifying valid marks for the property e.g '_' (all), '' (none)
 */
function prop(
  name: string,
  tag: string,
  content: string,
  marks = '_'
): NodeSpec {
  return {
    content,
    marks,
    defining: true,
    parseDOM: [{ tag: `${tag}[data-itemprop=${name}]` }],
    toDOM(_node) {
      return [tag, { 'data-itemprop': name }, 0]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `BlockContent` node type.
 *
 * @param name The name of the type e.g. `Paragraph`
 * @param group The content group that the type belongs to
 * @param tag The tag name of the HTML element for `toDOM` and `parseDOM`)
 * @param content The expression specifying valid content for the property e.g `InlineContent+`
 * @param marks The expression specifying valid marks for the property e.g '_' (all), '' (none)
 */
function block(
  name: string,
  tag: string,
  content: string,
  marks = '_'
): NodeSpec {
  return {
    group: 'BlockContent',
    content,
    marks,
    defining: true,
    parseDOM: [{ tag }],
    toDOM(_node) {
      return [
        tag,
        { itemtype: `http://schema.stenci.la/${name}`, itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `Heading`.
 *
 * Note that, consistent with treatment elsewhere, `h2` => level 3 etc.
 * This is because there should only be one `h1` (for the title) and when encoding to
 * HTML we add one to the depth e.g. `depth: 1` => `h2`
 */
function heading(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'InlineContent*',
    marks: '_',
    defining: true,
    attrs: { depth: { default: 1 } },
    parseDOM: [
      { tag: 'h1', attrs: { depth: 1 } },
      { tag: 'h2', attrs: { depth: 1 } },
      { tag: 'h3', attrs: { depth: 2 } },
      { tag: 'h4', attrs: { depth: 3 } },
      { tag: 'h5', attrs: { depth: 4 } },
      { tag: 'h6', attrs: { depth: 5 } },
    ],
    toDOM(node) {
      return [
        `h${(node.attrs.depth as number) + 1}`,
        { itemtype: 'http://schema.stenci.la/Heading', itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `List`
 *
 * See https://github.com/ProseMirror/prosemirror-schema-list/blob/master/src/schema-list.js
 * for slightly different node specs for lists.
 */
function list(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'ListItem*',
    contentProp: 'items',
    attrs: { order: { default: 'Unordered' } },
    parseDOM: [
      { tag: 'ul', attrs: { order: 'Unordered' } },
      { tag: 'ol', attrs: { order: 'Ascending' } },
    ],
    toDOM(node) {
      return [
        node.attrs.order === 'Unordered' ? 'ul' : 'ol',
        { itemtype: 'http://schema.org/ItemList', itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `ListItem`
 *
 * See https://github.com/ProseMirror/prosemirror-schema-list/blob/master/src/schema-list.js#L50
 * for why the `content` is defined as it is: to be able to use the commands in `prosemirror-schema-list`
 * package
 */
function listItem(): NodeSpec {
  return {
    content: 'Paragraph BlockContent*',
    parseDOM: [{ tag: 'li' }],
    toDOM(_node) {
      return [
        'li',
        { itemtype: 'http://schema.org/ListItem', itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `CodeBlock`
 *
 * This is temporary and wil be replaced with a CodeMirror editor
 * (see https://prosemirror.net/examples/codemirror/ and https://gist.github.com/BrianHung/08146f89ea903f893946963570263040).
 *
 * Based on https://github.com/ProseMirror/prosemirror-schema-basic/blob/b5ae707ab1be98a1d8735dfdc7d1845bcd126f18/src/schema-basic.js#L59
 */
function codeBlock(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'text*',
    contentProp: 'text',
    marks: '',
    attrs: {
      programmingLanguage: { default: '' },
    },
    code: true,
    defining: true,
    parseDOM: [
      {
        tag: 'pre',
        preserveWhitespace: 'full',
        getAttrs(dom) {
          const elem = dom as HTMLElement
          return {
            programmingLanguage:
              elem
                .querySelector('meta[itemprop="programmingLanguage"][content]')
                ?.getAttribute('content') ??
              elem
                .querySelector('code[class^="language-"]')
                ?.getAttribute('class')
                ?.substring(9),
          }
        },
      },
    ],
    toDOM(node) {
      return [
        'pre',
        {
          itemtype: 'http://schema.stenci.la/CodeBlock',
          itemscope: '',
          // This is just for inspection that language is parsed properly
          title: node.attrs.programmingLanguage as string,
        },
        ['code', 0],
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `CodeFragment`
 *
 * This is temporary and wil be replaced with a CodeMirror editor (as with `CodeBlock`)
 */
function codeFragment(): NodeSpec {
  return {
    inline: true,
    group: 'InlineContent',
    content: 'text*',
    contentProp: 'text',
    marks: '',
    attrs: {
      programmingLanguage: { default: '' },
    },
    code: true,
    parseDOM: [{ tag: 'pre', preserveWhitespace: 'full' }],
    toDOM(_node) {
      return [
        'code',
        { itemtype: 'http://schema.stenci.la/CodeFragment', itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `Table`
 *
 * This, and other table related schemas are compatible with `prosemirror-tables` (e.g. `tableRole`
 * and `isolating`) attributes but with Stencila compatible naming.
 *
 * See https://github.com/ProseMirror/prosemirror-tables/blob/master/src/schema.js
 */
function table(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'TableRow+',
    contentProp: 'rows',
    tableRole: 'table',
    isolating: true,
    parseDOM: [{ tag: 'table' }],
    toDOM(_node) {
      return [
        'table',
        { itemtype: 'http://schema.org/Table', itemscope: '' },
        ['tbody', 0],
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `TableRow`.
 */
function tableRow(): NodeSpec {
  return {
    content: '(TableHeader|TableCell)*',
    contentProp: 'cells',
    tableRole: 'row',
    parseDOM: [{ tag: 'tr' }],
    toDOM(_node) {
      return [
        'tr',
        { itemtype: 'http://schema.stenci.la/TableRow', itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * The attributes of a `TableCell`
 */
function tableCellAttrsSpec(): Record<string, AttributeSpec> {
  return {
    colspan: { default: 1 },
    rowspan: { default: 1 },
    colwidth: { default: null },
  }
}

/**
 * Get `TableCell` attributes as part of `parseDOM`
 */
function tableCellAttrsGet(dom: HTMLElement): Record<string, unknown> {
  const widthAttr = dom.getAttribute('data-colwidth') ?? ''
  const widths = /^\d+(,\d+)*$/.test(widthAttr)
    ? widthAttr.split(',').map((s) => Number(s))
    : null
  const colspan = Number(dom.getAttribute('colspan') ?? 1)

  return {
    colspan,
    rowspan: Number(dom.getAttribute('rowspan') ?? 1),
    colwidth: widths && widths.length === colspan ? widths : null,
  }
}

/**
 * Set `TableCell` attributes as part of `toDOM`
 */
function tableCellAttrsSet(node: Node): Record<string, string | number> {
  const attrs: Record<string, string | number> = {
    itemtype: 'http://schema.stenci.la/TableCell',
    itemscope: '',
  }

  if (node.attrs.colspan !== 1) attrs.colspan = node.attrs.colspan as number
  if (node.attrs.rowspan !== 1) attrs.rowspan = node.attrs.rowspan as number
  if (node.attrs.colwidth != null)
    attrs['data-colwidth'] = (node.attrs.colwidth as string[]).join(',')

  return attrs
}

/**
 * Generate a `NodeSpec` to represent a Stencila `TableCell`.
 */
function tableCell(): NodeSpec {
  return {
    content: 'InlineContent*',
    attrs: tableCellAttrsSpec(),
    tableRole: 'cell',
    isolating: true,
    parseDOM: [
      { tag: 'td', getAttrs: (dom) => tableCellAttrsGet(dom as HTMLElement) },
    ],
    toDOM(node) {
      return ['td', tableCellAttrsSet(node), 0]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `TableCell` with `cellType` 'Header'.
 *
 * The reason this exists as a separate `NodeSpec` to `TableCell` is that the
 * `prosemirror-tables` package seems to want to have a node type with `tableRole: header_cell`
 * presumably for its commands to work.
 *
 * See https://github.com/ProseMirror/prosemirror-tables/blob/master/src/schema.js#L96
 */
function tableHeader(): NodeSpec {
  return {
    content: 'InlineContent*',
    attrs: tableCellAttrsSpec(),
    tableRole: 'header_cell',
    isolating: true,
    parseDOM: [
      { tag: 'th', getAttrs: (dom) => tableCellAttrsGet(dom as HTMLElement) },
    ],
    toDOM(node) {
      return ['th', tableCellAttrsSet(node), 0]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `ThematicBreak`
 */
function thematicBreak(): NodeSpec {
  return {
    group: 'BlockContent',
    parseDOM: [{ tag: 'hr' }],
    toDOM(_node) {
      return [
        'hr',
        { itemtype: 'http://schema.stenci.la/ThematicBreak', itemscope: '' },
      ]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `InlineContent` node type.
 */
function _inline(
  name: string,
  tag: string,
  content: string,
  marks = '_'
): NodeSpec {
  return {
    group: 'InlineContent',
    inline: true,
    content,
    marks,
    parseDOM: [{ tag }],
    toDOM(_node) {
      return [
        tag,
        { itemtype: `http://schema.stenci.la/${name}`, itemscope: '' },
        0,
      ]
    },
  }
}

/**
 * Generate a `MarkSpec` to represent a Stencila inline node type.
 *
 * @param name The name of the type e.g. `Paragraph`
 * @param tag The tag name of the HTML element for `toDOM` and `parseDOM`)
 * @param parseDOM: The parse rules for the mark
 */
function mark(name: string, tag: string, parseDOM?: ParseRule[]): MarkSpec {
  return {
    parseDOM: parseDOM ?? [{ tag }],
    toDOM(_node) {
      return [tag, { itemtype: name, itemscope: '' }, 0]
    },
  }
}
