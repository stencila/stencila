import { Attrs, Node, NodeSpec, ParseRule } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import StencilaCall from '../../../nodes/call'
import { StencilaExecutableView } from './executable'
import { includeAttrs } from './include'

export function call(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'CallArgument*',
    attrs: includeAttrs,
    parseDOM,
    toDOM,
  }
}

export class StencilaCallView extends StencilaExecutableView<StencilaCall> {
  constructor(node: Node, view: EditorView, getPos: () => number) {
    super(node, view, getPos, getAttrs, toDOM)
  }
}

const parseDOM: ParseRule[] = [
  {
    tag: 'stencila-call',
    getAttrs,
    contentElement: '[slot=arguments]',
  },
]

function getAttrs(node: StencilaCall): Attrs {
  return {
    id: node.id,
    source: node.getAttribute('source'),
    select: node.getAttribute('select') ?? undefined,
    errors: node.querySelector('[slot=errors]')?.innerHTML,
    content: node.querySelector('[slot=content]')?.innerHTML,
  }
}

function toDOM(node: Node) {
  const dom = document.createElement('stencila-call')
  dom.contentEditable = 'false'
  dom.draggable = true
  dom.id = node.attrs.id
  dom.setAttribute('source', node.attrs.source)
  dom.setAttribute('select', node.attrs.select)

  const errors = document.createElement('div')
  errors.slot = 'errors'
  errors.innerHTML = node.attrs.errors
  dom.appendChild(errors)

  // Note: the `arguments` property is assigned to the `contentDOM` for this node type
  // (the same as how `clauses` are the content of `If` blocks)
  const contentDOM = document.createElement('div')
  contentDOM.slot = 'arguments'
  dom.appendChild(contentDOM)

  // Note: the `content` property is not editable so we just store it
  // on the node as HTML, not as a `contentDOM`, and reduce it's opacity
  const content = document.createElement('div')
  content.slot = 'content'
  content.innerHTML = node.attrs.content
  content.setAttribute('style', 'opacity: 0.75;')
  dom.appendChild(content)

  return { dom, contentDOM }
}
