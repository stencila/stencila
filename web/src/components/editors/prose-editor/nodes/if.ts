import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaIf from '../../../nodes/if'
import { StencilaExecutableView, executableAttrs } from './executable'

export function if_(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'IfClause+',
    attrs: executableAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaIfView extends StencilaExecutableView<StencilaIf> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-if',
    getAttrs,
    contentElement: '[slot=clauses]',
    consuming: true,
  },
]

function getAttrs(node: StencilaIf): Attrs {
  return {
    id: node.id,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-if')
  dom.draggable = true
  dom.id = node.attrs.id

  const contentDOM = document.createElement('div')
  contentDOM.slot = 'clauses'
  dom.appendChild(contentDOM)

  return { dom, contentDOM }
}
