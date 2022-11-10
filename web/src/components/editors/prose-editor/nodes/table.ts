import { AttributeSpec, DOMOutputSpec, Node, NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `Table`
 *
 * This, and other table related schemas are compatible with `prosemirror-tables` (e.g. `tableRole`
 * and `isolating`) attributes but with Stencila compatible naming.
 *
 * See https://github.com/ProseMirror/prosemirror-tables/blob/master/src/schema.js
 */
export function table(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'TableRow+',
    contentProp: 'rows',
    tableRole: 'table',
    isolating: true,
    parseDOM: [{ tag: 'table' }],
    toDOM(_node) {
      return ['table', ['tbody', 0]]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `TableRow`.
 */
export function tableRow(): NodeSpec {
  return {
    content: '(TableHeader|TableCell)*',
    contentProp: 'cells',
    tableRole: 'row',
    parseDOM: [{ tag: 'tr' }],
    toDOM(_node) {
      return ['tr', 0]
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
    itemtype: 'https://schema.stenci.la/TableCell',
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
export function tableCell(): NodeSpec {
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
export function tableHeader(): NodeSpec {
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
